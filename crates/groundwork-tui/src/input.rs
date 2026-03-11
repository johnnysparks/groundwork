use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::App;
use groundwork_sim::grid::GRID_Z;

pub fn handle_event(app: &mut App) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.running = false,

            // Navigate Z slices.
            KeyCode::Up | KeyCode::Char('k') => {
                if app.slice_z < GRID_Z - 1 {
                    app.slice_z += 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.slice_z > 0 {
                    app.slice_z -= 1;
                }
            }

            // Manual tick.
            KeyCode::Char(' ') => app.step(),

            // Toggle auto-tick.
            KeyCode::Char('p') => app.auto_tick = !app.auto_tick,

            // Adjust tick speed.
            KeyCode::Char('+') | KeyCode::Char('=') => {
                app.tick_rate_ms = app.tick_rate_ms.saturating_sub(50).max(50);
            }
            KeyCode::Char('-') => {
                app.tick_rate_ms = (app.tick_rate_ms + 50).min(2000);
            }

            _ => {}
        }
    }
    Ok(())
}
