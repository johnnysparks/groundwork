use bevy_ecs::world::World;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GROUND_LEVEL};
use groundwork_sim::voxel::Material;

use crate::app::App;

/// Map a voxel to an emoji string + color for TUI rendering.
fn voxel_style(mat: Material, water_level: u8, light_level: u8, nutrient_level: u8) -> (&'static str, Color) {
    // Dim factor based on light (0.0–1.0 mapped to color brightness).
    let dim = |c: u8| -> u8 {
        ((c as u16 * light_level as u16) / 255) as u8
    };

    match mat {
        Material::Air => {
            if water_level > 0 {
                ("💧", Color::Rgb(dim(80), dim(140), dim(255)))
            } else {
                ("  ", Color::Reset)
            }
        }
        Material::Water => {
            let intensity = 100 + (water_level as u16 * 155 / 255) as u8;
            ("💧", Color::Rgb(dim(40), dim(80), dim(intensity)))
        }
        Material::Soil => {
            if water_level > 50 {
                ("🟤", Color::Rgb(dim(80), dim(70), dim(100)))
            } else if water_level > 0 {
                ("🟫", Color::Rgb(dim(110), dim(80), dim(50)))
            } else {
                ("🟫", Color::Rgb(dim(139), dim(90), dim(43)))
            }
        }
        Material::Stone => ("🪨", Color::Rgb(dim(120), dim(120), dim(120))),
        Material::Root => ("🌿", Color::Rgb(dim(80), dim(180), dim(60))),
        Material::Seed => {
            if nutrient_level >= 100 {
                ("🌱", Color::Rgb(dim(140), dim(200), dim(60)))
            } else {
                ("🌰", Color::Rgb(dim(200), dim(180), dim(60)))
            }
        }
    }
}

fn in_tool_range(x: usize, y: usize, z: usize, app: &App) -> bool {
    if let Some((sx, sy, sz)) = app.tool_start {
        let ex = app.cursor_x;
        let ey = app.cursor_y;
        let ez = app.slice_z;
        x >= sx.min(ex) && x <= sx.max(ex)
            && y >= sy.min(ey) && y <= sy.max(ey)
            && z >= sz.min(ez) && z <= sz.max(ez)
    } else {
        false
    }
}

