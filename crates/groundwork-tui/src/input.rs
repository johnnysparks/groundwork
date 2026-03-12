use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::app::{App, ViewMode};

pub fn handle_event(app: &mut App) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        // View mode toggle (available in both modes)
        if key.code == KeyCode::Char('v') || key.code == KeyCode::Char('V') {
            app.toggle_view_mode();
            return Ok(());
        }

        match app.view_mode {
            ViewMode::Slice2D => handle_2d(app, key.code, shift),
            ViewMode::Projected3D => handle_3d(app, key.code, shift),
        }
    }
    Ok(())
}

/// Input handling for the classic 2D slice view.
fn handle_2d(app: &mut App, code: KeyCode, _shift: bool) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.tool_active {
                app.tool_cancel();
            } else {
                app.running = false;
            }
        }

        // Pan viewport (arrow keys / WASD). Focus stays at screen center.
        KeyCode::Left | KeyCode::Char('a') => app.pan(-1, 0),
        KeyCode::Right | KeyCode::Char('d') => app.pan(1, 0),
        KeyCode::Up | KeyCode::Char('w') => app.pan(0, -1),
        KeyCode::Down | KeyCode::Char('s') => app.pan(0, 1),

        // Z navigation (depth).
        KeyCode::Char('k') => app.move_z(1),
        KeyCode::Char('j') => app.move_z(-1),

        // Manual tick (Shift+P).
        KeyCode::Char('P') => app.step(),

        // Toggle auto-tick.
        KeyCode::Char('p') => app.auto_tick = !app.auto_tick,

        // Adjust tick speed.
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.tick_rate_ms = app.tick_rate_ms.saturating_sub(50).max(50);
        }
        KeyCode::Char('-') => {
            app.tick_rate_ms = (app.tick_rate_ms + 50).min(2000);
        }

        // Tool selection.
        KeyCode::Tab => app.cycle_tool(true),
        KeyCode::BackTab => app.cycle_tool(false),

        // Tool start/end (Space).
        KeyCode::Char(' ') => {
            if app.tool_active {
                app.tool_end();
            } else {
                app.tool_start_op();
            }
        }

        // Toggle panel sections.
        KeyCode::Char('i') => app.show_inspect = !app.show_inspect,
        KeyCode::Char('t') => app.show_status = !app.show_status,
        KeyCode::Char('h') => app.show_controls = !app.show_controls,

        _ => {}
    }
}

/// Input handling for the 3D projected view.
fn handle_3d(app: &mut App, code: KeyCode, shift: bool) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.tool_active {
                app.tool_cancel();
            } else {
                app.running = false;
            }
        }

        // WASD: fly/pan camera (Shift+W/S = zoom)
        KeyCode::Char('W') if shift => {
            // Zoom in
            if let Some(ref mut cam) = app.camera {
                cam.zoom(0.85);
            }
        }
        KeyCode::Char('S') if shift => {
            // Zoom out
            if let Some(ref mut cam) = app.camera {
                cam.zoom(1.18);
            }
        }
        KeyCode::Char('w') => {
            // Fly forward
            if let Some(ref mut cam) = app.camera {
                cam.fly_forward(2.0);
            }
        }
        KeyCode::Char('s') => {
            // Fly backward
            if let Some(ref mut cam) = app.camera {
                cam.fly_forward(-2.0);
            }
        }
        KeyCode::Char('a') => {
            // Pan left
            if let Some(ref mut cam) = app.camera {
                cam.pan_right(-2.0);
            }
        }
        KeyCode::Char('d') => {
            // Pan right
            if let Some(ref mut cam) = app.camera {
                cam.pan_right(2.0);
            }
        }

        // Q/E: orbit around focus. Note: 'q' without shift is quit (handled above).
        // We use 'e' for orbit right. For orbit left, we need a different key
        // since 'q' is quit. Use '[' and ']' as alternate, or handle 'Q' (shift+q).
        KeyCode::Char('Q') if shift => {
            // Orbit left (Shift+Q to distinguish from quit)
            if let Some(ref mut cam) = app.camera {
                cam.orbit(-0.15);
            }
        }
        KeyCode::Char('e') => {
            // Orbit right
            if let Some(ref mut cam) = app.camera {
                cam.orbit(0.15);
            }
        }

        // Z navigation for focus point
        KeyCode::Char('k') => {
            if let Some(ref mut cam) = app.camera {
                cam.move_focus_z(1.0);
            }
        }
        KeyCode::Char('j') => {
            if let Some(ref mut cam) = app.camera {
                cam.move_focus_z(-1.0);
            }
        }

        // R: reset camera to default orientation
        KeyCode::Char('r') => {
            if let Some(ref mut cam) = app.camera {
                cam.yaw = -std::f64::consts::PI / 4.0;
                cam.pitch = std::f64::consts::PI / 5.0;
                cam.ortho_scale = 0.6;
            }
        }

        // Simulation controls (same as 2D)
        KeyCode::Char('P') => app.step(),
        KeyCode::Char('p') => app.auto_tick = !app.auto_tick,
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.tick_rate_ms = app.tick_rate_ms.saturating_sub(50).max(50);
        }
        KeyCode::Char('-') => {
            app.tick_rate_ms = (app.tick_rate_ms + 50).min(2000);
        }

        // Tool selection
        KeyCode::Tab => app.cycle_tool(true),
        KeyCode::BackTab => app.cycle_tool(false),

        // Tool use
        KeyCode::Char(' ') => {
            if app.tool_active {
                app.tool_end();
            } else {
                app.tool_start_op();
            }
        }

        // Panel toggles
        KeyCode::Char('i') => app.show_inspect = !app.show_inspect,
        KeyCode::Char('t') => app.show_status = !app.show_status,
        KeyCode::Char('h') => app.show_controls = !app.show_controls,

        _ => {}
    }
}
