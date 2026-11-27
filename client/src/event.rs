/*
La boucle TUI / Événements (app.rs & event.rs) :
lit le clavier (dans un thread dédié ou via crossterm::event::EventStream)
et reçoit les messages réseau via un mpsc::unbounded_channel.
*/

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind}; //Gestion des evenements
//Timer
use std::time::{Duration};
//Asynchrone
use tokio::sync::mpsc;

//Enumeration des evenements = lier state.rs avec network
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent), //Entrée clavier
    Scroll(i16),
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
            match event::read() {
                Ok(Event::Key(key)) => {
                    if key.kind != KeyEventKind::Press{
                        continue;
                    }
                    if tokio_x.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
                Ok(Event::Mouse(mouse)) => match mouse.kind{
                    MouseEventKind::ScrollUp => {
                        let _ = tokio_x.send(AppEvent::Scroll(3));
                    }
                    MouseEventKind::ScrollDown => {
                        let _ = tokio_x.send(AppEvent::Scroll(-3));
                    }
                    _ => {}
                }
                _ => {}
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
