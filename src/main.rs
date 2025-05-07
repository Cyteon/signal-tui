use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{style::{Color, Style}, widgets::{Block, Borders, Gauge}, DefaultTerminal, Frame};
use signal::create_cli;
use rusqlite::Connection;
use directories::ProjectDirs;

mod signal;
mod db;
mod types;

use types::{App, AppState};

fn main() -> Result<()> {
    color_eyre::install()?;
    
    let terminal = ratatui::init();
    let result = App::default().run(terminal);
    ratatui::restore();

    result
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
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
    
        if !std::fs::exists(path.join("signal-cli/bin/signal-cli")).unwrap() {
            signal::download_cli(&mut terminal, path.clone()).unwrap();
        }
    
        let mut cli = create_cli(path, "".to_string()).unwrap();
        let stdin = cli.stdin.as_mut().unwrap();
        let stdout = cli.stdout.as_mut().unwrap();
    
        signal::list_accounts(stdin, stdout);
    
    
        loop {
            terminal.draw(|frame| {
                frame.render_widget("hello world", frame.area());
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        crossterm::event::KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    
        Ok(())
    }

    fn start(&mut self) {
        self.state = AppState::Started;
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }

    fn render_download(&mut self, frame: &mut Frame) {
        let gauge = Gauge::default()
            .block(Block::default().title("Downloading signal-cli"))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(self.download_status)
            .label(format!("{:.2}%", self.download_status as f32));
        
        frame.render_widget(gauge, frame.area());
    }
}