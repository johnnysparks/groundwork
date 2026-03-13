use bevy_ecs::world::World;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use groundwork_sim::grid::{VoxelGrid, GROUND_LEVEL};
use groundwork_sim::soil::SoilGrid;
use groundwork_sim::voxel::Material;

use crate::app::App;

/// Map a voxel to a 2-char ASCII string + foreground color.
/// Colors are dimmed by light level to show underground darkness.
fn voxel_style(mat: Material, water_level: u8, light_level: u8, nutrient_level: u8) -> (&'static str, Color) {
    let dim = |c: u8| -> u8 {
        ((c as u16 * light_level as u16) / 255).max(if light_level > 0 { 20 } else { 0 }) as u8
    };

    match mat {
        Material::Air => {
            if water_level > 0 {
                ("~ ", Color::Rgb(dim(80), dim(140), dim(255)))
            } else if light_level == 0 {
                ("  ", Color::Reset)
            } else {
                (". ", Color::Rgb(dim(60), dim(60), dim(60)))
            }
        }
        Material::Water => {
            let intensity = 100 + (water_level as u16 * 155 / 255) as u8;
            ("~~", Color::Rgb(dim(40), dim(80), dim(intensity)))
        }
        Material::Soil => {
            if water_level > 50 {
                ("%%", Color::Rgb(dim(80), dim(70), dim(100)))
            } else if water_level > 0 {
                ("##", Color::Rgb(dim(110), dim(80), dim(50)))
            } else {
                ("##", Color::Rgb(dim(139), dim(90), dim(43)))
            }
        }
        Material::Stone => ("@@", Color::Rgb(dim(170), dim(170), dim(180))),
        Material::Root => ("**", Color::Rgb(dim(80), dim(180), dim(60))),
        Material::Seed => {
            if nutrient_level >= 100 {
                ("<>", Color::Rgb(dim(140), dim(200), dim(60)))
            } else {
                ("()", Color::Rgb(dim(200), dim(180), dim(60)))
            }
        }
        Material::Trunk => ("||", Color::Rgb(dim(139), dim(90), dim(43))),
        Material::Branch => ("--", Color::Rgb(dim(120), dim(80), dim(40))),
        Material::Leaf => ("&&", Color::Rgb(dim(60), dim(160), dim(40))),
        Material::DeadWood => ("XX", Color::Rgb(dim(100), dim(80), dim(60))),
    }
}

