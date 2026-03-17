#!/usr/bin/env bash
# sync-check.sh — Verify WASM bridge exports (Rust) are consumed in TypeScript.
#
# Catches: new Rust exports with no TS consumer, removed Rust exports still
# referenced in TS, and enum variants that exist in Rust but are missing from
# the TS bridge or contract tests.
#
# Run: ./scripts/sync-check.sh
# Exit: 0 = all good, 1 = sync issues found.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM_BRIDGE="$ROOT/crates/groundwork-sim/src/wasm_bridge.rs"
TS_BRIDGE="$ROOT/crates/groundwork-web/src/bridge.ts"
CONTRACT_TEST="$ROOT/crates/groundwork-web/src/bridge.contract.test.ts"
FAUNA_RS="$ROOT/crates/groundwork-sim/src/fauna.rs"
VOXEL_RS="$ROOT/crates/groundwork-sim/src/voxel.rs"
GNOME_RS="$ROOT/crates/groundwork-sim/src/gnome.rs"
TREE_RS="$ROOT/crates/groundwork-sim/src/tree.rs"

errors=0
warnings=0

heading() { printf "\n\033[1;36m=== %s ===\033[0m\n" "$1"; }
ok()      { printf "  \033[32m✓\033[0m %s\n" "$1"; }
warn()    { printf "  \033[33m⚠\033[0m %s\n" "$1"; warnings=$((warnings + 1)); }
fail()    { printf "  \033[31m✗\033[0m %s\n" "$1"; errors=$((errors + 1)); }

# Helper: check enum variants match between Rust source and TS const object.
# Usage: check_enum <label> <rust_file> <rust_enum_name> <ts_const_name>
check_enum() {
  local label="$1" rs_file="$2" rs_enum="$3" ts_const="$4"

  heading "$label ($rs_enum ↔ $ts_const)"

  local rust_variants
  rust_variants=$(sed -n "/pub enum $rs_enum/,/^}/p" "$rs_file" \
    | grep -E '^[[:space:]]+[A-Za-z]+ = [0-9]+,' \
    | sed 's/^[[:space:]]*//; s/,$//')

  local ts_block
  ts_block=$(grep -A20 "^export const $ts_const " "$TS_BRIDGE" | head -20)

  while IFS= read -r line; do
    local name value
    name=$(echo "$line" | sed 's/[[:space:]]*=.*//')
    value=$(echo "$line" | sed 's/.*= //')
    if echo "$ts_block" | grep -q "$name: $value"; then
      ok "$ts_const.$name = $value"
    else
      fail "$ts_const.$name = $value in Rust but not matched in bridge.ts"
    fi
  done <<< "$rust_variants"

  # Check contract test coverage
  while IFS= read -r line; do
    local name
    name=$(echo "$line" | sed 's/[[:space:]]*=.*//')
    if grep -q "$name:" "$CONTRACT_TEST"; then
      ok "$ts_const.$name covered in contract tests"
    else
      fail "$ts_const.$name missing from contract tests"
    fi
  done <<< "$rust_variants"
}

# ---------- 1. WASM exports vs TS bridge references ----------
heading "WASM exports → TypeScript bridge"

# Extract all #[wasm_bindgen] pub fn names from Rust
rust_exports=$(grep -B1 'pub fn ' "$WASM_BRIDGE" \
  | grep -A1 '#\[wasm_bindgen\]' \
  | grep 'pub fn ' \
  | sed 's/.*pub fn \([a-z_0-9]*\).*/\1/')

missing_in_ts=()
for fn in $rust_exports; do
  # Check if bridge.ts references wasmModule.<fn_name> or wasmModule?.<fn_name>
  if grep -qE "wasmModule\??\.${fn}[^a-z_0-9]" "$TS_BRIDGE"; then
    ok "$fn"
  else
    # Some exports are referenced indirectly (e.g., wasm.init())
    if grep -q "$fn" "$TS_BRIDGE"; then
      ok "$fn (indirect reference)"
    else
      warn "$fn — exported from Rust but not consumed in bridge.ts"
      missing_in_ts+=("$fn")
    fi
  fi
done

# ---------- 2. TS bridge references vs Rust exports ----------
heading "TypeScript bridge → WASM exports"

# Extract all wasmModule.xxx and wasmModule?.xxx references from TS
ts_refs=$(grep -oE 'wasmModule\??\.\w+' "$TS_BRIDGE" \
  | sed 's/wasmModule?\.//; s/wasmModule\.//' \
  | sort -u)

for ref in $ts_refs; do
  if echo "$rust_exports" | grep -qx "$ref"; then
    ok "$ref"
  else
    # 'default' is the WASM init function, not a sim export
    if [ "$ref" = "default" ]; then
      ok "$ref (WASM init — expected)"
    else
      fail "$ref — referenced in bridge.ts but not exported from wasm_bridge.rs"
    fi
  fi
done

# ---------- 3. Material enum (special: not inside a pub enum block) ----------
heading "Material enum (voxel.rs ↔ bridge.ts)"

rust_materials=$(sed -n '/pub enum Material/,/^}/p' "$VOXEL_RS" \
  | grep -E '^[[:space:]]+[A-Za-z]+ = [0-9]+,' \
  | sed 's/.*#\[default\]//; s/^[[:space:]]*//; s/,$//')

ts_material_block=$(grep -A20 "^export const Material" "$TS_BRIDGE" | head -15)

while IFS= read -r line; do
  name=$(echo "$line" | sed 's/[[:space:]]*=.*//')
  value=$(echo "$line" | sed 's/.*= //')
  if echo "$ts_material_block" | grep -q "$name: $value"; then
    ok "Material.$name = $value"
  else
    fail "Material.$name = $value in Rust but not matched in bridge.ts"
  fi
done <<< "$rust_materials"

# Material contract test coverage
while IFS= read -r line; do
  name=$(echo "$line" | sed 's/[[:space:]]*=.*//')
  if grep -q "$name:" "$CONTRACT_TEST"; then
    ok "Material.$name covered in contract tests"
  else
    fail "Material.$name missing from contract tests"
  fi
done <<< "$rust_materials"

# ---------- 4-7. Enum checks ----------
check_enum "FaunaType enum"  "$FAUNA_RS" "FaunaType"  "FaunaType"
check_enum "FaunaState enum" "$FAUNA_RS" "FaunaState"  "FaunaState"
check_enum "GnomeState enum" "$GNOME_RS" "GnomeState"  "GnomeState"
check_enum "GrowthStage enum" "$TREE_RS" "GrowthStage" "GrowthStage"

# ---------- Summary ----------
echo ""
if [ $errors -gt 0 ]; then
  printf "\033[31m✗ %d sync error(s) found.\033[0m " "$errors"
  printf "Fix these before committing.\n"
  exit 1
elif [ $warnings -gt 0 ]; then
  printf "\033[33m⚠ %d warning(s), 0 errors.\033[0m\n" "$warnings"
  exit 0
else
  printf "\033[32m✓ Sim ↔ Web bridge fully synchronized.\033[0m\n"
  exit 0
fi
