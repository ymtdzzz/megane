use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::ui::{event_area::EventArea, side_menu::SideMenu, Drawable};

/// which component selected
enum SelectState {
    SideMenu(bool),
    EventAreas(usize),
}

pub struct App<B>
where
    B: Backend,
{
    side_menu: SideMenu<B>,
    event_areas: Vec<EventArea<B>>,
    select_state: SelectState,
}

impl<B> App<B>
where
    B: Backend,
{
    pub async fn new(side_menu: SideMenu<B>, event_areas: Vec<EventArea<B>>) -> Self {
        App {
            side_menu,
            event_areas,
            select_state: SelectState::SideMenu(true),
        }
    }

    pub fn split_logarea(&self, rect: Rect) -> Vec<Rect> {
        let constaints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let base_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constaints.as_ref())
            .split(rect);
        let mut left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constaints.as_ref())
            .split(base_chunks[0]);
        let mut right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constaints.as_ref())
            .split(base_chunks[1]);

        match self.event_areas.len() {
            1 => {
                vec![left_chunks[0]
                    .union(left_chunks[1])
                    .union(right_chunks[0])
                    .union(right_chunks[1])]
            }
            2 => {
                vec![
                    left_chunks[0].union(left_chunks[1]),
                    right_chunks[0].union(right_chunks[1]),
                ]
            }
            3 => {
                vec![
                    left_chunks[0],
                    right_chunks[0].union(right_chunks[1]),
                    left_chunks[1],
                ]
            }
            4 => {
                vec![
                    left_chunks[0],
                    right_chunks[0],
                    left_chunks[1],
                    right_chunks[1],
                ]
            }
            _ => vec![],
        }
    }

    pub fn toggle_side_fold(&mut self) {
        self.select_state = match self.select_state {
            SelectState::SideMenu(select) => SelectState::SideMenu(!select),
            SelectState::EventAreas(idx) => SelectState::EventAreas(idx),
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for App<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<B>, _area: Rect) {
        let (left, right) = match self.select_state {
            SelectState::SideMenu(show) => {
                if show {
                    (30, 70)
                } else {
                    (3, 97)
                }
            }
            SelectState::EventAreas(_) => (30, 70),
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(left), Constraint::Percentage(right)].as_ref())
            .split(f.size());
        // update select state
        self.side_menu.set_select(match self.select_state {
            SelectState::SideMenu(_) => true,
            SelectState::EventAreas(_) => false,
        });
        // draw side menu and event areas
        self.side_menu.draw(f, chunks[0]);
        let logarea_rects = self.split_logarea(chunks[1]);
        for (i, v) in self.event_areas.iter_mut().enumerate() {
            v.draw(f, logarea_rects[i]);
        }
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        let solved = match self.select_state {
            SelectState::SideMenu(_) => self.side_menu.handle_event(event).await,
            SelectState::EventAreas(idx) => {
                if let Some(logarea) = self.event_areas.get_mut(idx) {
                    logarea.handle_event(event).await
                } else {
                    false
                }
            }
        };
        if !solved {
            match event.code {
                KeyCode::Tab => {
                    self.toggle_side_fold();
                }
                _ => {}
            }
        }
        true
    }
}


