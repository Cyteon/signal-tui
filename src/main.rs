use color_eyre::Result;
use crossterm::event::{self, Event};
use qrcode::QrCode;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, BorderType, Borders, Gauge, Paragraph}, DefaultTerminal, Frame};
use signal::create_cli;
use rusqlite::Connection;
use directories::ProjectDirs;
use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};

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
    
        let mut cli = create_cli(path.clone(), "".to_string()).unwrap();
        let stdin = cli.stdin.as_mut().unwrap();
        let stdout = cli.stdout.as_mut().unwrap();
    
        //let accounts = signal::list_accounts(stdin, stdout);
        let accounts = vec![
            types::SignalAccount {
                uuid: "1234".to_string(),
                number: "+1234567890".to_string(),
            },
            types::SignalAccount {
                uuid: "5678".to_string(),
                number: "+0987654321".to_string(),
            },
            types::SignalAccount {
                uuid: "9101".to_string(),
                number: "+1122334455".to_string(),
            },
            types::SignalAccount {
                uuid: "1213".to_string(),
                number: "+5566778899".to_string(),
            },
        ];
        let mut selected_number = "";
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
                                let link = signal::link_device(stdin, stdout);
                                let code = QrCode::new(link.clone()).unwrap();
                                
                                let qr = code.render::<image::Luma<u8>>()
                                    .quiet_zone(false)
                                    .max_dimensions(200, 200)
                                    .build();

                                qr.save(path.join("qr.png")).unwrap();
                            
                                let dyn_image = image::io::Reader::open(path.join("qr.png"))?.decode()?;

                                let mut picker = Picker::from_query_stdio().unwrap();
                                let mut image = picker.new_resize_protocol(dyn_image);
                                
                                terminal.clear()?;
                                terminal.flush()?;

                                terminal.draw(|f| {
                                    let centered = {
                                        let area = f.area();
                                        let width = area.width.max(40);
                                        let height = (40).min(area.height.saturating_sub(4));

                                        let h_margin = (area.width - width) / 2;
                                        let v_margin = (area.height - height) / 2;

                                        Rect::new(
                                            h_margin,
                                            v_margin,
                                            width,
                                            height
                                        )
                                    };

                                    let block = Block::default()
                                        .title("Link Device")
                                        .borders(Borders::ALL)
                                        .title_alignment(Alignment::Center)
                                        .border_type(BorderType::Rounded);

                                    f.render_widget(block.clone(), centered);

                                    let inner = block.inner(centered);

                                    let chunks = Layout::default()
                                       .margin(1)
                                        .constraints(
                                            vec![
                                                Constraint::Min(40),
                                                Constraint::Length(1),
                                                Constraint::Length(2),
                                            ]
                                        )
                                        .split(inner);

                                    let instruction = Paragraph::new("Scan QR code with signal.")
                                        .alignment(Alignment::Center);

                                    f.render_stateful_widget(StatefulImage::default(), chunks[0], &mut image);
                                    f.render_widget(instruction, chunks[2]);
                                })?;

                                image.last_encoding_result().unwrap()?;

                                //terminal.flush()?;

                                signal::finish_link(stdin, stdout, link);

                                println!("Device linked successfully!");
                            } else {
                                selected_number = &accounts[index].number;
                                dbg!(selected_number);
                            }
                        }

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