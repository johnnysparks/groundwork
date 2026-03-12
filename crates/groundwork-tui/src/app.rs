use std::time::Duration;

use bevy_ecs::prelude::*;
use ratatui::DefaultTerminal;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use groundwork_sim::voxel::Material;
use groundwork_sim::Tick;

use crate::camera::Camera;
use crate::input;
use crate::render;
use crate::render3d;

/// Which view mode the TUI is displaying.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    /// Classic 2D slice view (one Z-layer at a time).
    Slice2D,
    /// Projected 3D camera view.
    Projected3D,
}

/// Gardening tools — each has specific placement behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Shovel,
    SeedBag,
    WateringCan,
    Soil,
    Stone,
}

const TOOL_PALETTE: [Tool; 5] = [
    Tool::SeedBag,
    Tool::WateringCan,
    Tool::Soil,
    Tool::Stone,
    Tool::Shovel,
];

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Shovel => "shovel",
            Tool::SeedBag => "seed bag",
            Tool::WateringCan => "watering can",
            Tool::Soil => "soil",
            Tool::Stone => "stone",
        }
    }

    pub fn material(self) -> Material {
        match self {
            Tool::Shovel => Material::Air,
            Tool::SeedBag => Material::Seed,
            Tool::WateringCan => Material::Water,
            Tool::Soil => Material::Soil,
            Tool::Stone => Material::Stone,
        }
    }

    /// Map a material name from CLI to the corresponding tool.
    pub fn from_material_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "air" | "shovel" | "dig" => Some(Tool::Shovel),
            "seed" | "seeds" => Some(Tool::SeedBag),
            "water" => Some(Tool::WateringCan),
            "soil" => Some(Tool::Soil),
            "stone" => Some(Tool::Stone),
            _ => None,
        }
    }
}

