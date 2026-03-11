use std::io;
use std::path::Path;

use bevy_ecs::prelude::*;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use crate::voxel::{Material, Voxel};
use crate::Tick;

const MAGIC: [u8; 4] = *b"GWRK";
const VERSION: u16 = 1;
const VOXEL_COUNT: usize = GRID_X * GRID_Y * GRID_Z;
const EXPECTED_SIZE: usize = 8 + 8 + VOXEL_COUNT * 4; // header + tick + voxels

pub fn save_to_file(grid: &VoxelGrid, tick: &Tick, path: &Path) -> io::Result<()> {
    let mut buf = Vec::with_capacity(EXPECTED_SIZE);

    // Header
    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&[0u8; 2]); // reserved

    // Tick
    buf.extend_from_slice(&tick.0.to_le_bytes());

    // Voxels
    for v in grid.cells() {
        buf.push(v.material.as_u8());
        buf.push(v.water_level);
        buf.push(v.light_level);
        buf.push(v.nutrient_level);
    }

    std::fs::write(path, &buf)
}

pub fn load_from_file(path: &Path) -> io::Result<(Vec<Voxel>, u64)> {
    let data = std::fs::read(path)?;

    if data.len() < 16 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file too small"));
    }
    if &data[0..4] != &MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bad magic bytes"));
    }
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported version: {version}"),
        ));
    }
    if data.len() != EXPECTED_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected {EXPECTED_SIZE} bytes, got {}", data.len()),
        ));
    }

    let tick = u64::from_le_bytes(data[8..16].try_into().unwrap());

    let mut cells = Vec::with_capacity(VOXEL_COUNT);
    let voxel_data = &data[16..];
    for i in 0..VOXEL_COUNT {
        let off = i * 4;
        let mat = Material::from_u8(voxel_data[off]).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid material byte: {}", voxel_data[off]),
            )
        })?;
        cells.push(Voxel {
            material: mat,
            water_level: voxel_data[off + 1],
            light_level: voxel_data[off + 2],
            nutrient_level: voxel_data[off + 3],
        });
    }

    Ok((cells, tick))
}

pub fn save_world(world: &World, path: &Path) -> io::Result<()> {
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>();
    save_to_file(grid, tick, path)
}

pub fn load_world(path: &Path) -> io::Result<World> {
    let (cells, tick) = load_from_file(path)?;
    let mut world = World::new();
    world.insert_resource(VoxelGrid::from_cells(cells));
    world.insert_resource(Tick(tick));
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_world;

    #[test]
    fn round_trip() {
        let world = create_world();
        let path = std::env::temp_dir().join("groundwork_test_roundtrip.state");

        save_world(&world, &path).unwrap();
        let loaded = load_world(&path).unwrap();

        let orig_grid = world.resource::<VoxelGrid>();
        let loaded_grid = loaded.resource::<VoxelGrid>();
        assert_eq!(orig_grid.cells().len(), loaded_grid.cells().len());

        for (a, b) in orig_grid.cells().iter().zip(loaded_grid.cells().iter()) {
            assert_eq!(a.material, b.material);
            assert_eq!(a.water_level, b.water_level);
            assert_eq!(a.light_level, b.light_level);
            assert_eq!(a.nutrient_level, b.nutrient_level);
        }

        assert_eq!(
            world.resource::<Tick>().0,
            loaded.resource::<Tick>().0,
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bad_magic() {
        let path = std::env::temp_dir().join("groundwork_test_bad_magic.state");
        std::fs::write(&path, b"BAAD").unwrap();
        let err = load_from_file(&path).unwrap_err();
        assert!(err.to_string().contains("magic") || err.to_string().contains("small"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tick_persists_across_save_load() {
        let mut world = create_world();
        let mut schedule = crate::create_schedule();
        for _ in 0..5 {
            crate::tick(&mut world, &mut schedule);
        }
        assert_eq!(world.resource::<Tick>().0, 5);

        let path = std::env::temp_dir().join("groundwork_test_tick_persist.state");
        save_world(&world, &path).unwrap();

        let loaded = load_world(&path).unwrap();
        assert_eq!(loaded.resource::<Tick>().0, 5);

        // Water should have spread after 5 ticks
        let orig_grid = world.resource::<VoxelGrid>();
        let loaded_grid = loaded.resource::<VoxelGrid>();
        for (a, b) in orig_grid.cells().iter().zip(loaded_grid.cells().iter()) {
            assert_eq!(a.material, b.material);
            assert_eq!(a.water_level, b.water_level);
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn invalid_material() {
        let path = std::env::temp_dir().join("groundwork_test_bad_mat.state");
        let mut buf = Vec::with_capacity(EXPECTED_SIZE);
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&VERSION.to_le_bytes());
        buf.extend_from_slice(&[0u8; 2]);
        buf.extend_from_slice(&0u64.to_le_bytes());
        // Fill with invalid material byte 255
        for _ in 0..VOXEL_COUNT {
            buf.extend_from_slice(&[255, 0, 0, 0]);
        }
        std::fs::write(&path, &buf).unwrap();
        let err = load_from_file(&path).unwrap_err();
        assert!(err.to_string().contains("invalid material"));
        let _ = std::fs::remove_file(&path);
    }
}
