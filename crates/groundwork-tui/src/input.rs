use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::app::{App, ViewMode};
use crate::quest::Action;

pub fn handle_event(app: &mut App) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        // View mode toggle (available in both modes)
        if key.code == KeyCode::Char('v') || key.code == KeyCode::Char('V') {
            app.toggle_view_mode();
            if app.view_mode == ViewMode::Projected3D {
                app.quest_log.record(Action::SwitchTo3D);
            }
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
        KeyCode::Left | KeyCode::Char('a') => {
            app.pan(-1, 0);
            app.quest_log.record(Action::Pan);
        }
        KeyCode::Right | KeyCode::Char('d') => {
            app.pan(1, 0);
            app.quest_log.record(Action::Pan);
        }
        KeyCode::Up | KeyCode::Char('w') => {
            app.pan(0, -1);
            app.quest_log.record(Action::Pan);
        }
        KeyCode::Down | KeyCode::Char('s') => {
            app.pan(0, 1);
            app.quest_log.record(Action::Pan);
        }

        // Z navigation (depth).
        KeyCode::Char('k') => {
            app.move_z(1);
            app.quest_log.record(Action::ChangeDepth);
        }
        KeyCode::Char('j') => {
            app.move_z(-1);
            app.quest_log.record(Action::ChangeDepth);
        }

        // Manual tick (Shift+P).
        KeyCode::Char('P') => {
            app.step();
            app.quest_log.record(Action::StepManually);
        }

        // Toggle auto-tick.
        KeyCode::Char('p') => {
            app.auto_tick = !app.auto_tick;
            app.quest_log.record(Action::ToggleAutoTick);
        }

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

        // Species selection (when seed bag is active).
        KeyCode::Char(']') => {
            app.cycle_species(true);
            app.quest_log.record(Action::CycleSpecies);
        }
        KeyCode::Char('[') => {
            app.cycle_species(false);
            app.quest_log.record(Action::CycleSpecies);
        }

        // Tool start/end (Space).
        KeyCode::Char(' ') => {
            if app.tool_active {
                app.tool_end();
            } else {
                app.tool_start_op();
            }
        }

        // Toggle panel sections.
        KeyCode::Char('i') => {
            app.show_inspect = !app.show_inspect;
            app.quest_log.record(Action::ToggleInspect);
        }
        KeyCode::Char('t') => app.show_status = !app.show_status,
        KeyCode::Char('h') => app.show_controls = !app.show_controls,
        KeyCode::Char('m') => app.show_missions = !app.show_missions,

        // Quest navigation (when missions panel is open).
        KeyCode::Char('.') => app.quest_log.select_next(),
        KeyCode::Char(',') => app.quest_log.select_prev(),

        _ => {}
    }
}

/// Input handling for the 3D projected view.
fn handle_3d(app: &mut App, code: KeyCode, shift: bool) {
    match code {
        KeyCode::Esc => {
            if app.tool_active {
                app.tool_cancel();
            } else {
                app.running = false;
            }
        }

        // WASD: fly/pan camera (Shift+W/S = zoom)
        KeyCode::Char('W') if shift => {
            if let Some(ref mut cam) = app.camera {
                cam.zoom(0.85);
            }
        }
        KeyCode::Char('S') if shift => {
            if let Some(ref mut cam) = app.camera {
                cam.zoom(1.18);
            }
        }
        KeyCode::Char('w') => {
            if let Some(ref mut cam) = app.camera {
                cam.fly_forward(2.0);
            }
            app.quest_log.record(Action::Pan);
        }
        KeyCode::Char('s') => {
            if let Some(ref mut cam) = app.camera {
                cam.fly_forward(-2.0);
            }
            app.quest_log.record(Action::Pan);
        }
        KeyCode::Char('a') => {
            if let Some(ref mut cam) = app.camera {
                cam.pan_right(-2.0);
            }
            app.quest_log.record(Action::Pan);
        }
        KeyCode::Char('d') => {
            if let Some(ref mut cam) = app.camera {
                cam.pan_right(2.0);
            }
            app.quest_log.record(Action::Pan);
        }

        // Q/E: orbit around focus
        KeyCode::Char('q') => {
            if let Some(ref mut cam) = app.camera {
                cam.orbit(-0.15);
            }
        }
        KeyCode::Char('e') => {
            if let Some(ref mut cam) = app.camera {
                cam.orbit(0.15);
            }
        }

        // Z navigation for focus point
        KeyCode::Char('k') => {
            if let Some(ref mut cam) = app.camera {
                cam.move_focus_z(1.0);
            }
            app.quest_log.record(Action::ChangeDepth);
        }
        KeyCode::Char('j') => {
            if let Some(ref mut cam) = app.camera {
                cam.move_focus_z(-1.0);
            }
            app.quest_log.record(Action::ChangeDepth);
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
        KeyCode::Char('P') => {
            app.step();
            app.quest_log.record(Action::StepManually);
        }
        KeyCode::Char('p') => {
            app.auto_tick = !app.auto_tick;
            app.quest_log.record(Action::ToggleAutoTick);
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.tick_rate_ms = app.tick_rate_ms.saturating_sub(50).max(50);
        }
        KeyCode::Char('-') => {
            app.tick_rate_ms = (app.tick_rate_ms + 50).min(2000);
        }

        // Tool selection
        KeyCode::Tab => app.cycle_tool(true),
        KeyCode::BackTab => app.cycle_tool(false),

        // Species selection
        KeyCode::Char(']') => {
            app.cycle_species(true);
            app.quest_log.record(Action::CycleSpecies);
        }
        KeyCode::Char('[') => {
            app.cycle_species(false);
            app.quest_log.record(Action::CycleSpecies);
        }

        // Tool use
        KeyCode::Char(' ') => {
            if app.tool_active {
                app.tool_end();
            } else {
                app.tool_start_op();
            }
        }

        // Panel toggles
        KeyCode::Char('i') => {
            app.show_inspect = !app.show_inspect;
            app.quest_log.record(Action::ToggleInspect);
        }
        KeyCode::Char('t') => app.show_status = !app.show_status,
        KeyCode::Char('h') => app.show_controls = !app.show_controls,
        KeyCode::Char('m') => app.show_missions = !app.show_missions,

        // Quest navigation
        KeyCode::Char('.') => app.quest_log.select_next(),
        KeyCode::Char(',') => app.quest_log.select_prev(),

        _ => {}
    }
}
