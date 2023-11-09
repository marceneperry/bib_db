// use crossterm::style::Stylize;

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line},
    widgets::{Block, Borders},
    Frame,
};
// use ratatui::prelude::Marker::Block;
use ratatui::prelude::Stylize;
use ratatui::widgets::Tabs;

use crate::app::{App};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    let block = Block::default().on_white().black();
    f.render_widget(block, size);

    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![first.yellow(), rest.green()])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.index)
        .style(Style::default().cyan().on_gray())
        .highlight_style(Style::default().bold().on_black());
    f.render_widget(tabs, chunks[0]);

    let inner = match app.index {
        0 => Block::default().title("Inner 0").borders(Borders::ALL),
        1 => Block::default().title("Inner 1").borders(Borders::ALL),
        2 => Block::default().title("Inner 2").borders(Borders::ALL),
        _ => unreachable!(),
    };
    f.render_widget(inner, chunks[1]);
}

