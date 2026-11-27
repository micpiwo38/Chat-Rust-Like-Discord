mod state;
mod event;
mod net;
mod ui;

//Gestion du panneau generale
use crossterm::{
    event::{KeyCode, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{ EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
//Evenements
use event::{AppEvent, is_quit_shortcut, spawn_input_task};
//Network
use net::spawn_network_task;
//Ratatui Terminal
use ratatui::{Terminal, backend::CrosstermBackend};
//Etat de l'application
use state::{ActivePane, AppState};
//Standard IN - OUT
use std::io;
//Async - Synch
use tokio::sync::mpsc;
use crate::net::WireMessage;
use crate::state::ScreenMode;

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal).await;
    //Restauration systematique du terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

//Lancer l'application asynchone
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    //Etat de l'application
    let mut app_state = AppState::new("Vous".to_string());
    //Par defaut l'ecran de login avec le chat
    app_state.screen = ScreenMode::Login;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();
    let (net_tx, net_rx) = mpsc::unbounded_channel::<WireMessage>();

    spawn_input_task(event_tx.clone());
    spawn_network_task("127.0.0.1:4000".to_string(),event_tx,net_rx);

    //Boucle infinie du chat async
    while !app_state.should_quit {
        terminal.draw(|frame| ui::draw(frame, &app_state))?;
            if let Some(event) = event_rx.recv().await {
                match event {
                    AppEvent::Key(key) => {
                        //Si Ctrl + c = Quit
                        if is_quit_shortcut(&key){
                            app_state.should_quit = true;
                            continue;
                        }
                        //Le entrée clavier

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
                                        app_state.input.clear(); // Important : vide ce qui avait été tapé pendant le login
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
                                    app_state.scroll_offset = app_state.scroll_offset.saturating_add_signed(5);
                                    if app_state.current_channel > 0 {
                                        app_state.current_channel -= 1;
                                    }
                                }
                                KeyCode::Down if app_state.active_pane == ActivePane::Channels => {
                                    app_state.scroll_offset = app_state.scroll_offset.saturating_sub_signed(5);
                                    if app_state.current_channel + 1 < app_state.channels.len() {
                                        app_state.current_channel += 1;
                                    }
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
                        app_state.push_message(&channel, author, content);
                    }
                    AppEvent::Tick => {}
                    AppEvent::Scroll(delta) if app_state.screen == ScreenMode::Chat => {
                        if delta > 0 {
                            app_state.scroll_offset = app_state.scroll_offset.saturating_add(delta as u16);
                        }else{
                            app_state.scroll_offset = app_state.scroll_offset.saturating_sub(delta.unsigned_abs());
                        }
                    }
                    _ => {}
                }
            }
    }
    Ok(())
}
