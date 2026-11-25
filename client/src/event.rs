/*
La boucle TUI / Événements (app.rs & event.rs) :
lit le clavier (dans un thread dédié ou via crossterm::event::EventStream)
et reçoit les messages réseau via un mpsc::unbounded_channel.
*/

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}; //Gestion des evenements
//Timer
use std::time::{Duration, Instant};
//Asynchrone
use tokio::sync::mpsc;

//Enumeration des evenements = lier state.rs avec network
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent), //Entrée clavier
    NetworkMessage {
        channel: String,
        author: String,
        content: String,
    },
    Tick,
}

//Lancer les events d'ecoute clavier dans un thread dédier pour ne jamais bloquer async Tokio
pub fn spawn_input_task(tokio_x: mpsc::UnboundedSender<AppEvent>) {
    std::thread::spawn(move || loop {
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                // On ne garde que l'appui réel de la touche (ignore Repeat et Release)
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if tokio_x.send(AppEvent::Key(key)).is_err() {
                    break;
                }
            }
        } else if tokio_x.send(AppEvent::Tick).is_err() {
            break;
        }
    });
}

//Racourci clavier pour quitter Salon ? Application ? Ctrl + c
pub fn is_quit_shortcut(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c')
}
