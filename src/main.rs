mod app;
mod cli;
mod config;
mod editor;
mod explorer;
mod gradle;
mod process;
mod project;
mod ui;

use app::App;
use cli::CliOptions;
use config::ConfigStore;
use crossterm::event;
use project::discover_project_root;
use ratatui::DefaultTerminal;
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
    let options = match CliOptions::parse() {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    let project_root = match discover_project_root(options.project.clone()) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    let mut store = ConfigStore::load(options.config.clone());
    if let Some(theme) = options.theme {
        store.config.theme = theme;
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &project_root, &store, options.read_only);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut DefaultTerminal,
    project_root: &std::path::Path,
    store: &ConfigStore,
    read_only: bool,
) -> io::Result<()> {
    let mut app = App::new(
        project_root.to_path_buf(),
        store.config.clone(),
        &store.session,
        read_only,
    );

    while !app.should_quit {
        app.handle_background_events();
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(60))? {
            let event = event::read()?;
            app.handle_event(event);
        }
    }

    store.save_session(&app.session_state());
    Ok(())
}
