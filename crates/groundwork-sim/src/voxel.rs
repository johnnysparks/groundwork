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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_u8_round_trip() {
        let all = [Material::Air, Material::Soil, Material::Stone, Material::Water, Material::Root, Material::Seed, Material::Trunk, Material::Branch, Material::Leaf, Material::DeadWood];
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
        let all = [Material::Air, Material::Soil, Material::Stone, Material::Water, Material::Root, Material::Seed, Material::Trunk, Material::Branch, Material::Leaf, Material::DeadWood];
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
