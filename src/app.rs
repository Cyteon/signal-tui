use std::{collections::HashMap, process::ChildStdout, sync::RwLock, thread};

use crossterm::{event::{self, Event}};
use color_eyre::Result;
use directories::ProjectDirs;
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect}, style::{Color, Style}, widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap}, DefaultTerminal
};
use rusqlite::Connection;

use crate::{signal, types};

pub fn app(
    terminal: &mut DefaultTerminal,
    stdin: &mut std::process::ChildStdin, 
    stdout: std::sync::Arc<std::sync::Mutex<ChildStdout>>,
    account_number: String,
) -> Result<()> {
    let path = ProjectDirs::from(
        "dev", 
        "cyteon", 
        "signal-tui"
    ).map(|proj_dirs| {
        proj_dirs.data_local_dir().to_path_buf()
    }).unwrap();
    std::fs::create_dir_all(&path)?;

    let db = Connection::open(path.join("data.db")).unwrap();

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

    let mut input_text = String::new();

    // 0 = group, 1 = contact
    let mut selected_type: usize = 0;

    let mut message_index: usize = 0;

    signal::subscribe_receive(stdin);

    let stoud_clone = stdout.clone();
    thread::spawn({
        move || {
            loop {
                {
                    let mut stdout: std::sync::MutexGuard<'_, ChildStdout> = stoud_clone.lock().unwrap();
                    signal::read_events_countinously(&mut *stdout);
                }
            }
        }
    });

    // (source name, message), cant be bothered to impl a better way rn :sob:
    let mut messages: Vec<(String, String, String)> = vec![];

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
                "unfocus input"
            } else if location_selected {
                "back"
            } else {
                "exit"
            };
            
            let last_action = if chatting {
                " | 'enter' - send message"
            } else if location_selected {
                " | 'e' - focus input"
            } else if !location_selected {
                " | 'enter' - select"
            } else {
                ""
            };
            
            let chat_block = Block::default()
                .borders(Borders::ALL).border_type(BorderType::Rounded)
                .title(format!(" 'esc' - {} | up/down - navigate{} ", esc_action, last_action))
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
                let title = Paragraph::new(
                    format!(" ▼ Groups ({})", groups.len())
                )
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
                let title = Paragraph::new(
                    format!(" ► Groups ({})", groups.len())
                )
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
                let title = Paragraph::new(
                    format!(" ▼ People ({})", contacts.len())
                )
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
                let title = Paragraph::new(
                    format!(" ► People ({})", contacts.len())
                )
                    .style(contact_title_style);

                f.render_widget(
                    title, contacts_layout[index]
                );
            }

            if location_selected {
                let messages_last_time = messages.len();
                let was_at_bottom = message_index >= messages.len().saturating_sub(1);
                messages = vec![];

                if selected_type == 0 {
                    let group_id = index_group_map.get(&selected_index).unwrap().id.clone();

                    let mut query = db.prepare("SELECT sourceName, sourceNumber, message FROM messages WHERE groupId = ? and pending = 0 and accountNumber = ?").unwrap();
                    let mut rows = query.query(&[&group_id, &account_number]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        let source_name: String = row.get(0).unwrap();
                        let source_number: String = row.get(1).unwrap_or_default();
                        let message: String = row.get(2).unwrap();

                        messages.push((source_name, source_number, message));
                    }
                } else {
                    let contact_uuid = index_contact_map.get(&selected_index).unwrap().uuid.clone();

                    let mut query = db.prepare("SELECT sourceName, sourceNumber, message FROM messages WHERE (destinationUuid = ? OR (sourceUuid = ? AND destinationUuid = 'self')) and pending = 0 and accountNumber = ?").unwrap();
                    let mut rows = query.query(&[&contact_uuid, &contact_uuid, &account_number]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        let source_name: String = row.get(0).unwrap();
                        let source_number: String = row.get(1).unwrap_or_default();
                        let message: String = row.get(2).unwrap();

                        messages.push((source_name, source_number, message));
                    }
                }

                let chat_area = chat_block.inner(h_chunks[1]);
                let chat_height = chat_area.height as usize;
                let chat_width = chat_area.width as usize;

                let mut message_line_counts = Vec::with_capacity(messages.len());
                for (source_name, source_number, message) in &messages {
                    let author = if source_number.is_empty() {
                        source_name.clone()
                    } else if source_number == &account_number {
                        "(you)".to_string()
                    } else {
                        source_name.clone()
                    };
                    let text = format!("{}: {}", author, message);

                    let line_count = text
                        .chars()
                        .collect::<Vec<_>>()
                        .chunks(chat_width.saturating_sub(2).max(1))
                        .count()
                        .max(1);
                    message_line_counts.push(line_count);
                }

                let available_lines = chat_height.saturating_sub(3);
                let mut total_lines = 0;
                let start = scroll_offset;
                let mut end = scroll_offset;
                while end < messages.len() && total_lines + message_line_counts[end] <= available_lines {
                    total_lines += message_line_counts[end];
                    end += 1;
                }
                let visible_count = end - start;

                let mut msg_line_sum = 0;
                for i in 0..message_index {
                    msg_line_sum += message_line_counts.get(i).copied().unwrap_or(1);
                }
                if msg_line_sum < scroll_offset {
                    scroll_offset = msg_line_sum;
                } else if msg_line_sum >= scroll_offset + available_lines {
                    scroll_offset = msg_line_sum + 1 - available_lines;
                }

                let mut c: Vec<Constraint> = Vec::new();
                for i in start..end {
                    c.push(Constraint::Length(message_line_counts[i] as u16));
                }
                // have 1 extra for "no messages" label
                if messages.len() == 0 {
                    c.push(Constraint::Length(1));
                }
                c.push(Constraint::Length(3));

                let chat_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(c)
                    .split(chat_area);

                let mut layout_idx = 0;
                for (i, (source_name, source_number, message)) in messages.iter().enumerate().skip(start).take(visible_count) {
                    let style = if i == message_index {
                        Style::default().bg(Color::Blue)
                    } else {
                        Style::default()
                    };

                    let author = if source_number.is_empty() {
                        source_name.clone()
                    } else if source_number == &account_number {
                        "(you)".to_string()
                    } else {
                        source_name.clone()
                    };

                    let p = Paragraph::new(format!("{}: {}", author, message))
                        .style(style)
                        .wrap(Wrap { trim: false });

                    f.render_widget(p, chat_layout[layout_idx]);
                    layout_idx += 1;
                }

                if messages.len() == 0 {
                    let layout = chat_layout[0];
                    let p = Paragraph::new(
                        "No messages :("
                    ).style(Style::default());
                    f.render_widget(p, layout);
                    layout_idx += 1;
                }

                if was_at_bottom {
                    message_index = messages.len().saturating_sub(1);
                    scroll_offset = messages.len().saturating_sub(visible_count);
                }

                let input = Paragraph::new(input_text.clone())
                    .block(Block::bordered().title("Input"))
                    .style(match chatting {
                        true => Style::default().fg(Color::Blue),
                        false => Style::default()
                    });

                f.render_widget(
                    input,
                    if messages.len() == 0 {
                        chat_layout[1] // 1 cause the no messages label
                    } else {
                        chat_layout[layout_idx]
                    }
                );

                if chatting {
                    f.set_cursor_position(Position::new(
                        chat_layout[layout_idx].x + 1 + input_text.len() as u16,
                        chat_layout[layout_idx].y + 1 + if messages.len() == 0 { 1 } else { 0 }
                    ));
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('e') => {
                        if location_selected && !chatting {
                            chatting = true;
                        } else if chatting {
                            input_text.push('e');
                        }
                    },

                    crossterm::event::KeyCode::Esc => {
                        if chatting {
                            chatting = false;
                        } else if location_selected {
                            location_selected = false;
                            chatting = false;
                            message_index = 0;  
                            input_text = String::new();
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
                        } else if chatting {
                            signal::send_msg(
                                stdin,
                                input_text.clone(),
                                if selected_type == 0 {
                                    index_group_map.get(&selected_index).unwrap().id.clone()
                                } else {
                                    index_contact_map.get(&selected_index).unwrap().uuid.clone()
                                },
                                selected_type,
                                &db,
                                account_number.clone(),
                            );

                            input_text = String::new();
                            chatting = false;
                        } else {
                            location_selected = true;
                            chatting = false;
                            message_index = 0;
                            scroll_offset = 0;

                            selected_type = if selected_index < contact_index {
                                0
                            } else {
                                1
                            };
                        }
                    }

                    char => {
                        if chatting {
                            if char == crossterm::event::KeyCode::Backspace {
                                if !input_text.is_empty() {
                                    input_text.pop();
                                }
                            } else {
                                if let crossterm::event::KeyCode::Char(c) = char {
                                    input_text.push(c);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}