fn bar(value: u8, width: usize) -> String {
    let filled = (value as usize * width) / 255;
    let empty = width - filled;
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

fn in_tool_range(x: usize, y: usize, z: usize, app: &App) -> bool {
    if let Some((sx, sy, sz)) = app.tool_start {
        let ex = app.focus_x;
        let ey = app.focus_y;
        let ez = app.focus_z;
        x >= sx.min(ex) && x <= sx.max(ex)
            && y >= sy.min(ey) && y <= sy.max(ey)
            && z >= sz.min(ez) && z <= sz.max(ez)
    } else {
        false
    }
}

pub fn draw(frame: &mut Frame, world: &World, app: &App) {
    let area = frame.area();

    let panel_width = 30u16;
    let [grid_area, panel_area] =
        Layout::horizontal([Constraint::Min(1), Constraint::Length(panel_width)]).areas(area);

    let grid = world.resource::<VoxelGrid>();
    let z = app.focus_z;

    // --- Grid (2 terminal columns per voxel) ---
    let vp_rows = grid_area.height as usize;
    let vp_cols = grid_area.width as usize / 2;

    let (ox, oy) = app.viewport_origin(vp_cols, vp_rows);

    let mut lines: Vec<Line> = Vec::with_capacity(vp_rows);

    for row in 0..vp_rows {
        let wy = oy + row as isize;
        let mut spans: Vec<Span> = Vec::with_capacity(vp_cols);

        for col in 0..vp_cols {
            let wx = ox + col as isize;
            let is_focus = wx == app.focus_x as isize && wy == app.focus_y as isize;

            if wx < 0 || wy < 0 {
                let style = if is_focus {
                    Style::default().bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(". ", style));
                continue;
            }

            let (wx, wy) = (wx as usize, wy as usize);

            if let Some(voxel) = grid.get(wx, wy, z) {
                let (s, fg) = voxel_style(voxel.material, voxel.water_level, voxel.light_level, voxel.nutrient_level);
                let is_in_range = app.tool_active && in_tool_range(wx, wy, z, app);

                let style = if is_focus {
                    Style::default().fg(fg).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if is_in_range {
                    Style::default().fg(fg).bg(Color::Rgb(60, 60, 100))
                } else {
                    Style::default().fg(fg)
                };

                spans.push(Span::styled(s, style));
            } else {
                let style = if is_focus {
                    Style::default().bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(". ", style));
            }
        }
        lines.push(Line::from(spans));
    }

    let grid_widget = Paragraph::new(lines);
    frame.render_widget(grid_widget, grid_area);

    // --- Side panel (always visible) ---
    let mut panel_lines: Vec<Line> = Vec::new();
    let label = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White);
    let badge = |text: &str| -> Line {
        Line::from(vec![Span::styled(
            format!(" {text} "),
            Style::default().fg(Color::Black).bg(Color::White),
        )])
    };

    // Inspect
    panel_lines.push(badge("INSPECT (I)"));
    if app.show_inspect {
        panel_lines.push(Line::from(""));

        if let Some(voxel) = grid.get(app.focus_x, app.focus_y, z) {
            let depth_label = if z > GROUND_LEVEL {
                format!("above +{}", z - GROUND_LEVEL)
            } else if z == GROUND_LEVEL {
                "surface".to_string()
            } else {
                format!("below -{}", GROUND_LEVEL - z)
            };

            let mat_color = match voxel.material {
                Material::Air => Color::Gray,
                Material::Soil => Color::Rgb(139, 90, 43),
                Material::Stone => Color::Rgb(170, 170, 180),
                Material::Water => Color::Rgb(80, 140, 255),
                Material::Root => Color::Rgb(80, 180, 60),
                Material::Seed => Color::Rgb(200, 180, 60),
                Material::Trunk => Color::Rgb(139, 90, 43),
                Material::Branch => Color::Rgb(120, 80, 40),
                Material::Leaf => Color::Rgb(60, 160, 40),
                Material::DeadWood => Color::Rgb(100, 80, 60),
            };

            panel_lines.push(Line::from(vec![
                Span::styled(" pos  ", label),
                Span::styled(
                    format!("({}, {}, {}) {}", app.focus_x, app.focus_y, z, depth_label),
                    value_style,
                ),
            ]));
            panel_lines.push(Line::from(vec![
                Span::styled(" mat  ", label),
                Span::styled(voxel.material.name().to_string(), Style::default().fg(mat_color)),
            ]));
            panel_lines.push(Line::from(""));
            panel_lines.push(Line::from(vec![
                Span::styled(" water ", label),
                Span::styled(
                    format!("{} ", bar(voxel.water_level, 10)),
                    Style::default().fg(Color::Rgb(80, 140, 255)),
                ),
                Span::styled(format!("{:>3}", voxel.water_level), value_style),
            ]));
            panel_lines.push(Line::from(vec![
                Span::styled(" light ", label),
                Span::styled(
                    format!("{} ", bar(voxel.light_level, 10)),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(format!("{:>3}", voxel.light_level), value_style),
            ]));
            panel_lines.push(Line::from(vec![
                Span::styled(" nutr  ", label),
                Span::styled(
                    format!("{} ", bar(voxel.nutrient_level, 10)),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(format!("{:>3}", voxel.nutrient_level), value_style),
            ]));

            // Soil composition diagnostics
            if voxel.material == Material::Soil {
                let soil_grid = world.resource::<SoilGrid>();
                if let Some(comp) = soil_grid.get(app.focus_x, app.focus_y, z) {
                    panel_lines.push(Line::from(""));
                    panel_lines.push(Line::from(vec![Span::styled(
                        format!(" {} SOIL ", comp.type_name().to_uppercase()),
                        Style::default().fg(Color::Black).bg(Color::Rgb(139, 90, 43)),
                    )]));
                    panel_lines.push(Line::from(vec![
                        Span::styled(" sand ", label),
                        Span::styled(format!("{} {:>3}", bar(comp.sand, 8), comp.sand), Style::default().fg(Color::Rgb(194, 178, 128))),
                    ]));
                    panel_lines.push(Line::from(vec![
                        Span::styled(" clay ", label),
                        Span::styled(format!("{} {:>3}", bar(comp.clay, 8), comp.clay), Style::default().fg(Color::Rgb(180, 120, 80))),
                    ]));
                    panel_lines.push(Line::from(vec![
                        Span::styled(" org  ", label),
                        Span::styled(format!("{} {:>3}", bar(comp.organic, 8), comp.organic), Style::default().fg(Color::Rgb(80, 60, 30))),
                    ]));
                    panel_lines.push(Line::from(vec![
                        Span::styled(" rock ", label),
                        Span::styled(format!("{} {:>3}", bar(comp.rock, 8), comp.rock), Style::default().fg(Color::Rgb(150, 150, 160))),
                    ]));
                    panel_lines.push(Line::from(vec![
                        Span::styled(" bact ", label),
                        Span::styled(format!("{} {:>3}", bar(comp.bacteria, 8), comp.bacteria), Style::default().fg(Color::Rgb(100, 200, 100))),
                    ]));
                    panel_lines.push(Line::from(vec![
                        Span::styled(" pH   ", label),
                        Span::styled(format!("{:.1}", comp.ph_value()), value_style),
                    ]));
                    if comp.is_compacted() {
                        panel_lines.push(Line::from(vec![Span::styled(
                            " COMPACTED",
                            Style::default().fg(Color::Red),
                        )]));
                    }
                }
            }

            // Seed diagnostics
            if voxel.material == Material::Seed {
                panel_lines.push(Line::from(""));
                panel_lines.push(Line::from(vec![Span::styled(
                    " SEED ",
                    Style::default().fg(Color::Black).bg(Color::Rgb(200, 180, 60)),
                )]));

                let growth = voxel.nutrient_level;
                let growth_max = 200u16;
                let pct = (growth as u16 * 100) / growth_max;
                panel_lines.push(Line::from(vec![
                    Span::styled(" growth ", label),
                    Span::styled(format!("{growth}/200 ({pct}%)"), value_style),
                ]));

                let has_water = voxel.water_level >= 30 || {
                    let dirs: [(isize, isize, isize); 6] = [
                        (-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)
                    ];
                    dirs.iter().any(|&(dx,dy,dz)| {
                        let nx = app.focus_x as isize + dx;
                        let ny = app.focus_y as isize + dy;
                        let nz = z as isize + dz;
                        if nx < 0 || ny < 0 || nz < 0 { return false; }
                        grid.get(nx as usize, ny as usize, nz as usize)
                            .map_or(false, |v| v.water_level >= 30)
                    })
                };
                let has_light = voxel.light_level >= 30;

                let water_ind = if has_water {
                    Span::styled(" YES", Style::default().fg(Color::Rgb(80, 140, 255)))
                } else {
                    Span::styled(" NO", Style::default().fg(Color::Red))
                };
                panel_lines.push(Line::from(vec![Span::styled(" water?", label), water_ind]));

                let light_ind = if has_light {
                    Span::styled(" YES", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled(" NO", Style::default().fg(Color::Red))
                };
                panel_lines.push(Line::from(vec![Span::styled(" light?", label), light_ind]));

                if has_water && has_light {
                    let remaining = growth_max.saturating_sub(growth as u16);
                    let ticks_left = (remaining + 4) / 5;
                    panel_lines.push(Line::from(vec![
                        Span::styled(" status ", label),
                        Span::styled(format!("~{ticks_left} ticks"), Style::default().fg(Color::Green)),
                    ]));
                } else {
                    panel_lines.push(Line::from(vec![
                        Span::styled(" status ", label),
                        Span::styled("dormant", Style::default().fg(Color::Red)),
                    ]));
                }
            }
        } else {
            panel_lines.push(Line::from(vec![
                Span::styled(" pos  ", label),
                Span::styled(
                    format!("({}, {}, {}) out of bounds", app.focus_x, app.focus_y, z),
                    Style::default().fg(Color::Red),
                ),
            ]));
        }
    }
    panel_lines.push(Line::from(""));

    // Sim info
    let mode = if app.auto_tick {
        format!("AUTO {}ms", app.tick_rate_ms)
    } else {
        "MANUAL".to_string()
    };

    panel_lines.push(Line::from(vec![
        Span::styled(" tick ", label),
        Span::styled(format!("{}", app.tick_count()), value_style),
        Span::styled("  ", label),
        Span::styled(mode, Style::default().fg(Color::DarkGray)),
    ]));

    // Tool state
    if app.tool_active {
        if let Some((sx, sy, sz)) = app.tool_start {
            panel_lines.push(Line::from(vec![
                Span::styled(" tool ", label),
                Span::styled(
                    format!("{} ({},{},{}) \u{2192} ({},{},{})",
                        app.selected_tool().name(), sx, sy, sz,
                        app.focus_x, app.focus_y, app.focus_z),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }
    }
    panel_lines.push(Line::from(""));

    // Missions
    panel_lines.extend(app.quest_log.render_lines(app.show_missions, panel_width as usize));

    // Status
    panel_lines.push(badge("STATUS (T)"));
    if app.show_status {
        panel_lines.push(Line::from(""));

        let mut counts = [0u64; 10];
        let mut wet_soil = 0u64;
        for v in grid.cells() {
            counts[v.material.as_u8() as usize] += 1;
            if v.material == Material::Soil && v.water_level > 50 {
                wet_soil += 1;
            }
        }

        let mat_entries: [(&str, &str, usize, Color); 10] = [
            ("~~", "water", Material::Water as usize, Color::Rgb(80, 140, 255)),
            ("##", "soil", Material::Soil as usize, Color::Rgb(139, 90, 43)),
            ("@@", "stone", Material::Stone as usize, Color::Rgb(170, 170, 180)),
            ("**", "root", Material::Root as usize, Color::Rgb(80, 180, 60)),
            ("()", "seed", Material::Seed as usize, Color::Rgb(200, 180, 60)),
            ("||", "trunk", Material::Trunk as usize, Color::Rgb(139, 90, 43)),
            ("--", "branch", Material::Branch as usize, Color::Rgb(120, 80, 40)),
            ("&&", "leaf", Material::Leaf as usize, Color::Rgb(60, 160, 40)),
            ("XX", "dead", Material::DeadWood as usize, Color::Rgb(100, 80, 60)),
            (". ", "air", Material::Air as usize, Color::Gray),
        ];

        for (icon, name, idx, color) in mat_entries {
            let count = counts[idx];
            if count > 0 {
                panel_lines.push(Line::from(vec![
                    Span::styled(format!(" {icon} "), Style::default().fg(color)),
                    Span::styled(format!("{:<6}", name), Style::default().fg(color)),
                    Span::styled(format!("{:>6}", count), value_style),
                ]));
            }
        }
        if wet_soil > 0 {
            panel_lines.push(Line::from(vec![
                Span::styled(" %% ", Style::default().fg(Color::Rgb(80, 70, 100))),
                Span::styled(format!("{:<6}", "wet"), Style::default().fg(Color::Rgb(80, 70, 100))),
                Span::styled(format!("{:>6}", wet_soil), value_style),
            ]));
        }
    }
    panel_lines.push(Line::from(""));

    // Tool (was Brush)
    {
        let tool = app.selected_tool();
        let tool_label = if tool == crate::app::Tool::SeedBag {
            format!("{} ({})", tool.name(), app.species_name())
        } else {
            tool.name().to_string()
        };
        panel_lines.push(Line::from(vec![
            Span::styled(" TOOL ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" "),
            Span::styled(tool_label, Style::default().fg(Color::Cyan)),
        ]));
        if tool == crate::app::Tool::SeedBag {
            panel_lines.push(Line::from(vec![
                Span::styled("  [ / ]", Style::default().fg(Color::DarkGray)),
                Span::styled(" change species", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }
    panel_lines.push(Line::from(""));

    // Controls
    panel_lines.push(badge("CONTROLS (H)"));
    if app.show_controls {
        panel_lines.push(Line::from(""));

        let controls: [(&str, &str); 14] = [
            ("WASD", "pan"),
            ("J/K", "depth"),
            ("Space", "use tool"),
            ("Tab", "next tool"),
            ("[ / ]", "species"),
            ("M", "missions"),
            (",/.", "quest nav"),
            ("I", "inspect"),
            ("T", "status"),
            ("H", "controls"),
            ("Shift+P", "step"),
            ("P", "auto-tick"),
            ("+/-", "speed"),
            ("Q", "quit"),
        ];

        for (key, desc) in controls {
            panel_lines.push(Line::from(vec![
                Span::styled(format!(" {key:<8}"), Style::default().fg(Color::Cyan)),
                Span::styled(desc.to_string(), label),
            ]));
        }
    }

    // Legend
    panel_lines.push(Line::from(""));
    panel_lines.push(badge("LEGEND"));
    let legend: [(&str, &str, Color); 10] = [
        (". ", "air", Color::Gray),
        ("~~", "water", Color::Rgb(80, 140, 255)),
        ("##", "soil", Color::Rgb(139, 90, 43)),
        ("@@", "stone", Color::Rgb(170, 170, 180)),
        ("**", "root", Color::Rgb(80, 180, 60)),
        ("()", "seed", Color::Rgb(200, 180, 60)),
        ("||", "trunk", Color::Rgb(139, 90, 43)),
        ("--", "branch", Color::Rgb(120, 80, 40)),
        ("&&", "leaf", Color::Rgb(60, 160, 40)),
        ("XX", "dead", Color::Rgb(100, 80, 60)),
    ];
    for (ch, name, color) in legend {
        panel_lines.push(Line::from(vec![
            Span::styled(format!(" {ch} "), Style::default().fg(color)),
            Span::styled(name.to_string(), label),
        ]));
    }

    let panel_block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));

    let panel = Paragraph::new(panel_lines).block(panel_block);
    frame.render_widget(panel, panel_area);
}
