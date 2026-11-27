mod event;
mod net;
mod state;
mod ui;

// Gestion du panneau général
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
// Événements
use event::{AppEvent, is_quit_shortcut, spawn_input_task};
// Network
use net::spawn_network_task;
// Ratatui Terminal
use ratatui::{Terminal, backend::CrosstermBackend};
// État de l'application
use state::{ActivePane, AppState, ScreenMode};
// Standard IN - OUT
use std::io;
// Async - Synch
use tokio::sync::mpsc;
use crate::net::WireMessage;

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal).await;

    // Restauration systématique du terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}
//Fonction helper pour resoudre l'hote et IP => par defaut "127.0.0.1:4000".to_string()
fn resolve_server_addr() -> String {
    if let Some(addr) = std::env::args().nth(1){
        let trimmed = addr.trim().to_string();
        if !trimmed.is_empty(){
            return trimmed;
        }
    }
    if let Ok(addr) = std::env::var("CHAT_SERVER"){
        let trimmed = addr.trim().to_string();
        if !trimmed.is_empty(){
            return trimmed;
        }
    }
    "127.0.0.1:4000".to_string()
}

// Lancer l'application asynchrone
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app_state = AppState::new("Vous".to_string());
    app_state.screen = ScreenMode::Login;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();
    let (net_tx, net_rx) = mpsc::unbounded_channel::<WireMessage>();

    spawn_input_task(event_tx.clone());
    let server_addr = resolve_server_addr();
    println!("Server address: {}", server_addr);
    spawn_network_task(server_addr, event_tx, net_rx);

    while !app_state.should_quit {
        terminal.draw(|frame| ui::draw(frame, &mut app_state))?;

        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Key(key) => {
                    if is_quit_shortcut(&key) {
                        app_state.should_quit = true;
                        continue;
                    }

                    match app_state.screen {
                        ScreenMode::Login => match key.code {
                            KeyCode::Char(c) => {
                                app_state.login_input.push(c);
                            }
                            KeyCode::Backspace => {
                                app_state.login_input.pop();
                            }
                            KeyCode::Enter => {
                                let pseudo = app_state.login_input.trim().to_string();
                                if !pseudo.is_empty() {
                                    app_state.username = pseudo;
                                    app_state.login_input.clear();
                                    app_state.input.clear();
                                    app_state.screen = ScreenMode::Chat;
                                }
                            }
                            _ => {}
                        },
                        ScreenMode::Chat => match key.code {
                            KeyCode::Tab => {
                                app_state.active_pane = match app_state.active_pane {
                                    ActivePane::Input => ActivePane::Channels,
                                    ActivePane::Channels => ActivePane::Input,
                                };
                            }
                            KeyCode::Up if app_state.active_pane == ActivePane::Channels => {
                                if app_state.current_channel > 0 {
                                    let prev_chanel = app_state.current_channel - 1;
                                    app_state.select_channel(prev_chanel);
                                }
                            }
                            KeyCode::Down if app_state.active_pane == ActivePane::Channels => {
                                if app_state.current_channel + 1 < app_state.channels.len() {
                                    let next_chanel = app_state.current_channel + 1;
                                    app_state.select_channel(next_chanel);
                                }
                            }
                            KeyCode::PageUp => {
                                app_state.scroll_offset = app_state.scroll_offset.saturating_add(5);
                            }
                            KeyCode::PageDown => {
                                app_state.scroll_offset = app_state.scroll_offset.saturating_sub(5);
                            }
                            KeyCode::Char(c) if app_state.active_pane == ActivePane::Input => {
                                app_state.input.push(c);
                            }
                            KeyCode::Backspace if app_state.active_pane == ActivePane::Input => {
                                app_state.input.pop();
                            }
                            KeyCode::Enter if app_state.active_pane == ActivePane::Input => {
                                let content = app_state.input.trim().to_string();
                                if !content.is_empty() {
                                    let channel = app_state.active_channel_name().to_string();
                                    let author = app_state.username.clone();
                                    let msg = WireMessage::Chat {
                                        channel,
                                        author,
                                        content,
                                    };
                                    let _ = net_tx.send(msg);
                                    app_state.input.clear();
                                }
                            }
                            _ => {}
                        },
                    }
                }
                AppEvent::NetworkMessage {
                    channel,
                    author,
                    content,
                } => {
                    app_state.on_network_message(channel, author, content);
                }
                AppEvent::Scroll(delta) if app_state.screen == ScreenMode::Chat => {
                    if delta > 0 {
                        app_state.scroll_offset = app_state.scroll_offset.saturating_add(delta as u16);
                    } else {
                        app_state.scroll_offset =
                            app_state.scroll_offset.saturating_sub(delta.unsigned_abs());
                    }
                }
                AppEvent::Click((column, row)) if app_state.screen == ScreenMode::Chat => {
                    if column < 0 || row < 0 {
                        continue;
                    }
                    let col = column as u16;
                    let r = row as u16;

                    // 1. Clic dans la zone des salons
                    let ch_area = app_state.channels_area;
                    let inside_ch_x = col >= ch_area.x && col < ch_area.x + ch_area.width;
                    let inside_ch_y = r > ch_area.y && r + 1 < ch_area.y + ch_area.height;

                    if inside_ch_x && inside_ch_y {
                        let index = (r - ch_area.y - 1) as usize;
                        if index < app_state.channels.len() {
                            app_state.select_channel(index);
                            app_state.active_pane = ActivePane::Channels;
                        }
                        continue;
                    }

                    // 2. Clic dans la zone de saisie chat
                    let in_area = app_state.input_area;
                    let inside_in_x = col >= in_area.x && col < in_area.x + in_area.width;
                    let inside_in_y = r >= in_area.y && r < in_area.y + in_area.height;

                    if inside_in_x && inside_in_y {
                        app_state.active_pane = ActivePane::Input;
                    }
                }
                AppEvent::Tick => {}
                _ => {}
            }
        }
    }
    Ok(())
}