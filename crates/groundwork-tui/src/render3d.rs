/// Projected 3D terminal renderer.
///
/// Pipeline: world state → camera projection → DDA raycast → shape-aware glyph selection.
/// No intermediate image or framebuffer. Each terminal cell is computed directly from
/// world voxels via orthographic raycasting.

use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use groundwork_sim::voxel::Material;

use crate::app::App;
use crate::camera::{Camera, Vec3};
use crate::glyph::GlyphAtlas;

/// Result of a single sub-ray hitting the voxel grid.
#[derive(Clone, Copy)]
struct RayHit {
    material: Material,
    /// Which face was entered: 0=top/bottom, 1=X-face, 2=Y-face.
    face: u8,
    /// Depth along the ray (for sorting / shading).
    t: f64,
    /// World Z of the hit voxel (for lighting estimation).
    world_z: usize,
    /// Water level at hit voxel.
    water_level: u8,
}

/// Cast a ray through the voxel grid using DDA (Digital Differential Analyzer).
/// Returns the first non-Air voxel hit, or None if the ray exits the grid.
fn raycast(grid: &VoxelGrid, origin: Vec3, dir: Vec3) -> Option<RayHit> {
    let dir = dir.normalized();

    // If direction component is near-zero, treat as very large step
    let inv_dir = Vec3::new(
        if dir.x.abs() < 1e-12 { 1e12 } else { 1.0 / dir.x },
        if dir.y.abs() < 1e-12 { 1e12 } else { 1.0 / dir.y },
        if dir.z.abs() < 1e-12 { 1e12 } else { 1.0 / dir.z },
    );

    // Find entry point into the grid AABB [0, GRID_X] × [0, GRID_Y] × [0, GRID_Z]
    let (t_enter, t_exit) = aabb_intersect(
        origin,
        inv_dir,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(GRID_X as f64, GRID_Y as f64, GRID_Z as f64),
    )?;

    let t_start = t_enter.max(0.0);
    if t_start >= t_exit {
        return None;
    }

    // Start position (just inside the grid)
    let start = origin.add(dir.scale(t_start + 0.001));

    // Current voxel coordinates
    let mut vx = (start.x.floor() as i32).clamp(0, GRID_X as i32 - 1);
    let mut vy = (start.y.floor() as i32).clamp(0, GRID_Y as i32 - 1);
    let mut vz = (start.z.floor() as i32).clamp(0, GRID_Z as i32 - 1);

    // Step direction
    let step_x: i32 = if dir.x >= 0.0 { 1 } else { -1 };
    let step_y: i32 = if dir.y >= 0.0 { 1 } else { -1 };
    let step_z: i32 = if dir.z >= 0.0 { 1 } else { -1 };

    // Distance along the ray to the next voxel boundary in each axis
    let next_boundary = |pos: f64, voxel: i32, step: i32| -> f64 {
        if step > 0 {
            (voxel + 1) as f64 - pos
        } else {
            pos - voxel as f64
        }
    };

    let mut t_max_x = next_boundary(start.x, vx, step_x) * inv_dir.x.abs();
    let mut t_max_y = next_boundary(start.y, vy, step_y) * inv_dir.y.abs();
    let mut t_max_z = next_boundary(start.z, vz, step_z) * inv_dir.z.abs();

    let t_delta_x = inv_dir.x.abs();
    let t_delta_y = inv_dir.y.abs();
    let t_delta_z = inv_dir.z.abs();

    // Track which axis we last stepped along (for face determination)
    let mut last_axis: u8 = 0; // 0=Z, 1=X, 2=Y

    // Maximum iterations to prevent infinite loops
    let max_steps = (GRID_X + GRID_Y + GRID_Z) * 2;

    for _ in 0..max_steps {
        // Check current voxel
        if vx >= 0 && vx < GRID_X as i32 && vy >= 0 && vy < GRID_Y as i32 && vz >= 0 && vz < GRID_Z as i32 {
            if let Some(voxel) = grid.get(vx as usize, vy as usize, vz as usize) {
                if voxel.material != Material::Air {
                    let t = t_start + t_max_x.min(t_max_y).min(t_max_z);
                    return Some(RayHit {
                        material: voxel.material,
                        face: last_axis,
                        t,
                        world_z: vz as usize,
                        water_level: voxel.water_level,
                    });
                }
            }
        } else {
            // Exited grid bounds
            return None;
        }

        // Step to next voxel boundary
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                vx += step_x;
                t_max_x += t_delta_x;
                last_axis = 1;
            } else {
                vz += step_z;
                t_max_z += t_delta_z;
                last_axis = 0;
            }
        } else if t_max_y < t_max_z {
            vy += step_y;
            t_max_y += t_delta_y;
            last_axis = 2;
        } else {
            vz += step_z;
            t_max_z += t_delta_z;
            last_axis = 0;
        }
    }

    None
}

