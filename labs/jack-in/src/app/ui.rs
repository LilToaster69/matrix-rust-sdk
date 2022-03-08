use std::time::Duration;

use symbols::line;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, LineGauge, Paragraph, Row, Tabs, List, ListItem, Table};
use tui::{symbols, Frame};
use tui_logger::TuiLoggerWidget;

use super::actions::Actions;
use super::state::{AppState, Syncv2State, SlidingSyncState};
use crate::app::App;

pub fn draw<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{

    let size = rect.size();

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),   // header
                Constraint::Min(10),     // body
                Constraint::Length(12),  // logs 
                Constraint::Length(3),   //footer
            ]
            .as_ref(),
        )
        .split(size);

    // Title
    let title = draw_title(app.title());
    rect.render_widget(title, chunks[0]);

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(chunks[1]);

    let bodyv3 = draw_sliding(app.state().get_sliding());
    rect.render_widget(bodyv3, body_chunks[0]);

    let bodyv2 = draw_v2(app.state().get_v2());
    rect.render_widget(bodyv2, body_chunks[1]);
    // Logs
    let logs = draw_logs();
    rect.render_widget(logs, chunks[2]);

    // Footer
    let footer = draw_footer(app.is_loading(), app.state());
    rect.render_widget(footer, chunks[3]);

}

fn draw_title<'a>(title: Option<String>) -> Paragraph<'a> {
    Paragraph::new(title.map(|n| format!("Sliding Sync for: {}", n)).unwrap_or_else(||"loading...".to_owned()))
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}


fn calc_v2<'a>(state: Option<&Syncv2State>) -> Vec<ListItem<'a>> {
    if state.is_none() {
        return vec![ListItem::new("Sync v2 hasn't started yet")]
    }

    let state = state.expect("We've tested before");
    let mut paras = vec![];

    if let Some(dur) = state.time_to_first_render() {
        paras.push(ListItem::new(format!("took {}s", dur.as_secs())));
    } else {
        paras.push(ListItem::new(format!("loading for {}s", state.started().elapsed().as_secs())));

    }

    if let Some(count) = state.rooms_count() {
        paras.push(ListItem::new(format!("to load {} rooms", count)));
    }

    return paras;

}


fn draw_sliding<'a>(state: Option<&SlidingSyncState>) -> List<'a> {
    List::new(calc_sliding(state))
    .style(Style::default().fg(Color::LightCyan))
    .block(
        Block::default()
            .title("Sliding Sync")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}


fn calc_sliding<'a>(state: Option<&SlidingSyncState>) -> Vec<ListItem<'a>> {
    if state.is_none() {
        return vec![ListItem::new("Sliding sync hasn't started yet")]
    }

    let state = state.expect("We've tested before");
    let mut paras = vec![];

    if let Some(dur) = state.time_to_first_render() {
        paras.push(ListItem::new(format!("First view took {}s", dur.as_secs())));
    } else {
        paras.push(ListItem::new(format!("loading for {}s", state.started().elapsed().as_secs())));
    }

    if let Some(dur) = state.time_to_full_sync() {
        paras.push(ListItem::new(format!("Full sync took {}s", dur.as_secs())));
    } else {
        paras.push(ListItem::new(format!("loading for {}s", state.started().elapsed().as_secs())));
    }

    if let Some(count) = state.rooms_count() {
        paras.push(ListItem::new(format!("to load {} rooms", count)));
    }

    return paras;

}


fn draw_v2<'a>(state: Option<&Syncv2State>) -> List<'a> {
    List::new(calc_v2(state))
    .style(Style::default().fg(Color::LightCyan))
    .block(
        Block::default()
            .title("Sync v2")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn draw_footer<'a>(loading: bool, state: &AppState) -> Tabs<'a> {
    if !state.is_initialized() {
        return Tabs::new(vec![Spans::from("initialising")]);
    }
    Tabs::new(vec![Spans::from("ESC / <q> to quit")])
    .style(Style::default().fg(Color::LightCyan))
    .block(
        Block::default()
            // .title("Body")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .title("Logs")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
}