/// Apply a tool at a single cell. Handles gravity and tool-specific rules.
/// Returns true if the tool had an effect.
pub fn apply_tool(grid: &mut VoxelGrid, tool: Tool, x: usize, y: usize, z: usize) -> bool {
    // Shovel: dig out whatever is here, no protection
    if tool == Tool::Shovel {
        if let Some(voxel) = grid.get_mut(x, y, z) {
            if voxel.material == Material::Air {
                return false; // nothing to dig
            }
            voxel.set_material(Material::Air);
            return true;
        }
        return false;
    }

    // Other tools: check if target cell is Air, then apply gravity
    if let Some(voxel) = grid.get(x, y, z) {
        if voxel.material != Material::Air {
            return false; // cell is occupied
        }
    } else {
        return false; // out of bounds
    }

    // Apply gravity: drop through Air to find landing position
    let landing_z = match tool {
        Tool::SeedBag | Tool::WateringCan | Tool::Soil => grid.find_landing_z(x, y, z),
        _ => z,
    };

    // Check landing cell is still Air
    if let Some(landing) = grid.get(x, y, landing_z) {
        if landing.material != Material::Air {
            return false;
        }
    } else {
        return false;
    }

    let mat = tool.material();

    // Seed bag: seeds die on stone (can't root into stone)
    if tool == Tool::SeedBag && landing_z > 0 {
        if let Some(below) = grid.get(x, y, landing_z - 1) {
            if below.material == Material::Stone {
                return false;
            }
        }
    }

    // Watering can: no-op if landing on existing water
    if tool == Tool::WateringCan {
        if landing_z > 0 {
            if let Some(below) = grid.get(x, y, landing_z - 1) {
                if below.material == Material::Water {
                    return false;
                }
            }
        }
    }

    // Place the material
    if let Some(voxel) = grid.get_mut(x, y, landing_z) {
        voxel.set_material(mat);
        true
    } else {
        false
    }
}

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
    pub selected_tool: usize, // index into TOOL_PALETTE
    pub tool_active: bool,
    pub tool_start: Option<(usize, usize, usize)>,
    // UI panels (toggleable sections in side panel)
    pub show_inspect: bool,
    pub show_status: bool,
    pub show_controls: bool,
    // View mode: 2D slice or 3D projected
    pub view_mode: ViewMode,
    /// 3D camera state (created on first switch to 3D mode).
    pub camera: Option<Camera>,
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
            selected_tool: 0,
            tool_active: false,
            tool_start: None,
            show_inspect: true,
            show_status: false,
            show_controls: true,
            view_mode: ViewMode::Slice2D,
            camera: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while self.running {
            terminal.draw(|frame| {
                match self.view_mode {
                    ViewMode::Slice2D => render::draw(frame, &self.world, self),
                    ViewMode::Projected3D => render3d::draw_3d(frame, self),
                }
            })?;

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

    pub fn selected_tool(&self) -> Tool {
        TOOL_PALETTE[self.selected_tool]
    }

    pub fn cycle_tool(&mut self, forward: bool) {
        if forward {
            self.selected_tool = (self.selected_tool + 1) % TOOL_PALETTE.len();
        } else {
            self.selected_tool = (self.selected_tool + TOOL_PALETTE.len() - 1) % TOOL_PALETTE.len();
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
    pub fn tool_start_op(&mut self) {
        self.sync_focus_from_camera();
        self.tool_active = true;
        self.tool_start = Some((self.focus_x, self.focus_y, self.focus_z));
    }

    /// End tool operation: apply tool from start to current focus.
    pub fn tool_end(&mut self) {
        self.sync_focus_from_camera();
        if let Some((sx, sy, sz)) = self.tool_start.take() {
            let tool = self.selected_tool();
            let xlo = sx.min(self.focus_x);
            let xhi = sx.max(self.focus_x);
            let ylo = sy.min(self.focus_y);
            let yhi = sy.max(self.focus_y);
            let zlo = sz.min(self.focus_z);
            let zhi = sz.max(self.focus_z);

            let mut grid = self.world.resource_mut::<VoxelGrid>();
            for z in zlo..=zhi {
                for y in ylo..=yhi {
                    for x in xlo..=xhi {
                        apply_tool(&mut grid, tool, x, y, z);
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

    /// Toggle between 2D slice and 3D projected view.
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Slice2D => {
                // Initialize camera on first switch, or sync focus
                self.ensure_camera();
                ViewMode::Projected3D
            }
            ViewMode::Projected3D => {
                // Sync app focus back from camera position
                if let Some(ref cam) = self.camera {
                    let fx = (cam.focus.x as usize).min(GRID_X - 1);
                    let fy = (cam.focus.y as usize).min(GRID_Y - 1);
                    let fz = (cam.focus.z as usize).min(GRID_Z - 1);
                    self.focus_x = fx;
                    self.focus_y = fy;
                    self.focus_z = fz;
                }
                ViewMode::Slice2D
            }
        };
    }

    /// Sync the discrete app focus from the camera's continuous focus.
    /// Called before tool operations in 3D mode so tools target the right voxel.
    fn sync_focus_from_camera(&mut self) {
        if self.view_mode == ViewMode::Projected3D {
            if let Some(ref cam) = self.camera {
                self.focus_x = (cam.focus.x as usize).min(GRID_X - 1);
                self.focus_y = (cam.focus.y as usize).min(GRID_Y - 1);
                self.focus_z = (cam.focus.z as usize).min(GRID_Z - 1);
            }
        }
    }

    /// Ensure the 3D camera exists and is synced to the current focus.
    fn ensure_camera(&mut self) {
        if let Some(ref mut cam) = self.camera {
            cam.sync_focus(self.focus_x, self.focus_y, self.focus_z);
        } else {
            self.camera = Some(Camera::new(
                self.focus_x as f64 + 0.5,
                self.focus_y as f64 + 0.5,
                self.focus_z as f64 + 0.5,
            ));
        }
    }
}
