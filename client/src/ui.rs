//Le rendu (ui.rs) : fonction pure qui lit AppState et dessine le layout avec ratatui.
//Etat de l'application
use crate::state::{ActivePane, AppState, ScreenMode};
//Ratatui => UI
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};



pub fn draw(frame: &mut Frame, state: &AppState){
    match state.screen {
        ScreenMode::Login => draw_login(frame, state),
        ScreenMode::Chat => draw_chat(frame, state),
    }
}

fn draw_login(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let box_area = ratatui::layout::Rect {
        x: area.width / 4,
        y: area.height / 3,
        width: area.width.max(20) / 2,
        height: 3,
    };

    let widget = ratatui::widgets::Paragraph::new(state.login_input.as_str()).block(
        ratatui::widgets::Block::default()
            .title(" Choisis ton pseudo (Entrée pour valider) ")
            .borders(ratatui::widgets::Borders::ALL),
    );
    frame.render_widget(widget, box_area);

    // Positionne le curseur juste après le texte saisi (à l'intérieur des bordures : +1 en x, +1 en y)
    let cursor_x = box_area.x + 1 + state.login_input.chars().count() as u16;
    let cursor_y = box_area.y + 1;
    frame.set_cursor_position((cursor_x.min(box_area.x + box_area.width.saturating_sub(2)), cursor_y));
}

fn color_for_author(author: &str, own_username: &str) -> Color {
    if author == "Système" {
        return Color::Yellow;
    }
    if author == own_username {
        return Color::Green;
    }

    const PALETTE: &[Color] = &[
        Color::Cyan,
        Color::Magenta,
        Color::Blue,
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
        Color::LightMagenta,
    ];

    let mut hash: usize = 0;
    for b in author.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(*b as usize);
    }

    PALETTE[hash % PALETTE.len()]
}

pub fn draw_chat(frame: &mut Frame, state: &AppState) {
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
                    let author_color = color_for_author(&m.author, &state.username);
                    Line::from(vec![
                        Span::styled(
                            format!("[{}]", m.author),
                            Style::default()
                                .fg(author_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&m.content),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    //Le bloc des messages
    let messages_widget = Paragraph::new(message_lines.clone())
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(format!(
                    " #{current_channel} - Tab: Salon - Chat | PgUp/PgDn: scroll | Ctrl + c: Quitter"
                ))
                .borders(Borders::ALL),
        );
    //Hauteur et largeur interieurs (hors bordure)
    let inner_width = right[0].width.saturating_sub(2).max(1) as usize;
    let inner_height = right[0].height.saturating_sub(2);
    let total_lines: u16 = message_lines.iter().map(|line|{
        let line_len = line.width(); //Largeur d'une cellule
        let wrapped = if line_len == 0{
            1
        }else{
            line_len.div_ceil(inner_width.into())
        };
        wrapped as u16
    }).sum();

    let max_scroll = total_lines.saturating_sub(inner_height);
    let effective_offset = state.scroll_offset.min(max_scroll);
    let scroll_y = max_scroll.saturating_sub(effective_offset);
    //Afficher le panneau de droite
    frame.render_widget(messages_widget.scroll((scroll_y,0)), right[0]);

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
    //Afficher le curseur dans la zone input text
    if state.active_pane == ActivePane::Input {
        let char_count = state.input.chars().count() as u16;
        let max_x = right[1].x + right[1].width.saturating_sub(2);
        let cursor_x = (right[1].x + 1 + char_count).min(max_x);
        let cursor_y = right[1].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
