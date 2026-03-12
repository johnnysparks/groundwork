use bevy_ecs::world::World;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GROUND_LEVEL};
use groundwork_sim::voxel::Material;

use crate::app::App;

/// Map a voxel to a single ASCII char + foreground color for TUI rendering.
/// Color is the primary material indicator; symbols will later differentiate
/// sub-properties (density, soil type, etc.).
fn voxel_style(mat: Material, water_level: u8, light_level: u8, nutrient_level: u8, below_ground: bool) -> (&'static str, Color) {
    // Dim factor based on light (0.0–1.0 mapped to color brightness).
    // Underground cells keep a minimum brightness so colors stay readable.
    let min_bright: u8 = if below_ground { 80 } else { 30 };
    let effective_light = light_level.max(min_bright);
    let dim = |c: u8| -> u8 {
        ((c as u16 * effective_light as u16) / 255).max(20) as u8
    };

    match mat {
        Material::Air => {
            if water_level > 0 {
                // Airborne moisture — blue tilde
                ("~", Color::Rgb(dim(100), dim(160), dim(255)))
            } else if below_ground {
                // Dark underground air
                (" ", Color::Reset)
            } else {
                (".", Color::Rgb(40, 40, 50))
            }
        }
        Material::Water => {
            // Blue tones — deeper water is more saturated
            let intensity = 140 + (water_level as u16 * 115 / 255) as u8;
            ("~", Color::Rgb(dim(30), dim(90), dim(intensity)))
        }
        Material::Soil => {
            // Brown tones — wetter soil is darker
            if water_level > 50 {
                // Wet soil — dark rich brown
                ("%", Color::Rgb(dim(66), dim(40), dim(14)))
            } else if water_level > 0 {
                // Moist soil — medium brown
                ("#", Color::Rgb(dim(101), dim(67), dim(33)))
            } else {
                // Dry soil — warm brown
                ("#", Color::Rgb(dim(160), dim(110), dim(60)))
            }
        }
        Material::Stone => {
            // Gray tones for rock
            ("@", Color::Rgb(dim(150), dim(150), dim(155)))
        }
        Material::Root => {
            if below_ground {
                // Light tan underground
                ("*", Color::Rgb(dim(210), dim(180), dim(140)))
            } else {
                // Green above ground (visible growth)
                ("*", Color::Rgb(dim(60), dim(180), dim(60)))
            }
        }
        Material::Seed => {
            if nutrient_level >= 100 {
                // Sprouting — bright green
                ("s", Color::Rgb(dim(80), dim(220), dim(60)))
            } else {
                // Dormant — muted yellow-brown
                ("s", Color::Rgb(dim(200), dim(170), dim(60)))
            }
        }
    }
}

pub fn draw(frame: &mut Frame, world: &World, app: &App) {
    let area = frame.area();

    let [grid_area, status_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).areas(area);

    let grid = world.resource::<VoxelGrid>();
    let z = app.slice_z;

    // Render the XY slice at the current Z level.
    let max_rows = grid_area.height as usize;
    let max_cols = grid_area.width as usize;

    let below_ground = z < GROUND_LEVEL;
    let mut lines: Vec<Line> = Vec::with_capacity(max_rows.min(GRID_Y));

    for y in 0..GRID_Y.min(max_rows) {
        let mut spans: Vec<Span> = Vec::with_capacity(max_cols.min(GRID_X));
        for x in 0..GRID_X.min(max_cols) {
            if let Some(voxel) = grid.get(x, y, z) {
                let (s, fg) = voxel_style(voxel.material, voxel.water_level, voxel.light_level, voxel.nutrient_level, below_ground);
                spans.push(Span::styled(
                    s,
                    Style::default().fg(fg),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    let grid_widget = Paragraph::new(lines);
    frame.render_widget(grid_widget, grid_area);

    // Status bar.
    let depth_label = if z > GROUND_LEVEL {
        format!("above +{}", z - GROUND_LEVEL)
    } else if z == GROUND_LEVEL {
        "surface".to_string()
    } else {
        format!("below -{}", GROUND_LEVEL - z)
    };

    let mode = if app.auto_tick {
        format!("AUTO {}ms", app.tick_rate_ms)
    } else {
        "MANUAL".to_string()
    };

    let status = format!(
        " Z:{z}/{max} ({depth})  Tick:{tick}  [{mode}]  \
         [Space]step [P]auto [J/K]depth [+/-]speed [Q]quit",
        z = z,
        max = groundwork_sim::grid::GRID_Z - 1,
        depth = depth_label,
        tick = app.tick_count(),
        mode = mode,
    );

    let status_widget = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(status_widget, status_area);
}
