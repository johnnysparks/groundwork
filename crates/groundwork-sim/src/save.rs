use std::io;
use std::path::Path;

use bevy_ecs::prelude::*;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use crate::voxel::{Material, Voxel};
use crate::{FocusState, ToolState, Tick};

const MAGIC: [u8; 4] = *b"GWRK";
const VERSION: u16 = 2;
const VOXEL_COUNT: usize = GRID_X * GRID_Y * GRID_Z;
// V1: header(8) + tick(8) + voxels
const V1_SIZE: usize = 8 + 8 + VOXEL_COUNT * 4;
// V2: V1 + focus(6) + tool_active(1) + tool_material(1) + tool_start(6) = V1 + 14
const FOCUS_BLOCK: usize = 14;
const V2_SIZE: usize = V1_SIZE + FOCUS_BLOCK;

pub fn save_to_file(grid: &VoxelGrid, tick: &Tick, focus: &FocusState, path: &Path) -> io::Result<()> {
    let mut buf = Vec::with_capacity(V2_SIZE);

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

    // Focus state (V2)
    buf.extend_from_slice(&(focus.x as u16).to_le_bytes());
    buf.extend_from_slice(&(focus.y as u16).to_le_bytes());
    buf.extend_from_slice(&(focus.z as u16).to_le_bytes());
    if let Some(ref tool) = focus.tool {
        buf.push(1); // tool active
        buf.push(tool.material.as_u8());
        buf.extend_from_slice(&(tool.start_x as u16).to_le_bytes());
        buf.extend_from_slice(&(tool.start_y as u16).to_le_bytes());
        buf.extend_from_slice(&(tool.start_z as u16).to_le_bytes());
    } else {
        buf.push(0); // tool inactive
        buf.push(0);
        buf.extend_from_slice(&[0u8; 6]);
    }

    std::fs::write(path, &buf)
}

pub fn load_from_file(path: &Path) -> io::Result<(Vec<Voxel>, u64, FocusState)> {
    let data = std::fs::read(path)?;

    if data.len() < 16 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file too small"));
    }
    if &data[0..4] != &MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bad magic bytes"));
    }
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != 1 && version != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported version: {version}"),
        ));
    }

    let expected_size = if version == 1 { V1_SIZE } else { V2_SIZE };
    if data.len() != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected {expected_size} bytes, got {}", data.len()),
        ));
    }

    let tick = u64::from_le_bytes(data[8..16].try_into().unwrap());

    let mut cells = Vec::with_capacity(VOXEL_COUNT);
    let voxel_data = &data[16..16 + VOXEL_COUNT * 4];
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

    // Parse focus state (V2) or use defaults (V1)
    let focus = if version >= 2 {
        let fo = 16 + VOXEL_COUNT * 4;
        let fx = u16::from_le_bytes([data[fo], data[fo + 1]]) as usize;
        let fy = u16::from_le_bytes([data[fo + 2], data[fo + 3]]) as usize;
        let fz = u16::from_le_bytes([data[fo + 4], data[fo + 5]]) as usize;
        let tool_active = data[fo + 6];
        let tool = if tool_active != 0 {
            let mat = Material::from_u8(data[fo + 7]).unwrap_or(Material::Air);
            let sx = u16::from_le_bytes([data[fo + 8], data[fo + 9]]) as usize;
            let sy = u16::from_le_bytes([data[fo + 10], data[fo + 11]]) as usize;
            let sz = u16::from_le_bytes([data[fo + 12], data[fo + 13]]) as usize;
            Some(ToolState { material: mat, start_x: sx, start_y: sy, start_z: sz })
        } else {
            None
        };
        FocusState { x: fx, y: fy, z: fz, tool }
    } else {
        FocusState::default()
    };

    Ok((cells, tick, focus))
}

pub fn save_world(world: &World, path: &Path) -> io::Result<()> {
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>();
    let focus = world.resource::<FocusState>();
    save_to_file(grid, tick, focus, path)
}

pub fn load_world(path: &Path) -> io::Result<World> {
    let (cells, tick, focus) = load_from_file(path)?;
    let mut world = World::new();
    world.insert_resource(VoxelGrid::from_cells(cells));
    world.insert_resource(Tick(tick));
    world.insert_resource(focus);
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

        // Focus should round-trip
        let orig_focus = world.resource::<FocusState>();
        let loaded_focus = loaded.resource::<FocusState>();
        assert_eq!(orig_focus.x, loaded_focus.x);
        assert_eq!(orig_focus.y, loaded_focus.y);
        assert_eq!(orig_focus.z, loaded_focus.z);
        assert!(loaded_focus.tool.is_none());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn round_trip_with_tool_state() {
        let mut world = create_world();
        {
            let mut focus = world.resource_mut::<FocusState>();
            focus.x = 10;
            focus.y = 20;
            focus.z = 5;
            focus.tool = Some(ToolState {
                material: Material::Seed,
                start_x: 5,
                start_y: 15,
                start_z: 5,
            });
        }
        let path = std::env::temp_dir().join("groundwork_test_tool_rt.state");
        save_world(&world, &path).unwrap();
        let loaded = load_world(&path).unwrap();

        let f = loaded.resource::<FocusState>();
        assert_eq!(f.x, 10);
        assert_eq!(f.y, 20);
        assert_eq!(f.z, 5);
        let t = f.tool.as_ref().unwrap();
        assert_eq!(t.material, Material::Seed);
        assert_eq!(t.start_x, 5);
        assert_eq!(t.start_y, 15);
        assert_eq!(t.start_z, 5);

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
        let mut buf = Vec::with_capacity(V2_SIZE);
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&VERSION.to_le_bytes());
        buf.extend_from_slice(&[0u8; 2]);
        buf.extend_from_slice(&0u64.to_le_bytes());
        // Fill with invalid material byte 255
        for _ in 0..VOXEL_COUNT {
            buf.extend_from_slice(&[255, 0, 0, 0]);
        }
        // Focus block
        buf.extend_from_slice(&[0u8; FOCUS_BLOCK]);
        std::fs::write(&path, &buf).unwrap();
        let err = load_from_file(&path).unwrap_err();
        assert!(err.to_string().contains("invalid material"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn v1_backward_compatible() {
        // Build a valid V1 file
        let world = create_world();
        let grid = world.resource::<VoxelGrid>();
        let tick = world.resource::<Tick>();

        let mut buf = Vec::with_capacity(V1_SIZE);
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&1u16.to_le_bytes()); // version 1
        buf.extend_from_slice(&[0u8; 2]);
        buf.extend_from_slice(&tick.0.to_le_bytes());
        for v in grid.cells() {
            buf.push(v.material.as_u8());
            buf.push(v.water_level);
            buf.push(v.light_level);
            buf.push(v.nutrient_level);
        }

        let path = std::env::temp_dir().join("groundwork_test_v1_compat.state");
        std::fs::write(&path, &buf).unwrap();

        let loaded = load_world(&path).unwrap();
        // Should load with default focus
        let f = loaded.resource::<FocusState>();
        assert_eq!(f.x, 30);
        assert_eq!(f.y, 30);
        assert!(f.tool.is_none());

        let _ = std::fs::remove_file(&path);
    }
}
