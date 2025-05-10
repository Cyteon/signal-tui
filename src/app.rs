use std::{collections::HashMap, process::ChildStdout, sync::RwLock, thread};

use crossterm::{event::{self, Event}};
use color_eyre::Result;
use directories::ProjectDirs;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, BorderType, Borders, Padding, Paragraph}, DefaultTerminal
};
use rusqlite::Connection;

use crate::{signal, types};

pub fn app(
    terminal: &mut DefaultTerminal,
    stdin: &mut std::process::ChildStdin, 
    stdout: std::sync::Arc<std::sync::Mutex<ChildStdout>>
) -> Result<()> {
    let mut stdout = stdout;

    let path = ProjectDirs::from(
        "dev", 
        "cyteon", 
        "signal-tui"
    ).map(|proj_dirs| {
        proj_dirs.data_local_dir().to_path_buf()
    }).unwrap();
    std::fs::create_dir_all(&path)?;

    terminal.draw(|f| {
        let centered = {
            let area = f.area();
            let h_margin = (area.width - 20) / 2;
            let v_margin = (area.height - 3) / 2;

            Rect::new(
                h_margin,
                v_margin,
                20,
                3
            )
        };

        f.render_widget("Syncronizing...", centered);
    })?;

    let (groups, contacts) = signal::sync(
        stdin,
        &mut *stdout.lock().unwrap(),
    );

    let mut selected_index = 0;
    let mut show_groups = true;
    let mut show_contacts = true;

    let mut group_index = 0;
    let mut contact_index = 0;

    let mut index_group_map: HashMap<usize, &types::SignalGroup> = HashMap::new();
    let mut index_contact_map: HashMap<usize, &types::SignalContact> = HashMap::new();

    let mut location_selected: bool = false;
    let mut chatting = false;

    // 0 = group, 1 = contact
    let mut selected_type: usize = 0;

    let mut message_index: usize = 0;

    signal::subscribe_receive(stdin);

    let path_clone = path.clone();
    thread::spawn({
        move || {
            let database: &rusqlite::Connection = &Connection::open(path_clone.join("data.db")).unwrap();

            loop {
                signal::read_msg_event(&mut *stdout.lock().unwrap(), database);
            }
        }
    });

    // (source name, message), cant be bothered to impl a better way rn :sob:
    let mut messages: Vec<(String, String)> = vec![];

    let mut scroll_offset: usize = 0;

    loop {
        terminal.draw(|f| {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(30),
                    Constraint::Fill(1)
                ].as_ref())
                .split(f.area());

            let contacts_block = Block::default()
                .borders(Borders::ALL).border_type(BorderType::Rounded);

            let esc_action = if chatting {
                "stop typing"
            } else if location_selected {
                "back"
            } else {
                "exit"
            };
            
            let chat_block = Block::default()
                .borders(Borders::ALL).border_type(BorderType::Rounded)
                .title(format!(" 'esc' - {} | up/down - navigate | 'enter' - reply | 'e' - chat ", esc_action))
                .title_alignment(ratatui::layout::Alignment::Center)
                .padding(Padding::horizontal(1));

            f.render_widget(contacts_block.clone(), h_chunks[0]);
            f.render_widget(chat_block.clone(), h_chunks[1]);

            let contacts_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(1); groups.len() + contacts.len() + 2 // two extra for "Groups" and "People"
                ])
                .split(contacts_block.inner(h_chunks[0]));

            let mut index = 0;

            let group_title_style = if selected_index == index {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };

            if show_groups {
                let title = Paragraph::new(" ▼ Groups")
                    .style(group_title_style);

                f.render_widget(
                    title, contacts_layout[0]
                );

                group_index = index;
                index += 1;
    
                for group in groups.iter() {
                    index_group_map.insert(index, group);

                    let name: &String = &group.name;
                    let layout = contacts_layout[index];
    
                    let style = if index == selected_index {
                        Style::default().bg(Color::Blue)
                    } else {
                        Style::default()
                    };
                    
                    let p = Paragraph::new(
                        format!(" - {}", name)
                    ).style(style);
    
                    f.render_widget(p, layout);
                    
                    index += 1;
                }
            } else {
                let title = Paragraph::new(" ► Groups")
                    .style(group_title_style);

                f.render_widget(
                    title, contacts_layout[index]
                );

                index += 1;
            }

            let contact_title_style = if selected_index == index {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };

            if show_contacts {
                let title = Paragraph::new(" ▼ People")
                    .style(contact_title_style);

                f.render_widget(
                    title, contacts_layout[index]
                );
                
                contact_index = index;
                index += 1;
    
                for contact in contacts.iter() {
                    index_contact_map.insert(index, contact);

                    let mut name: String = String::new();
    
                    if let Some(profile) = &contact.profile {
                        name = profile.given_name.clone().unwrap_or_default();
                    }
    
                    if name.is_empty() {
                        if !contact.name.is_empty() {
                            name = contact.name.clone();
                        } else if contact.given_name.is_some() {
                            name = contact.given_name.clone().unwrap_or_default();
                        } else if !contact.number.is_some() {
                            name = contact.number.clone().unwrap_or_default();
                        } else {
                            name = "Unnamed".to_string();
                        }
                    }
    
                    let layout = contacts_layout[index];
    
                    let style = if index == selected_index {
                        Style::default().bg(Color::Blue)
                    } else {
                        Style::default()
                    };
                    
                    let p = Paragraph::new(
                        format!(" - {}", name)
                    ).style(style);
    
                    f.render_widget(p, layout);
                    
                    index += 1;
                }
            } else {
                let title = Paragraph::new(" ► People")
                    .style(contact_title_style);

                f.render_widget(
                    title, contacts_layout[index]
                );
            }

            if location_selected {
                let db = Connection::open(path.join("data.db")).unwrap();

                let messages_last_time = messages.len();
                messages = vec![];

                if selected_type == 0 {
                    let group_id = index_group_map.get(&selected_index).unwrap().id.clone();

                    let mut query = db.prepare("SELECT sourceName, message FROM messages WHERE groupId = ?").unwrap();
                    let mut rows = query.query(&[&group_id]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        let source_name: String = row.get(0).unwrap();
                        let message: String = row.get(1).unwrap();

                        messages.push((source_name, message));
                    }
                } else {
                    let contact_uuid = index_contact_map.get(&selected_index).unwrap().uuid.clone();

                    let mut query = db.prepare("SELECT sourceName, message FROM messages WHERE destinationUuid = ? OR (sourceUuid = ? AND destinationUuid = 'self')").unwrap();
                    let mut rows = query.query(&[&contact_uuid, &contact_uuid]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        let source_name: String = row.get(0).unwrap();
                        let message: String = row.get(1).unwrap();

                        messages.push((source_name, message));
                    }
                }
                                
                let chat_height = chat_block.inner(h_chunks[1]).height as usize;                
                let visible_count = chat_height.min(messages.len());

                if message_index < scroll_offset {
                    scroll_offset = message_index;
                } else if message_index >= scroll_offset + visible_count {
                    scroll_offset = message_index + 1 - visible_count;
                }

                if messages_last_time == 0 {
                    message_index = messages.len().saturating_sub(1);
                    scroll_offset = messages.len().saturating_sub(visible_count);
                }

                let chat_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(1); visible_count + 1
                    ])
                    .split(chat_block.inner(h_chunks[1]));


                for (i, message) in messages.iter().enumerate().skip(scroll_offset).take(visible_count) {
                    let layout = chat_layout[i - scroll_offset];

                    let style = if i == message_index {
                        Style::default().bg(Color::Blue)
                    } else {
                        Style::default()
                    };

                    let p = Paragraph::new(
                        format!("{}: {}", message.0, message.1)
                    ).style(style);

                    f.render_widget(p, layout);
                }

                if messages.len() == 0 {
                    let layout = chat_layout[0];

                    let p = Paragraph::new(
                        "No messages :("
                    ).style(Style::default());

                    f.render_widget(p, layout);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Esc => {
                        if chatting {
                            chatting = false;
                        } else if location_selected {
                            location_selected = false;
                            chatting = false;
                            message_index = 0;  
                        } else {
                            break;
                        }
                    },

                    crossterm::event::KeyCode::Up => {
                        if location_selected {
                            if message_index > 0 {
                                message_index -= 1;
                            }
                        } else if selected_index > 0 {
                            selected_index -= 1;
                        }
                    }

                    crossterm::event::KeyCode::Down => {
                        if location_selected {
                            if message_index < messages.len().saturating_sub(1) {
                                message_index += 1;
                            }
                        } else if selected_index < groups.len() + contacts.len() + 1 {
                            selected_index += 1;
                        }
                    }

                    crossterm::event::KeyCode::Enter => {
                        messages = vec![];

                        if selected_index == group_index {
                            show_groups = !show_groups;
                        } else if selected_index == contact_index {
                            show_contacts = !show_contacts;
                            
                        } else {
                            location_selected = true;
                            chatting = false;

                            selected_type = if selected_index < contact_index {
                                0
                            } else {
                                1
                            };
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}