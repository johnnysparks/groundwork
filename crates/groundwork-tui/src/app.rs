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
    /// Focus position — always rendered at the center of the viewport.
    /// WASD moves focus, which pans the viewport over the world.
    pub focus_x: usize,
    pub focus_y: usize,
    pub focus_z: usize,
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
            focus_x: 30,
            focus_y: 30,
            focus_z: GROUND_LEVEL + 1,
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

    /// Move the focus (and viewport) by dx/dy within grid bounds.
    pub fn pan(&mut self, dx: isize, dy: isize) {
        let nx = self.focus_x as isize + dx;
        let ny = self.focus_y as isize + dy;
        if nx >= 0 && (nx as usize) < GRID_X {
            self.focus_x = nx as usize;
        }
        if ny >= 0 && (ny as usize) < GRID_Y {
            self.focus_y = ny as usize;
        }
    }

    pub fn move_z(&mut self, dz: isize) {
        let nz = self.focus_z as isize + dz;
        if nz >= 0 && (nz as usize) < GRID_Z {
            self.focus_z = nz as usize;
        }
    }

    /// Compute the viewport origin for a given screen size.
    /// Returns (world_x_start, world_y_start) such that focus is centered.
    /// Coordinates can be negative (meaning the viewport extends beyond world edge).
    pub fn viewport_origin(&self, viewport_cols: usize, viewport_rows: usize) -> (isize, isize) {
        let half_w = viewport_cols as isize / 2;
        let half_h = viewport_rows as isize / 2;
        (self.focus_x as isize - half_w, self.focus_y as isize - half_h)
    }

    /// Begin tool operation at current focus.
    pub fn tool_start(&mut self) {
        self.tool_active = true;
        self.tool_start = Some((self.focus_x, self.focus_y, self.focus_z));
    }

    /// End tool operation: fill from start to current focus with selected material.
    pub fn tool_end(&mut self) {
        if let Some((sx, sy, sz)) = self.tool_start.take() {
            let mat = self.selected_material();
            let xlo = sx.min(self.focus_x);
            let xhi = sx.max(self.focus_x);
            let ylo = sy.min(self.focus_y);
            let yhi = sy.max(self.focus_y);
            let zlo = sz.min(self.focus_z);
            let zhi = sz.max(self.focus_z);

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
