mod app;
mod cli;
mod input;
mod render;

use app::App;
use std::io::IsTerminal;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let rest = if args.len() > 2 { &args[2..] } else { &[] };

    match args.get(1).map(|s| s.as_str()) {
        Some("new") => cli::cmd_new(rest),
        Some("tick") => cli::cmd_tick(rest),
        Some("view") => cli::cmd_view(rest),
        Some("place") => cli::cmd_place(rest),
        Some("fill") => cli::cmd_fill(rest),
        Some("inspect") => cli::cmd_inspect(rest),
        Some("status") => cli::cmd_status(rest),
        Some("focus") => cli::cmd_focus(rest),
        Some("tool-start") => cli::cmd_tool_start(rest),
        Some("tool-end") => cli::cmd_tool_end(rest),
        Some("help") | Some("--help") | Some("-h") => {
            cli::print_help();
            Ok(())
        }
        Some("tui") | None => {
            if !std::io::stdout().is_terminal() {
                cli::print_help();
                return Ok(());
            }
            let mut terminal = ratatui::init();
            let result = App::new().run(&mut terminal);
            ratatui::restore();
            result
        }
        Some(other) => {
            eprintln!("Unknown command: {other}");
            cli::print_help();
            std::process::exit(1);
        }
    }
}
