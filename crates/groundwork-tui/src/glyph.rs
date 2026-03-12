/// Shape-aware glyph selection for the 3D projected terminal view.
///
/// Core idea (adapted from Alex Harri's "ASCII characters are not pixels"):
/// Each ASCII/Unicode character has a visual shape that occupies specific regions
/// of its cell. We precompute a 4-value descriptor for each glyph (coverage in
/// TL, TR, BL, BR quadrants). At render time, we compute the same descriptor
/// from projected voxel geometry and pick the nearest-match glyph.
///
/// The renderer produces binary coverage (0.0 or 1.0 per quadrant), giving 16
/// possible input patterns. The atlas covers all 16 with Unicode block elements
/// for precise shape matching, plus ASCII characters as softer alternatives
/// that win when their descriptors are a closer fit.

/// A glyph with its precomputed shape descriptor.
struct GlyphEntry {
    ch: char,
    /// Coverage in [top-left, top-right, bottom-left, bottom-right], each 0.0–1.0.
    coverage: [f32; 4],
}

/// Atlas of curated glyphs with precomputed shape descriptors.
pub struct GlyphAtlas {
    glyphs: Vec<GlyphEntry>,
}

impl GlyphAtlas {
    /// Build the curated glyph atlas.
    ///
    /// Three tiers of glyphs:
    /// 1. **Unicode block elements** — precise quadrant coverage, clean edges.
    ///    These give exact matches for all 16 binary coverage patterns.
    /// 2. **ASCII shape characters** — softer/textured alternatives for edges
    ///    and partial fills. Win over blocks when their descriptors are closer.
    /// 3. **Special characters** — material-specific or decorative glyphs
    ///    placed at fractional descriptors for visual variety.
    ///
    /// Descriptors are hand-tuned to match visual weight in a typical monospace
    /// terminal font. They don't need to be pixel-precise — they capture
    /// relative shape differences well enough for nearest-match selection.
    pub fn new() -> Self {
        let glyphs = vec![
            // ── Tier 1: Unicode block elements ──────────────────────────
            // Exact coverage for all 16 binary quadrant patterns.
            //
            // Pattern [TL, TR, BL, BR]

            // 0 quadrants filled
            GlyphEntry { ch: ' ',  coverage: [0.0, 0.0, 0.0, 0.0] },

            // 1 quadrant filled
            GlyphEntry { ch: '▘',  coverage: [1.0, 0.0, 0.0, 0.0] }, // top-left
            GlyphEntry { ch: '▝',  coverage: [0.0, 1.0, 0.0, 0.0] }, // top-right
            GlyphEntry { ch: '▖',  coverage: [0.0, 0.0, 1.0, 0.0] }, // bottom-left
            GlyphEntry { ch: '▗',  coverage: [0.0, 0.0, 0.0, 1.0] }, // bottom-right

            // 2 quadrants filled — halves
            GlyphEntry { ch: '▀',  coverage: [1.0, 1.0, 0.0, 0.0] }, // upper half
            GlyphEntry { ch: '▄',  coverage: [0.0, 0.0, 1.0, 1.0] }, // lower half
            GlyphEntry { ch: '▌',  coverage: [1.0, 0.0, 1.0, 0.0] }, // left half
            GlyphEntry { ch: '▐',  coverage: [0.0, 1.0, 0.0, 1.0] }, // right half

            // 2 quadrants filled — diagonals
            GlyphEntry { ch: '▚',  coverage: [1.0, 0.0, 0.0, 1.0] }, // TL + BR diagonal
            GlyphEntry { ch: '▞',  coverage: [0.0, 1.0, 1.0, 0.0] }, // TR + BL diagonal

            // 3 quadrants filled
            GlyphEntry { ch: '▛',  coverage: [1.0, 1.0, 1.0, 0.0] }, // missing BR
            GlyphEntry { ch: '▜',  coverage: [1.0, 1.0, 0.0, 1.0] }, // missing BL
            GlyphEntry { ch: '▙',  coverage: [1.0, 0.0, 1.0, 1.0] }, // missing TR
            GlyphEntry { ch: '▟',  coverage: [0.0, 1.0, 1.0, 1.0] }, // missing TL

            // 4 quadrants filled
            GlyphEntry { ch: '█',  coverage: [1.0, 1.0, 1.0, 1.0] }, // full block

            // ── Tier 2: ASCII shape characters ──────────────────────────
            // Softer alternatives at fractional descriptors. These win when
            // the renderer produces non-binary coverage (future) or when
            // material-specific overrides route through here.

            // Near-empty / sparse
            GlyphEntry { ch: '.',  coverage: [0.0, 0.0, 0.15, 0.15] },
            GlyphEntry { ch: ',',  coverage: [0.0, 0.0, 0.15, 0.0] },
            GlyphEntry { ch: '\'', coverage: [0.1, 0.1, 0.0, 0.0] },
            GlyphEntry { ch: ':',  coverage: [0.2, 0.2, 0.2, 0.2] },
            GlyphEntry { ch: ';',  coverage: [0.2, 0.2, 0.15, 0.0] },

            // Horizontal lines / top surfaces
            GlyphEntry { ch: '_',  coverage: [0.0, 0.0, 0.85, 0.85] },
            GlyphEntry { ch: '‾',  coverage: [0.85, 0.85, 0.0, 0.0] }, // overline
            GlyphEntry { ch: '-',  coverage: [0.35, 0.35, 0.35, 0.35] },
            GlyphEntry { ch: '─',  coverage: [0.35, 0.35, 0.35, 0.35] }, // box horizontal
            GlyphEntry { ch: '=',  coverage: [0.45, 0.45, 0.45, 0.45] },
            GlyphEntry { ch: '≡',  coverage: [0.55, 0.55, 0.55, 0.55] }, // triple bar

            // Vertical lines / walls
            GlyphEntry { ch: '│',  coverage: [0.3, 0.3, 0.3, 0.3] }, // box vertical
            GlyphEntry { ch: '|',  coverage: [0.3, 0.3, 0.3, 0.3] },
            GlyphEntry { ch: '!',  coverage: [0.4, 0.4, 0.1, 0.1] },

            // Diagonal / contour edges
            GlyphEntry { ch: '/',  coverage: [0.05, 0.65, 0.65, 0.05] },
            GlyphEntry { ch: '\\', coverage: [0.65, 0.05, 0.05, 0.65] },
            GlyphEntry { ch: '╱',  coverage: [0.05, 0.7, 0.7, 0.05] }, // box diagonal
            GlyphEntry { ch: '╲',  coverage: [0.7, 0.05, 0.05, 0.7] }, // box diagonal

            // Corner pieces — box drawing
            GlyphEntry { ch: '┌',  coverage: [0.0, 0.0, 0.5, 0.5] },
            GlyphEntry { ch: '┐',  coverage: [0.0, 0.0, 0.5, 0.5] },
            GlyphEntry { ch: '└',  coverage: [0.5, 0.5, 0.0, 0.0] },
            GlyphEntry { ch: '┘',  coverage: [0.5, 0.5, 0.0, 0.0] },

            // Brackets — left/right heavy
            GlyphEntry { ch: '[',  coverage: [0.65, 0.15, 0.65, 0.15] },
            GlyphEntry { ch: ']',  coverage: [0.15, 0.65, 0.15, 0.65] },
            GlyphEntry { ch: '(',  coverage: [0.3, 0.0, 0.3, 0.0] },
            GlyphEntry { ch: ')',  coverage: [0.0, 0.3, 0.0, 0.3] },
            GlyphEntry { ch: '{',  coverage: [0.35, 0.1, 0.35, 0.1] },
            GlyphEntry { ch: '}',  coverage: [0.1, 0.35, 0.1, 0.35] },

            // Arrow/pointer shapes
            GlyphEntry { ch: '^',  coverage: [0.45, 0.45, 0.15, 0.15] },
            GlyphEntry { ch: 'v',  coverage: [0.15, 0.15, 0.5, 0.5] },
            GlyphEntry { ch: '<',  coverage: [0.3, 0.0, 0.3, 0.0] },
            GlyphEntry { ch: '>',  coverage: [0.0, 0.3, 0.0, 0.3] },

            // ── Tier 3: Dense / textured fills ──────────────────────────
            // Medium to full density — used for fully covered cells where
            // material styling wants a textured look over solid blocks.

            GlyphEntry { ch: '░',  coverage: [0.25, 0.25, 0.25, 0.25] }, // light shade
            GlyphEntry { ch: '▒',  coverage: [0.5, 0.5, 0.5, 0.5] },    // medium shade
            GlyphEntry { ch: '▓',  coverage: [0.75, 0.75, 0.75, 0.75] }, // dark shade
            GlyphEntry { ch: '#',  coverage: [0.7, 0.7, 0.7, 0.7] },
            GlyphEntry { ch: '%',  coverage: [0.6, 0.6, 0.6, 0.6] },
            GlyphEntry { ch: '@',  coverage: [0.85, 0.85, 0.85, 0.85] },
            GlyphEntry { ch: '&',  coverage: [0.65, 0.65, 0.65, 0.65] },
            GlyphEntry { ch: '*',  coverage: [0.4, 0.4, 0.4, 0.4] },
            GlyphEntry { ch: '+',  coverage: [0.3, 0.3, 0.3, 0.3] },
            GlyphEntry { ch: '×',  coverage: [0.35, 0.35, 0.35, 0.35] },

            // ── Tier 4: Partial fills — asymmetric weight ───────────────
            // Characters with visual weight concentrated in specific regions.
            // These help when coverage is fractional or for future sub-pixel
            // refinement.

            GlyphEntry { ch: '⌐',  coverage: [0.7, 0.7, 0.3, 0.0] }, // top + left stub
            GlyphEntry { ch: '¬',  coverage: [0.7, 0.7, 0.0, 0.3] }, // top + right stub
            GlyphEntry { ch: '"',  coverage: [0.45, 0.45, 0.0, 0.0] },
            GlyphEntry { ch: '„',  coverage: [0.0, 0.0, 0.45, 0.45] }, // bottom double quote
            GlyphEntry { ch: '¡',  coverage: [0.1, 0.1, 0.4, 0.4] }, // inverted !
            GlyphEntry { ch: '·',  coverage: [0.15, 0.15, 0.15, 0.15] }, // middle dot
            GlyphEntry { ch: '•',  coverage: [0.35, 0.35, 0.35, 0.35] }, // bullet

            // ── Special: water/organic ──────────────────────────────────
            GlyphEntry { ch: '~',  coverage: [0.3, 0.4, 0.4, 0.3] },
            GlyphEntry { ch: '≈',  coverage: [0.4, 0.5, 0.5, 0.4] }, // double tilde
        ];

        Self { glyphs }
    }

    /// Find the glyph whose shape descriptor is closest to the given coverage.
    ///
    /// Uses squared Euclidean distance — no sqrt needed for comparison.
    pub fn best_match(&self, coverage: [f32; 4]) -> char {
        let mut best_ch = ' ';
        let mut best_dist = f32::MAX;

        for entry in &self.glyphs {
            let dist: f32 = (0..4)
                .map(|i| {
                    let d = coverage[i] - entry.coverage[i];
                    d * d
                })
                .sum();

            if dist < best_dist {
                best_dist = dist;
                best_ch = entry.ch;
            }
        }

        best_ch
    }
}
