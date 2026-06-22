use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::app::ActivePane;
use crate::app::AppState;

pub fn draw(f: &mut Frame, state: &mut AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, state, main_layout[0]);
    draw_content(f, state, main_layout[1]);
    draw_footer(f, state, main_layout[2]);
}

fn draw_header(f: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let status_style = if state.error_message.is_some() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if state.loading {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let status_text = match &state.error_message {
        Some(err) => format!("ERROR: {err}"),
        None => state.status_message.clone(),
    };

    let header_paragraph = Paragraph::new(status_text)
        .block(header_block)
        .style(status_style)
        .alignment(Alignment::Left);
    f.render_widget(header_paragraph, area);
}

fn draw_content(f: &mut Frame, state: &mut AppState, area: ratatui::layout::Rect) {
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(20)])
        .split(area);

    draw_games_sidebar(f, state, content_layout[0]);
    draw_servers_table(f, state, content_layout[1]);
}

fn draw_games_sidebar(f: &mut Frame, state: &mut AppState, area: ratatui::layout::Rect) {
    let border_style = if state.active_pane == ActivePane::Games {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Games ")
        .border_style(border_style);

    let items: Vec<ListItem> = state
        .games
        .iter()
        .map(|game| {
            ListItem::new(format!("  {}", game.display_name))
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let highlight_style = if state.active_pane == ActivePane::Games {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);
    f.render_stateful_widget(list, area, &mut state.games_list_state);
}

fn draw_servers_table(f: &mut Frame, state: &mut AppState, area: ratatui::layout::Rect) {
    let border_style = if state.active_pane == ActivePane::Servers {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let game_name = if state.selected_game_idx < state.games.len() {
        state.games[state.selected_game_idx].display_name.as_str()
    } else {
        ""
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Servers ({game_name}) "))
        .border_style(border_style);

    let header_cells = ["ID", "Name", "Latency", "Status"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    });
    let table_header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = state
        .servers
        .iter()
        .map(|server| {
            let block_status = if server.blocked {
                "[X] Blocked"
            } else {
                "[-] Allowed"
            };
            let block_style = if server.blocked {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            let latency_style = if let Some(ping) = server.ping {
                if ping < 80 {
                    Style::default().fg(Color::Green)
                } else if ping < 150 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                }
            } else if server.status_text == "Checking..." {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(server.id.clone()),
                Cell::from(server.description.clone()),
                Cell::from(server.status_text.clone()).style(latency_style),
                Cell::from(block_status).style(block_style),
            ])
            .style(Style::default().fg(Color::White))
            .height(1)
        })
        .collect();

    let highlight_style = if state.active_pane == ActivePane::Servers {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Min(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(table_header)
    .block(block)
    .row_highlight_style(highlight_style);

    f.render_stateful_widget(table, area, &mut state.servers_table_state);
}

fn draw_footer(f: &mut Frame, _state: &AppState, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let text = "[Q] Quit | [←/→] Switch Pane | [↑/↓] Navigate | [Space] Toggle Block | [D] Block All | [E] Unblock All | [R] Refresh";
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    f.render_widget(paragraph, area);
}