pub fn draw(frame: &mut Frame, world: &World, app: &App) {
    let area = frame.area();

    // Layout: optional side panels + grid + status bar
    let has_side_panel = app.show_inspect || app.show_status;
    let [main_area, status_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

    let (grid_area, side_area) = if has_side_panel {
        let [g, s] = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(main_area);
        (g, Some(s))
    } else {
        (main_area, None)
    };

    let grid = world.resource::<VoxelGrid>();
    let z = app.slice_z;

    // Render the XY slice at the current Z level.
    let max_rows = grid_area.height as usize;
    let emoji_cols = grid_area.width as usize / 2;

    let mut lines: Vec<Line> = Vec::with_capacity(max_rows.min(GRID_Y));

    for y in 0..GRID_Y.min(max_rows) {
        let mut spans: Vec<Span> = Vec::with_capacity(emoji_cols.min(GRID_X));
        for x in 0..GRID_X.min(emoji_cols) {
            if let Some(voxel) = grid.get(x, y, z) {
                let (s, fg) = voxel_style(voxel.material, voxel.water_level, voxel.light_level, voxel.nutrient_level);

                let is_cursor = x == app.cursor_x && y == app.cursor_y;
                let is_in_range = app.tool_active && in_tool_range(x, y, z, app);

                let style = if is_cursor {
                    Style::default().fg(fg).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if is_in_range {
                    Style::default().fg(fg).bg(Color::Rgb(60, 60, 100))
                } else {
                    Style::default().fg(fg)
                };

                spans.push(Span::styled(s, style));
            }
        }
        lines.push(Line::from(spans));
    }

    let grid_widget = Paragraph::new(lines);
    frame.render_widget(grid_widget, grid_area);

    // Side panel (inspect + status)
    if let Some(side) = side_area {
        let mut panel_lines: Vec<Line> = Vec::new();

        if app.show_inspect {
            panel_lines.push(Line::from(Span::styled(
                "-- Inspect --",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));

            if let Some(voxel) = grid.get(app.cursor_x, app.cursor_y, z) {
                let depth_label = if z > GROUND_LEVEL {
                    format!("above +{}", z - GROUND_LEVEL)
                } else if z == GROUND_LEVEL {
                    "surface".to_string()
                } else {
                    format!("below -{}", GROUND_LEVEL - z)
                };
                panel_lines.push(Line::from(format!(
                    "({}, {}, {}) [{}]",
                    app.cursor_x, app.cursor_y, z, depth_label
                )));
                panel_lines.push(Line::from(format!("material: {}", voxel.material.name())));
                panel_lines.push(Line::from(format!("water: {}/255", voxel.water_level)));
                panel_lines.push(Line::from(format!("light: {}/255", voxel.light_level)));
                panel_lines.push(Line::from(format!("nutrient: {}/255", voxel.nutrient_level)));

                if voxel.material == Material::Seed {
                    let growth = voxel.nutrient_level;
                    let pct = (growth as u16 * 100) / 200;
                    panel_lines.push(Line::from(format!("growth: {}% ({}/200)", pct, growth)));
                    let has_water = voxel.water_level >= 30 || {
                        let dirs: [(isize, isize, isize); 6] = [
                            (-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)
                        ];
                        dirs.iter().any(|&(dx,dy,dz)| {
                            let nx = app.cursor_x as isize + dx;
                            let ny = app.cursor_y as isize + dy;
                            let nz = z as isize + dz;
                            if nx < 0 || ny < 0 || nz < 0 { return false; }
                            grid.get(nx as usize, ny as usize, nz as usize)
                                .map_or(false, |v| v.water_level >= 30)
                        })
                    };
                    let has_light = voxel.light_level >= 30;
                    if has_water && has_light {
                        panel_lines.push(Line::from(Span::styled(
                            "status: growing",
                            Style::default().fg(Color::Green),
                        )));
                    } else {
                        let mut missing = Vec::new();
                        if !has_water { missing.push("water"); }
                        if !has_light { missing.push("light"); }
                        panel_lines.push(Line::from(Span::styled(
                            format!("dormant (need {})", missing.join(", ")),
                            Style::default().fg(Color::Red),
                        )));
                    }
                }
            }
            panel_lines.push(Line::from(""));
        }

        if app.show_status {
            panel_lines.push(Line::from(Span::styled(
                "-- Status --",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
            panel_lines.push(Line::from(format!("Tick: {}", app.tick_count())));

            let mut counts = [0u64; 6];
            let mut wet_soil = 0u64;
            for v in grid.cells() {
                counts[v.material.as_u8() as usize] += 1;
                if v.material == Material::Soil && v.water_level > 50 {
                    wet_soil += 1;
                }
            }
            let names = ["air", "soil", "stone", "water", "root", "seed"];
            for i in 0..6 {
                panel_lines.push(Line::from(format!("  {}: {}", names[i], counts[i])));
            }
            panel_lines.push(Line::from(format!("  wet soil: {}", wet_soil)));
        }

        let panel = Paragraph::new(panel_lines);
        frame.render_widget(panel, side);
    }

    // Status bar (bottom).
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

    let tool_str = if app.tool_active {
        if let Some((sx, sy, sz)) = app.tool_start {
            format!("TOOL:{} ({},{},{})→({},{},{})",
                app.selected_material().name(), sx, sy, sz,
                app.cursor_x, app.cursor_y, app.slice_z)
        } else {
            format!("TOOL:{}", app.selected_material().name())
        }
    } else {
        format!("[{}]", app.selected_material().name())
    };

    let status_line1 = format!(
        " X:{cx} Y:{cy} Z:{z}/{max} ({depth})  Tick:{tick}  [{mode}]  {tool}",
        cx = app.cursor_x,
        cy = app.cursor_y,
        z = z,
        max = groundwork_sim::grid::GRID_Z - 1,
        depth = depth_label,
        tick = app.tick_count(),
        mode = mode,
        tool = tool_str,
    );
    let status_line2 = " [WASD]move [J/K]depth [Tab]material [Enter]tool [I]inspect [T]status [Space]step [P]auto [Q]quit";

    let status = Paragraph::new(vec![
        Line::from(status_line1),
        Line::from(status_line2),
    ]).style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(status, status_area);
}
