#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Material {
    #[default]
    Air = 0,
    Soil = 1,
    Stone = 2,
    Water = 3,
    Root = 4,
    Seed = 5,
    Trunk = 6,
    Branch = 7,
    Leaf = 8,
    DeadWood = 9,
}

impl Material {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Air),
            1 => Some(Self::Soil),
            2 => Some(Self::Stone),
            3 => Some(Self::Water),
            4 => Some(Self::Root),
            5 => Some(Self::Seed),
            6 => Some(Self::Trunk),
            7 => Some(Self::Branch),
            8 => Some(Self::Leaf),
            9 => Some(Self::DeadWood),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Soil => "soil",
            Self::Stone => "stone",
            Self::Water => "water",
            Self::Root => "root",
            Self::Seed => "seed",
            Self::Trunk => "trunk",
            Self::Branch => "branch",
            Self::Leaf => "leaf",
            Self::DeadWood => "deadwood",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "air" => Some(Self::Air),
            "soil" => Some(Self::Soil),
            "stone" => Some(Self::Stone),
            "water" => Some(Self::Water),
            "root" => Some(Self::Root),
            "seed" => Some(Self::Seed),
            "trunk" => Some(Self::Trunk),
            "branch" => Some(Self::Branch),
            "leaf" => Some(Self::Leaf),
            "deadwood" => Some(Self::DeadWood),
            _ => None,
        }
    }

    /// Whether this material generates solid mesh geometry.
    /// Air, Water, Leaf, and Seed are non-solid (rendered separately or not at all).
    pub fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Water | Self::Leaf | Self::Seed)
    }

    /// Whether this material is foliage rendered as billboard sprites.
    pub fn is_foliage(self) -> bool {
        matches!(self, Self::Leaf)
    }

    /// Whether this material is a seed (rendered as small sprites).
    pub fn is_seed(self) -> bool {
        matches!(self, Self::Seed)
    }

    /// Total number of material variants.
    pub const COUNT: u8 = 10;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_u8_round_trip() {
        let all = [
            Material::Air,
            Material::Soil,
            Material::Stone,
            Material::Water,
            Material::Root,
            Material::Seed,
            Material::Trunk,
            Material::Branch,
            Material::Leaf,
            Material::DeadWood,
        ];
        for mat in all {
            assert_eq!(Material::from_u8(mat.as_u8()), Some(mat));
        }
    }

    #[test]
    fn material_from_u8_invalid() {
        assert_eq!(Material::from_u8(10), None);
        assert_eq!(Material::from_u8(255), None);
    }

    #[test]
    fn material_name_round_trip() {
        let all = [
            Material::Air,
            Material::Soil,
            Material::Stone,
            Material::Water,
            Material::Root,
            Material::Seed,
            Material::Trunk,
            Material::Branch,
            Material::Leaf,
            Material::DeadWood,
        ];
        for mat in all {
            assert_eq!(Material::from_name(mat.name()), Some(mat));
        }
    }

    #[test]
    fn material_from_name_case_insensitive() {
        assert_eq!(Material::from_name("Water"), Some(Material::Water));
        assert_eq!(Material::from_name("SOIL"), Some(Material::Soil));
        assert_eq!(Material::from_name("unknown"), None);
    }

    #[test]
    fn set_material_resets_state() {
        let mut v = Voxel {
            material: Material::Soil,
            water_level: 200,
            light_level: 150,
            nutrient_level: 50,
        };
        v.set_material(Material::Stone);
        assert_eq!(v.material, Material::Stone);
        assert_eq!(v.water_level, 0);
        assert_eq!(v.light_level, 0);
        assert_eq!(v.nutrient_level, 0);
    }

    /// Sync guard: if the Material enum or Voxel struct layout changes,
    /// this test fails — reminding you to update bridge.ts and bridge.contract.test.ts.
    #[test]
    fn wasm_bridge_sync_guard() {
        // Material enum values (must match bridge.ts Material object)
        assert_eq!(
            Material::Air as u8,
            0,
            "Material::Air changed — update bridge.ts"
        );
        assert_eq!(Material::Soil as u8, 1);
        assert_eq!(Material::Stone as u8, 2);
        assert_eq!(Material::Water as u8, 3);
        assert_eq!(Material::Root as u8, 4);
        assert_eq!(Material::Seed as u8, 5);
        assert_eq!(Material::Trunk as u8, 6);
        assert_eq!(Material::Branch as u8, 7);
        assert_eq!(Material::Leaf as u8, 8);
        assert_eq!(Material::DeadWood as u8, 9);
        assert_eq!(
            Material::COUNT,
            10,
            "Material variant count changed — update bridge.ts and bridge.contract.test.ts"
        );

        // Voxel struct: 4 bytes, repr(C), field order matters for zero-copy
        assert_eq!(
            std::mem::size_of::<Voxel>(),
            4,
            "Voxel size changed — update VOXEL_BYTES in bridge.ts"
        );
        assert_eq!(std::mem::align_of::<Voxel>(), 1);

        // Verify byte layout: [material, water_level, light_level, nutrient_level]
        let v = Voxel {
            material: Material::Stone,
            water_level: 100,
            light_level: 200,
            nutrient_level: 50,
        };
        let bytes: [u8; 4] = unsafe { std::mem::transmute(v) };
        assert_eq!(bytes[0], Material::Stone as u8, "material at offset 0");
        assert_eq!(bytes[1], 100, "water_level at offset 1");
        assert_eq!(bytes[2], 200, "light_level at offset 2");
        assert_eq!(bytes[3], 50, "nutrient_level at offset 3");
    }

    #[test]
    fn set_material_water_gets_full_water_level() {
        let mut v = Voxel::default();
        v.set_material(Material::Water);
        assert_eq!(v.material, Material::Water);
        assert_eq!(v.water_level, 255);
        assert_eq!(v.light_level, 0);
        assert_eq!(v.nutrient_level, 0);
    }
}

/// A single cell in the voxel grid.
/// Kept small (4 bytes) so the full 60×60×30 grid fits in ~324 KB.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Voxel {
    pub material: Material,
    pub water_level: u8,
    pub light_level: u8,
    pub nutrient_level: u8,
}

impl Voxel {
    /// Change this voxel's material and reset all state fields to defaults
    /// for the new material. This prevents state bleed (e.g. stone retaining
    /// water_level from wet soil).
    ///
    /// Water voxels start with water_level=255; all others start at 0.
    /// Light is always reset to 0 (recalculated each tick by light_propagation).
    pub fn set_material(&mut self, mat: Material) {
        self.material = mat;
        self.water_level = if mat == Material::Water { 255 } else { 0 };
        self.light_level = 0;
        self.nutrient_level = 0;
    }
}
