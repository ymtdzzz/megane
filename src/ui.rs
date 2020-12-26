use std::io::Stdout;

use super::app::App;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Block,
    Frame,
};

mod side_menu;

/// ui module draws whole screen according to app state.
/// It has two areas: SideMenu and MainArea.
/// SideMenu is for showing the log groups.
/// MainArea is for showing the log events belong to the selected log group.
pub fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, _app: &mut App) {
    // Layout
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let side_menu = Block::default()
        .title("log groups")
        .border_style(Style::default().fg(Color::Red));
    f.render_widget(side_menu, chunks[0]);
}