/// Ray-AABB intersection. Returns (t_enter, t_exit) or None if no intersection.
fn aabb_intersect(origin: Vec3, inv_dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<(f64, f64)> {
    let t1x = (aabb_min.x - origin.x) * inv_dir.x;
    let t2x = (aabb_max.x - origin.x) * inv_dir.x;
    let t1y = (aabb_min.y - origin.y) * inv_dir.y;
    let t2y = (aabb_max.y - origin.y) * inv_dir.y;
    let t1z = (aabb_min.z - origin.z) * inv_dir.z;
    let t2z = (aabb_max.z - origin.z) * inv_dir.z;

    let t_enter = t1x.min(t2x).max(t1y.min(t2y)).max(t1z.min(t2z));
    let t_exit = t1x.max(t2x).min(t1y.max(t2y)).min(t1z.max(t2z));

    if t_enter <= t_exit && t_exit >= 0.0 {
        Some((t_enter, t_exit))
    } else {
        None
    }
}

/// Color for a material, with face-based shading.
/// Top faces are brightest, side faces slightly darker.
fn material_color(mat: Material, face: u8, world_z: usize, water_level: u8) -> Color {
    // Brightness factor: top=1.0, front=0.8, side=0.65
    let face_brightness = match face {
        0 => 1.0f32,   // top/bottom face
        1 => 0.75,     // X face
        _ => 0.6,      // Y face
    };

    // Height-based ambient: underground is darker
    let height_factor = if world_z <= GROUND_LEVEL {
        0.4 + 0.6 * (world_z as f32 / GROUND_LEVEL as f32)
    } else {
        1.0
    };

    let brightness = face_brightness * height_factor;
    let dim = |c: u8| -> u8 { (c as f32 * brightness).min(255.0) as u8 };

    match mat {
        Material::Air => Color::Rgb(40, 40, 50),
        Material::Water => {
            let intensity = 100 + (water_level as u16 * 155 / 255) as u8;
            Color::Rgb(dim(40), dim(80), dim(intensity))
        }
        Material::Soil => Color::Rgb(dim(139), dim(90), dim(43)),
        Material::Stone => Color::Rgb(dim(160), dim(160), dim(170)),
        Material::Root => Color::Rgb(dim(70), dim(170), dim(50)),
        Material::Seed => Color::Rgb(dim(200), dim(180), dim(60)),
    }
}

/// Render the 3D projected view into the frame.
pub fn draw_3d(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let panel_width = 30u16;
    let [view_area, panel_area] =
        Layout::horizontal([Constraint::Min(1), Constraint::Length(panel_width)]).areas(area);

    let grid = app.world.resource::<VoxelGrid>();
    let camera = app.camera.as_ref().expect("camera must exist in 3D mode");
    let atlas = GlyphAtlas::new();

    let cols = view_area.width as usize;
    let rows = view_area.height as usize;
    let half_w = cols as f64 / 2.0;
    let half_h = rows as f64 / 2.0;

    let mut lines: Vec<Line> = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut spans: Vec<Span> = Vec::with_capacity(cols);

        for col in 0..cols {
            let col_offset = col as f64 - half_w + 0.5;
            let row_offset = row as f64 - half_h + 0.5;

            // Cast 4 sub-rays (2×2 within the cell) for shape-aware glyph selection.
            // Offsets: ±0.25 in col/row space.
            let sub_offsets: [(f64, f64); 4] = [
                (-0.25, -0.25), // top-left
                (0.25, -0.25),  // top-right
                (-0.25, 0.25),  // bottom-left
                (0.25, 0.25),   // bottom-right
            ];

            let mut coverage = [0.0f32; 4];
            let mut best_hit: Option<RayHit> = None;

            for (i, &(dc, dr)) in sub_offsets.iter().enumerate() {
                let (origin, dir) = camera.ray_for_cell(col_offset + dc, row_offset + dr);
                if let Some(hit) = raycast(grid, origin, dir) {
                    coverage[i] = 1.0;
                    // Keep the closest (shallowest) hit for material/color
                    if best_hit.is_none() || hit.t < best_hit.unwrap().t {
                        best_hit = Some(hit);
                    }
                }
            }

            if let Some(hit) = best_hit {
                // Material-specific glyph override for water
                let ch = if hit.material == Material::Water && coverage == [1.0, 1.0, 1.0, 1.0] {
                    '~'
                } else {
                    atlas.best_match(coverage)
                };

                let fg = material_color(hit.material, hit.face, hit.world_z, hit.water_level);
                let bg = Color::Reset;

                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(fg).bg(bg),
                ));
            } else {
                // Sky / empty — gradient by row
                let sky_intensity = ((1.0 - row as f32 / rows as f32) * 30.0) as u8 + 10;
                spans.push(Span::styled(
                    " ".to_string(),
                    Style::default().bg(Color::Rgb(sky_intensity, sky_intensity, sky_intensity + 15)),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    let view_widget = Paragraph::new(lines);
    frame.render_widget(view_widget, view_area);

    // --- Side panel (reuse structure from 2D view) ---
    draw_3d_panel(frame, app, grid, camera, panel_area);
}

/// Draw the side panel for 3D mode.
fn draw_3d_panel(
    frame: &mut Frame,
    app: &App,
    grid: &VoxelGrid,
    camera: &Camera,
    panel_area: ratatui::layout::Rect,
) {
    let mut panel_lines: Vec<Line> = Vec::new();
    let label = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White);
    let badge = |text: &str| -> Line {
        Line::from(vec![Span::styled(
            format!(" {text} "),
            Style::default().fg(Color::Black).bg(Color::White),
        )])
    };

    // View mode indicator
    panel_lines.push(Line::from(vec![
        Span::styled(" VIEW ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" "),
        Span::styled("3D projected", Style::default().fg(Color::Cyan)),
    ]));
    panel_lines.push(Line::from(""));

    // Camera info
    panel_lines.push(badge("CAMERA"));
    panel_lines.push(Line::from(""));
    panel_lines.push(Line::from(vec![
        Span::styled(" focus ", label),
        Span::styled(
            format!("({:.0}, {:.0}, {:.0})", camera.focus.x, camera.focus.y, camera.focus.z),
            value_style,
        ),
    ]));
    panel_lines.push(Line::from(vec![
        Span::styled(" yaw   ", label),
        Span::styled(format!("{:.0}°", camera.yaw.to_degrees()), value_style),
    ]));
    panel_lines.push(Line::from(vec![
        Span::styled(" pitch ", label),
        Span::styled(format!("{:.0}°", camera.pitch.to_degrees()), value_style),
    ]));
    panel_lines.push(Line::from(vec![
        Span::styled(" zoom  ", label),
        Span::styled(format!("{:.2}", camera.ortho_scale), value_style),
    ]));
    panel_lines.push(Line::from(""));

    // Focus voxel inspect
    let fx = camera.focus.x as usize;
    let fy = camera.focus.y as usize;
    let fz = camera.focus.z as usize;
    panel_lines.push(badge("INSPECT (I)"));
    if app.show_inspect {
        panel_lines.push(Line::from(""));
        if let Some(voxel) = grid.get(fx, fy, fz) {
            let depth_label = if fz > GROUND_LEVEL {
                format!("above +{}", fz - GROUND_LEVEL)
            } else if fz == GROUND_LEVEL {
                "surface".to_string()
            } else {
                format!("below -{}", GROUND_LEVEL - fz)
            };
            panel_lines.push(Line::from(vec![
                Span::styled(" pos  ", label),
                Span::styled(format!("({fx}, {fy}, {fz}) {depth_label}"), value_style),
            ]));

            let mat_color = match voxel.material {
                Material::Air => Color::Gray,
                Material::Soil => Color::Rgb(139, 90, 43),
                Material::Stone => Color::Rgb(170, 170, 180),
                Material::Water => Color::Rgb(80, 140, 255),
                Material::Root => Color::Rgb(80, 180, 60),
                Material::Seed => Color::Rgb(200, 180, 60),
            };
            panel_lines.push(Line::from(vec![
                Span::styled(" mat  ", label),
                Span::styled(voxel.material.name().to_string(), Style::default().fg(mat_color)),
            ]));
        } else {
            panel_lines.push(Line::from(vec![
                Span::styled(" pos  ", label),
                Span::styled("out of bounds", Style::default().fg(Color::Red)),
            ]));
        }
    }
    panel_lines.push(Line::from(""));

    // Tick
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

    // Tool
    panel_lines.push(Line::from(""));
    panel_lines.push(Line::from(vec![
        Span::styled(" TOOL ", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" "),
        Span::styled(app.selected_tool().name().to_string(), Style::default().fg(Color::Cyan)),
    ]));
    panel_lines.push(Line::from(""));

    // Controls
    panel_lines.push(badge("CONTROLS (H)"));
    if app.show_controls {
        panel_lines.push(Line::from(""));

        let controls: [(&str, &str); 12] = [
            ("W/S", "fly fwd/back"),
            ("A/D", "pan left/right"),
            ("Q/E", "orbit left/right"),
            ("Sh+W/S", "zoom in/out"),
            ("J/K", "focus up/down"),
            ("Space", "use tool"),
            ("Tab", "next tool"),
            ("V", "toggle 2D/3D"),
            ("I", "inspect"),
            ("T", "status"),
            ("P", "auto-tick"),
            ("Esc", "quit"),
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
    let legend: [(&str, &str, Color); 6] = [
        (" ", "air/sky", Color::Gray),
        ("~", "water", Color::Rgb(80, 140, 255)),
        ("#", "soil", Color::Rgb(139, 90, 43)),
        ("@", "stone", Color::Rgb(170, 170, 180)),
        ("*", "root", Color::Rgb(80, 180, 60)),
        ("*", "seed", Color::Rgb(200, 180, 60)),
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
