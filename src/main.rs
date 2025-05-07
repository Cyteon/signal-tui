use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};
use signal::create_cli;
use rusqlite::{Connection, OptionalExtension};
use directories::ProjectDirs;

mod signal;
mod db;
mod types;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let path = ProjectDirs::from(
        "dev", 
        "cyteon", 
        "signal-tui"
    ).map(|proj_dirs| {
        proj_dirs.data_local_dir().to_path_buf()
    }).unwrap();
    std::fs::create_dir_all(&path)?;

    let database = Connection::open(path.join("data.db"))?;
    db::init(&database)?;

    let mut cli = create_cli("".to_string()).unwrap();
    let mut stdin = cli.stdin.as_mut().unwrap();
    let mut stdout = cli.stdout.as_mut().unwrap();

    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}