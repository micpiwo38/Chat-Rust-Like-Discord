mod state;
mod event;
mod net;
mod ui;

//Gestion du panneau generale
use crossterm::{
    event::KeyCode,
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

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal).await;
    //Restauration systematique du terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

//Lancer l'application asynchone
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    //Etat de l'application
    let mut app_state = AppState::new("Vous".to_string());
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

    spawn_input_task(tx.clone());
    spawn_network_task(tx);

    //Boucle infinie du chat async
    while !app_state.should_quit {
        terminal.draw(|frame| ui::draw(frame, &app_state))?;
            if let Some(event) = rx.recv().await {
                match event {
                    AppEvent::Key(key) => {
                        //Si Ctrl + c = Quit
                        if is_quit_shortcut(&key){
                            app_state.should_quit = true;
                            continue;
                        }
                        //Le entrée clavier

                        match key.code {
                            //Avec Tab => on passe d'un panneau a l'autre
                            KeyCode::Tab => {
                                app_state.active_pane = match app_state.active_pane {
                                    ActivePane::Input => ActivePane::Channels,
                                    ActivePane::Channels => ActivePane::Input,
                                };
                            }
                            //La touche UP => si on est dans le bloc changes, on change de salon
                            KeyCode::Up if app_state.active_pane == ActivePane::Channels => {
                                if app_state.current_channel > 0{
                                    app_state.current_channel -= 1;
                                }
                            }
                            //La touche Down => si on est dans le bloc changes, on change de salon
                            KeyCode::Down if app_state.active_pane == ActivePane::Channels => {
                                if app_state.current_channel + 1 < app_state.channels.len(){
                                    app_state.current_channel += 1;
                                }
                            }
                            //La touche C = Si on est dans le panneau Input
                            KeyCode::Char(c) if app_state.active_pane == ActivePane::Input => {
                                app_state.input.push(c);
                            }
                            //La touche back space => si on est dans le bloc input
                            KeyCode::Backspace if app_state.active_pane == ActivePane::Input => {
                                app_state.input.pop();
                            }
                            //La touche entrée pour envoyer un message => si on est dans input
                            KeyCode::Enter if app_state.active_pane == ActivePane::Input => {
                                let content = app_state.input.trim().to_string(); //Supprime les espaces dans le contenu input message
                                //Si input n'est pas vide
                                if !content.is_empty(){
                                    let channel = app_state.active_channel_name().to_string(); //Recup le salon
                                    //Auteur du message
                                    let author = app_state.username.clone();
                                    //Ajouter le message a la table de Hash
                                    app_state.push_message(&channel, content, author);
                                    app_state.input.clear(); //Vider le champ input
                                    //TODO -------------------envoyer WIREMESSAGE vers la tache reseau
                                }
                            }
                            _ => {}
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
                }
            }
    }
    Ok(())
}
