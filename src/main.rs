use std::io::Write;

use color_eyre::Result;
use crossterm::event::{self, Event};
use qrcode::QrCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, 
    style::{Color, Style}, 
    text::Text, widgets::{Block, BorderType, Borders, Paragraph}, 
    DefaultTerminal
};
use signal::create_cli;
use rusqlite::Connection;
use directories::ProjectDirs;

mod signal;
mod db;
mod types;
mod app;

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

    let _ = std::fs::remove_file(path.join("debug.log"));

    let database = Connection::open(path.join("data.db"))?;
    db::init(&database)?;

    if !std::fs::exists(path.join("signal-cli/bin/signal-cli")).unwrap() {
        signal::download_cli(&mut terminal, path.clone()).unwrap();
    }

    let mut cli = create_cli(path.clone(), "".to_string()).unwrap();
    let stdin = std::sync::Arc::new(std::sync::Mutex::new(cli.stdin.take().unwrap()));
    let stdout: std::sync::Arc<std::sync::Mutex<std::process::ChildStdout>> = std::sync::Arc::new(std::sync::Mutex::new(cli.stdout.take().unwrap()));

    let mut accounts = signal::list_accounts(&mut *stdin.lock().unwrap(), &mut *stdout.lock().unwrap());
    let mut selected_number: String = "".to_string();
    let mut index = 0;

    loop {
        terminal.draw(|frame| {
            let centered = {
                let r = frame.area();
                let width = r.width.min(60).max(20);
                let height = (accounts.len() as u16 + 5).min(r.height.max(10));

                let h_margin = (r.width - width) / 2;
                let v_margin = (r.height - height) / 2;

                Rect::new(
                    h_margin,
                    v_margin,
                    width,
                    height
                )
            };

            let block = Block::default()
                .title("Select Account")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            frame.render_widget(block.clone(), centered);

            let inner = block.inner(centered);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    vec![
                        Constraint::Length(1); accounts.len() + 1
                    ]
                )
                .split(inner);

            for (i, account) in accounts.iter().enumerate() {
                let text = if i == index {
                    format!("> {} <", account.number)
                } else {
                    format!(" {} ", account.number)
                };

                let style = if i == index {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                };

                let paragraph = Paragraph::new(text)
                    .style(style)
                    .alignment(Alignment::Center);
            
                frame.render_widget(paragraph, chunks[i]);
            }

            let text = if index == accounts.len() {
                format!("> Link Device <")
            } else {
                format!(" Link Device ")
            };

            let style = if index == accounts.len() {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };

            let paragraph = Paragraph::new(text)
                .style(style)
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, chunks[accounts.len()]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Esc => break,

                    crossterm::event::KeyCode::Up => {
                        if index > 0 {
                            index -= 1;
                        } else {
                            index = accounts.len();
                        }
                    }

                    crossterm::event::KeyCode::Down => {
                        if index < accounts.len() {
                            index += 1;
                        } else {
                            index = 0;
                        }
                    }

                    crossterm::event::KeyCode::Enter => {
                        if index == accounts.len() {
                            let link = signal::link_device(&mut *stdin.lock().unwrap(), &mut *stdout.lock().unwrap());
                            let code = QrCode::new(link.clone()).unwrap();
                            
                            let qr = code.render::<image::Luma<u8>>()
                                .quiet_zone(true)
                                .module_dimensions(1,1)
                                .build();

                            let mut out = String::new();
                            let width = qr.width() as usize;
                            let height = qr.height() as usize;
                        
                            for y in (0..height).step_by(2) {
                                for x in 0..width {
                                    let top = qr.get_pixel(x as u32, y as u32)[0] < 128;
                                    let bottom = if y + 1 < height {
                                        qr.get_pixel(x as u32, (y + 1) as u32)[0] < 128
                                    } else {
                                        false
                                    };
                                    let ch = match (top, bottom) {
                                        (true, true) => '█',
                                        (true, false) => '▀',
                                        (false, true) => '▄',
                                        (false, false) => ' ',
                                    };
                                    out.push(ch);
                                }
                                out.push('\n');
                            }
                                                        
                            terminal.clear()?;
                            terminal.flush()?;

                            let (tx, rx) = std::sync::mpsc::channel();
                            let stdin_clone = stdin.clone();
                            let stdout_clone = stdout.clone();

                            let cloned = link.clone();

                            std::thread::spawn(move || {
                                signal::finish_link(&mut *stdin_clone.lock().unwrap(), &mut *stdout_clone.lock().unwrap(), cloned);
                                tx.send(true).unwrap();
                            });

                            loop {
                                terminal.draw(|f| {
                                    let centered = {
                                        let qr_width = 49u16;
                                        let qr_height = (49u16) / 2;

                                        let area = f.area();
                                        let h_margin = (area.width.saturating_sub(qr_width + 4)) / 2;  // +4 for block borders/margin
                                        let v_margin = (area.height.saturating_sub(qr_height + 4)) / 2;
                                    
                                        Rect::new(
                                            h_margin,
                                            v_margin,
                                            qr_width + 4,
                                            qr_height + 4
                                        )
                                    };

                                    let block = Block::default()
                                        .title("Link Device - Scan QR Code with Signal")
                                        .title_alignment(Alignment::Center);

                                    f.render_widget(block.clone(), centered);

                                    let inner = block.inner(centered);

                                    let chunks = Layout::default()
                                        .constraints(
                                            vec![
                                                Constraint::Length(1),
                                                Constraint::Min(10),
                                            ]
                                        )
                                        .split(inner);

                                    f.render_widget(
                                        Text::from(out.clone()),
                                        chunks[1]
                                    );

                                    f.render_widget(
                                        Text::from("Not working? Press 'o' to open image in browser."),
                                        chunks[0]
                                    )
                                })?;

                                if event::poll(std::time::Duration::from_millis(100))? {
                                    if let Event::Key(key) = event::read()? {
                                        if key.code == crossterm::event::KeyCode::Esc {
                                            break;
                                        }

                                        if key.code == crossterm::event::KeyCode::Char('o') {
                                            webbrowser::open(
                                                format!(
                                                    "https://api.qrserver.com/v1/create-qr-code/?size=500x500&data={}",
                                                    urlencoding::encode(
                                                        &link
                                                    )
                                                ).as_str()
                                            ).unwrap();
                                        }
                                    }
                                }

                                if rx.try_recv().is_ok() {
                                    accounts = signal::list_accounts(&mut *stdin.lock().unwrap(), &mut *stdout.lock().unwrap());
                                    break;
                                }

                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                        } else {
                            selected_number = accounts[index].number.clone();

                            cli.kill().unwrap();
                            cli = create_cli(path.clone(), format!(
                                "-a {}", selected_number
                            )).unwrap();

                            *stdin.lock().unwrap() = cli.stdin.take().unwrap();
                            *stdout.lock().unwrap() = cli.stdout.take().unwrap();

                            // all stff before was just starting the app, now we do the actual app in another file
                            app::app(
                                &mut terminal, 
                                &mut *stdin.lock().unwrap(), 
                                stdout,
                                accounts[index].number.clone(),
                            ).unwrap();

                            break;
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}

pub fn debug_to_file(
    content: String,
) {
    use std::io::Write;

    let path = ProjectDirs::from(
        "dev", 
        "cyteon", 
        "signal-tui"
    ).map(|proj_dirs| {
        proj_dirs.data_local_dir().to_path_buf()
    }).unwrap();

    std::fs::create_dir_all(&path).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.join("debug.log"))
        .unwrap();

    file.write_all((content + "\n").as_bytes()).unwrap();
}