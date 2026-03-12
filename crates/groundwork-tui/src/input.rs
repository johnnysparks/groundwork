use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::App;

pub fn handle_event(app: &mut App) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
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
    Ok(())
}
