//Le rendu (ui.rs) : fonction pure qui lit AppState et dessine le layout avec ratatui.
//Etat de l'application
use crate::state::{ActivePane, AppState};
//Ratatui => UI
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, state: &AppState) {
    //Couper le layout principale en 2 parties horizontale
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.area());
    //-----------------------------------------------PANNEAU DE GAUCHE--------------------------------------------//
    //Colonne de gauche 20% => salon
    let items: Vec<ListItem> = state
        .channels
        .iter()
        .enumerate()
        .map(|(channel_index, channel)| {
            let prefix = if channel_index == state.current_channel {
                "> #"
            } else {
                " #"
            };
            let style = if channel_index == state.current_channel {
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{prefix}{channel}"),
                style,
            )))
        })
        .collect();
    //Bordure des salons
    let channels_border = if state.active_pane == ActivePane::Channels {
        Color::LightMagenta
    } else {
        Color::LightBlue
    };
    //Widget des salons
    let channels_widget = List::new(items).block(
        Block::default()
            .title(" - SALONS - ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(channels_border)),
    );
    //Afficher le panneau de gauche
    frame.render_widget(channels_widget, root[0]);
    //-----------------------------------------------PANNEAU DE DROITE-----------------------------------------------//
    //Historique + zone de saisie
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Max(3)])
        .split(root[1]);
    //Salon en cours
    let current_channel = state.active_channel_name();
    //Ligne des message de la hash map
    let message_lines: Vec<Line> = state
        .messages
        .get(current_channel)
        .map(|messages| {
            //Boucle de parcours historique
            messages
                .iter()
                .map(|m| {
                    Line::from(vec![
                        Span::styled(
                            format!("[{}]", m.author),
                            Style::default()
                                .fg(Color::LightMagenta)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&m.content),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    //Le bloc des messages
    let messages_widget = Paragraph::new(message_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(format!(
                    " #{current_channel} - Tab: Focus | Ctrl + c: Quitter"
                ))
                .borders(Borders::ALL),
        );
    //Afficher le panneau de droite
    frame.render_widget(messages_widget, right[0]);

    //Bordure de la zone d'entrée des messages
    let input_border = if state.active_pane == ActivePane::Input {
        Color::LightGreen
    } else {
        Color::LightRed
    };

    //Le widget d'input
    let input_widget = Paragraph::new(state.input.as_str()).block(
        Block::default()
            .title(format!(" Message : ({})", state.username))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(input_border)),
    );
    //Afficher le bloc input Droite Verticale Bas
    frame.render_widget(input_widget, right[1]);
}
