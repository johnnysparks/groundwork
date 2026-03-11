#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Material {
    #[default]
    Air = 0,
    Soil = 1,
    Stone = 2,
    Water = 3,
    Root = 4,
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
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "air" => Some(Self::Air),
            "soil" => Some(Self::Soil),
            "stone" => Some(Self::Stone),
            "water" => Some(Self::Water),
            "root" => Some(Self::Root),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_u8_round_trip() {
        let all = [Material::Air, Material::Soil, Material::Stone, Material::Water, Material::Root];
        for mat in all {
            assert_eq!(Material::from_u8(mat.as_u8()), Some(mat));
        }
    }

    #[test]
    fn material_from_u8_invalid() {
        assert_eq!(Material::from_u8(5), None);
        assert_eq!(Material::from_u8(255), None);
    }

    #[test]
    fn material_name_round_trip() {
        let all = [Material::Air, Material::Soil, Material::Stone, Material::Water, Material::Root];
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
}

/// A single cell in the voxel grid.
/// Kept small (4 bytes) so the full 60×60×30 grid fits in ~324 KB.
#[derive(Clone, Copy, Debug, Default)]
pub struct Voxel {
    pub material: Material,
    pub water_level: u8,
    pub light_level: u8,
    pub nutrient_level: u8,
}
