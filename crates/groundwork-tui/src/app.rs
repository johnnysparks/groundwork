use std::time::Duration;

use bevy_ecs::prelude::*;
use ratatui::DefaultTerminal;

use groundwork_sim::grid::{GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use groundwork_sim::voxel::Material;
use groundwork_sim::Tick;

use crate::input;
use crate::render;

/// Which material the player has selected for placement.
const MATERIAL_PALETTE: [Material; 6] = [
    Material::Seed,
    Material::Soil,
    Material::Water,
    Material::Stone,
    Material::Root,
    Material::Air,
];

pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub running: bool,
    pub auto_tick: bool,
    pub tick_rate_ms: u64,
    // Cursor
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub slice_z: usize,
    // Tool mode
    pub selected_material: usize, // index into MATERIAL_PALETTE
    pub tool_active: bool,
    pub tool_start: Option<(usize, usize, usize)>,
    // UI panels
    pub show_inspect: bool,
    pub show_status: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: groundwork_sim::create_world(),
            schedule: groundwork_sim::create_schedule(),
            running: true,
            auto_tick: false,
            tick_rate_ms: 200,
            cursor_x: 30,
            cursor_y: 30,
            slice_z: GROUND_LEVEL + 1,
            selected_material: 0,
            tool_active: false,
            tool_start: None,
            show_inspect: false,
            show_status: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while self.running {
            terminal.draw(|frame| render::draw(frame, &self.world, self))?;

            let timeout = Duration::from_millis(self.tick_rate_ms);
            if ratatui::crossterm::event::poll(timeout)? {
                input::handle_event(self)?;
            } else if self.auto_tick {
                groundwork_sim::tick(&mut self.world, &mut self.schedule);
            }
        }
        Ok(())
    }

    pub fn tick_count(&self) -> u64 {
        self.world.resource::<Tick>().0
    }

    pub fn step(&mut self) {
        groundwork_sim::tick(&mut self.world, &mut self.schedule);
    }

    pub fn selected_material(&self) -> Material {
        MATERIAL_PALETTE[self.selected_material]
    }

    pub fn cycle_material(&mut self, forward: bool) {
        if forward {
            self.selected_material = (self.selected_material + 1) % MATERIAL_PALETTE.len();
        } else {
            self.selected_material = (self.selected_material + MATERIAL_PALETTE.len() - 1) % MATERIAL_PALETTE.len();
        }
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        let nx = self.cursor_x as isize + dx;
        let ny = self.cursor_y as isize + dy;
        if nx >= 0 && (nx as usize) < GRID_X {
            self.cursor_x = nx as usize;
        }
        if ny >= 0 && (ny as usize) < GRID_Y {
            self.cursor_y = ny as usize;
        }
    }

    pub fn move_z(&mut self, dz: isize) {
        let nz = self.slice_z as isize + dz;
        if nz >= 0 && (nz as usize) < GRID_Z {
            self.slice_z = nz as usize;
        }
    }

    /// Begin tool operation at current cursor.
    pub fn tool_start(&mut self) {
        self.tool_active = true;
        self.tool_start = Some((self.cursor_x, self.cursor_y, self.slice_z));
    }

    /// End tool operation: fill from start to current cursor with selected material.
    pub fn tool_end(&mut self) {
        if let Some((sx, sy, sz)) = self.tool_start.take() {
            let mat = self.selected_material();
            let xlo = sx.min(self.cursor_x);
            let xhi = sx.max(self.cursor_x);
            let ylo = sy.min(self.cursor_y);
            let yhi = sy.max(self.cursor_y);
            let zlo = sz.min(self.slice_z);
            let zhi = sz.max(self.slice_z);

            use groundwork_sim::grid::VoxelGrid;
            let mut grid = self.world.resource_mut::<VoxelGrid>();
            for z in zlo..=zhi {
                for y in ylo..=yhi {
                    for x in xlo..=xhi {
                        if let Some(voxel) = grid.get_mut(x, y, z) {
                            let existing = voxel.material;
                            // Protect seeds and roots
                            if existing == Material::Seed || existing == Material::Root {
                                continue;
                            }
                            voxel.set_material(mat);
                        }
                    }
                }
            }
        }
        self.tool_active = false;
    }

    /// Cancel tool without applying.
    pub fn tool_cancel(&mut self) {
        self.tool_active = false;
        self.tool_start = None;
    }
}
