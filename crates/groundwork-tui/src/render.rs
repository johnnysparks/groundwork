use bevy_ecs::world::World;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GROUND_LEVEL};
use groundwork_sim::voxel::Material;

use crate::app::App;

/// Map a voxel to an ASCII character + color.
fn voxel_style(mat: Material, water_level: u8, light_level: u8) -> (char, Color) {
    // Dim factor based on light (0.0–1.0 mapped to color brightness).
    let dim = |c: u8| -> u8 {
        ((c as u16 * light_level as u16) / 255) as u8
    };

    match mat {
        Material::Air => {
            if water_level > 200 {
                ('≈', Color::Rgb(dim(80), dim(140), dim(255)))
            } else if water_level > 0 {
                ('~', Color::Rgb(dim(80), dim(140), dim(255)))
            } else {
                (' ', Color::Reset)
            }
        }
        Material::Water => {
            let intensity = 100 + (water_level as u16 * 155 / 255) as u8;
            let ch = if water_level > 200 { '≈' } else { '~' };
            (ch, Color::Rgb(dim(40), dim(80), dim(intensity)))
        }
        Material::Soil => {
            if water_level > 100 {
                ('#', Color::Rgb(dim(80), dim(70), dim(100)))
            } else if water_level > 0 {
                ('#', Color::Rgb(dim(110), dim(80), dim(50)))
            } else {
                ('#', Color::Rgb(dim(139), dim(90), dim(43)))
            }
        }
        Material::Stone => ('@', Color::Rgb(dim(120), dim(120), dim(120))),
        Material::Root => ('*', Color::Rgb(dim(80), dim(180), dim(60))),
        Material::Seed => ('s', Color::Rgb(dim(200), dim(180), dim(60))),
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

    let mut lines: Vec<Line> = Vec::with_capacity(max_rows.min(GRID_Y));

    for y in 0..GRID_Y.min(max_rows) {
        let mut spans: Vec<Span> = Vec::with_capacity(max_cols.min(GRID_X));
        for x in 0..GRID_X.min(max_cols) {
            if let Some(voxel) = grid.get(x, y, z) {
                let (ch, fg) = voxel_style(voxel.material, voxel.water_level, voxel.light_level);
                spans.push(Span::styled(
                    String::from(ch),
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
