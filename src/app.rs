use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    event::LogEventEvent,
    state::logevents_state::LogEventsState,
    ui::{event_area::EventArea, help::Help, side_menu::SideMenu, status_bar::StatusBar, Drawable},
};

/// which component selected
#[derive(PartialEq, Debug)]
pub enum SelectState {
    SideMenu,
    EventAreas(usize),
}

pub struct App<B>
where
    B: Backend,
{
    side_menu: SideMenu<B>,
    event_areas: Vec<EventArea<B>>,
    logevent_states: [Arc<Mutex<LogEventsState>>; 4],
    logevent_inst_txs: [mpsc::Sender<LogEventEvent>; 4],
    status_bar: StatusBar<B>,
    select_state: SelectState,
    show_help: bool,
    fold: bool,
    help: Help<B>,
    free_idx: [bool; 4],
}

impl<B> App<B>
where
    B: Backend,
{
    pub async fn new(
        side_menu: SideMenu<B>,
        event_areas: Vec<EventArea<B>>,
        logevent_states: [Arc<Mutex<LogEventsState>>; 4],
        logevent_inst_txs: [mpsc::Sender<LogEventEvent>; 4],
        status_bar: StatusBar<B>,
        show_help: bool,
        fold: bool,
    ) -> Self {
        App {
            side_menu,
            event_areas,
            logevent_states,
            logevent_inst_txs,
            status_bar,
            select_state: SelectState::SideMenu,
            show_help,
            fold,
            help: Help::new(),
            free_idx: [true, true, true, true],
        }
    }

    pub fn split_event_area(&self, rect: Rect) -> Vec<Rect> {
        let constaints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let base_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constaints.as_ref())
            .split(rect);
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constaints.as_ref())
            .split(base_chunks[0]);
        let right_chunks = Layout::default()
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
        self.fold = !self.fold;
    }

    pub fn toggle_show_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn rotate_select_state(&mut self, key: KeyCode) {
        let event_areas_len = self.event_areas.len();
        match self.select_state {
            SelectState::SideMenu => {
                if let KeyCode::Right = key {
                    if event_areas_len > 0 {
                        self.select_state = SelectState::EventAreas(0);
                    }
                }
            }
            SelectState::EventAreas(idx) => match key {
                KeyCode::Left => match idx {
                    0 => self.select_state = SelectState::SideMenu,
                    1 => self.select_state = SelectState::EventAreas(0),
                    2 => self.select_state = SelectState::SideMenu,
                    3 => self.select_state = SelectState::EventAreas(2),
                    _ => {}
                },
                KeyCode::Right => match idx {
                    0 => {
                        if event_areas_len > 1 {
                            self.select_state = SelectState::EventAreas(1);
                        }
                    }
                    2 => {
                        if event_areas_len > 3 {
                            self.select_state = SelectState::EventAreas(3);
                        } else {
                            self.select_state = SelectState::EventAreas(1);
                        }
                    }
                    _ => {}
                },
                KeyCode::Down => match idx {
                    0 => {
                        if event_areas_len > 2 {
                            self.select_state = SelectState::EventAreas(2);
                        }
                    }
                    1 => {
                        if event_areas_len > 3 {
                            self.select_state = SelectState::EventAreas(3);
                        }
                    }
                    _ => {}
                },
                KeyCode::Up => match idx {
                    2 => self.select_state = SelectState::EventAreas(0),
                    3 => self.select_state = SelectState::EventAreas(1),
                    _ => {}
                },
                _ => {}
            },
        }
    }

    /// get index to push the next event_area
    pub fn get_next_idx(&self) -> Result<usize> {
        for (idx, is_free) in self.free_idx.iter().enumerate() {
            if *is_free {
                return Ok(idx);
            }
        }
        Err(anyhow!("Free idx does not exist. Something wrong."))
    }
}

impl<B> Default for App<B>
where
    B: Backend,
{
    fn default() -> Self {
        // dummy channel
        let (tx1, _rx1): (mpsc::Sender<LogEventEvent>, mpsc::Receiver<LogEventEvent>) =
            mpsc::channel(1);
        let (tx2, _rx2): (mpsc::Sender<LogEventEvent>, mpsc::Receiver<LogEventEvent>) =
            mpsc::channel(1);
        let (tx3, _rx3): (mpsc::Sender<LogEventEvent>, mpsc::Receiver<LogEventEvent>) =
            mpsc::channel(1);
        let (tx4, _rx4): (mpsc::Sender<LogEventEvent>, mpsc::Receiver<LogEventEvent>) =
            mpsc::channel(1);
        App {
            side_menu: SideMenu::default(),
            event_areas: vec![],
            logevent_states: [
                Arc::new(Mutex::new(LogEventsState::default())),
                Arc::new(Mutex::new(LogEventsState::default())),
                Arc::new(Mutex::new(LogEventsState::default())),
                Arc::new(Mutex::new(LogEventsState::default())),
            ],
            logevent_inst_txs: [tx1, tx2, tx3, tx4],
            status_bar: StatusBar::default(),
            select_state: SelectState::SideMenu,
            show_help: false,
            fold: false,
            help: Help::default(),
            free_idx: [true, true, true, true],
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for App<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, _area: Rect) {
        let (left, right) = if self.fold { (3, 97) } else { (30, 70) };
        // base_chunks[0] - side menu and event area
        // base_chunks[1] - status bar area
        let base_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(100), Constraint::Length(2)].as_ref())
            .split(f.size());
        // chunks[0] - side menu
        // chunks[1] - event area
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(left), Constraint::Percentage(right)].as_ref())
            .split(base_chunks[0]);
        // update select state
        self.side_menu.set_select(match self.select_state {
            SelectState::SideMenu => true,
            SelectState::EventAreas(_) => false,
        });
        for (i, v) in self.event_areas.iter_mut().enumerate() {
            v.set_select(match self.select_state {
                SelectState::SideMenu => false,
                SelectState::EventAreas(idx) => i == idx,
            })
        }
        if self.show_help {
            let chunk = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .split(f.size());
            self.help.draw(f, chunk[0]);
        } else {
            // draw side menu and event areas
            self.side_menu.draw(f, chunks[0]);
            let event_area_rects = self.split_event_area(chunks[1]);
            for (i, v) in self.event_areas.iter_mut().enumerate() {
                v.draw(f, event_area_rects[i]);
            }
            self.status_bar.draw(f, base_chunks[1]);
        }
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        let solved = match self.select_state {
            SelectState::SideMenu => self.side_menu.handle_event(event).await,
            SelectState::EventAreas(idx) => {
                if let Some(event_area) = self.event_areas.get_mut(idx) {
                    event_area.handle_event(event).await
                } else {
                    false
                }
            }
        };
        if !solved {
            match event.code {
                KeyCode::Char('?') => {
                    self.toggle_show_help();
                }
                KeyCode::Tab => {
                    self.toggle_side_fold();
                }
                KeyCode::Right => {
                    self.rotate_select_state(KeyCode::Right);
                }
                KeyCode::Up => {
                    self.rotate_select_state(KeyCode::Up);
                }
                KeyCode::Left => {
                    self.rotate_select_state(KeyCode::Left);
                }
                KeyCode::Down => {
                    self.rotate_select_state(KeyCode::Down);
                }
                KeyCode::Enter => {
                    if let SelectState::SideMenu = self.select_state {
                        // log group selection updated
                        let current_log_groups = self
                            .event_areas
                            .iter()
                            .map(|i| i.log_group_name())
                            .collect::<Vec<&str>>();
                        let log_groups_to_create = self
                            .side_menu
                            .selected_log_groups()
                            .iter()
                            .filter(|group| !current_log_groups.contains(&group.as_str()))
                            .collect::<Vec<&String>>();
                        let mut idx_to_remove = vec![];
                        current_log_groups
                            .iter()
                            .enumerate()
                            .for_each(|(i, group)| {
                                if !self
                                    .side_menu
                                    .selected_log_groups()
                                    .contains(&group.to_string())
                                {
                                    idx_to_remove.push(i);
                                }
                            });
                        for i in idx_to_remove {
                            if self.event_areas.len() > i {
                                self.event_areas.remove(i);
                                self.logevent_states[i].lock().unwrap().reset();
                                self.free_idx[i] = true;
                            }
                        }
                        for i in log_groups_to_create {
                            let idx = self.get_next_idx().unwrap();
                            self.free_idx[idx] = false;
                            let state = Arc::clone(&self.logevent_states[idx]);
                            self.event_areas.push(EventArea::new(
                                i,
                                state,
                                mpsc::Sender::clone(&self.logevent_inst_txs[idx]),
                            ));
                            let _ = self.logevent_inst_txs[idx]
                                .send(LogEventEvent::FetchLogEvents(
                                    i.to_string(),
                                    None,
                                    None,
                                    true,
                                ))
                                .await;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};

    use super::*;
    use crate::{loggroups::LogGroups, state::loggroups_state::LogGroupsState, test_helper::*};

    fn test_case(
        app: &mut App<TestBackend>,
        side_menu_color: Color,
        event_area_color: Color,
        lines: Vec<&str>,
        side_menu_length: u16,
        header_exists: bool,
    ) {
        let mut terminal = get_test_terminal(100, 10);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "┌Log Groups [type to search]─┐                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "└────────────────────────────┘                                                                      ",
                "                                                                                     initial message",
                "                                                                                                    ",
            ]
        };
        let mut expected = Buffer::with_lines(lines);
        for y in 0..10 {
            for x in 0..100 {
                let ch = expected.get_mut(x, y);
                if y == 0 {
                    if x < side_menu_length {
                        ch.set_fg(side_menu_color);
                    }
                } else if y == 7 {
                    if x >= side_menu_length {
                        if ch.symbol != " " {
                            ch.set_fg(event_area_color);
                        }
                    } else {
                        ch.set_fg(side_menu_color);
                    }
                } else if y == 2 {
                    if ch.symbol == "│" {
                        if x >= side_menu_length {
                            ch.set_fg(event_area_color);
                        } else {
                            ch.set_fg(side_menu_color);
                        }
                    } else if x >= side_menu_length && header_exists {
                        ch.set_fg(Color::White);
                    }
                } else if y == 8 {
                } else if ch.symbol != " " {
                    if x >= side_menu_length {
                        ch.set_fg(event_area_color);
                    } else {
                        ch.set_fg(side_menu_color);
                    }
                }
            }
        }
        terminal
            .draw(|f| {
                app.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_split_event_area() {
        let mut app: App<TestBackend> = App::default();
        // no event areas
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect: Vec<Rect> = vec![];
        assert_eq!(expect, result);
        // 1 event area
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![Rect::new(0, 0, 100, 100)];
        assert_eq!(expect, result);
        // 2 event areas
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![Rect::new(0, 0, 50, 100), Rect::new(50, 0, 50, 100)];
        assert_eq!(expect, result);
        // 3 event areas
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![
            Rect::new(0, 0, 50, 50),
            Rect::new(50, 0, 50, 100),
            Rect::new(0, 50, 50, 50),
        ];
        assert_eq!(expect, result);
        // 4 event areas
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![
            Rect::new(0, 0, 50, 50),
            Rect::new(50, 0, 50, 50),
            Rect::new(0, 50, 50, 50),
            Rect::new(50, 50, 50, 50),
        ];
        assert_eq!(expect, result);
    }

    #[tokio::test]
    async fn test_draw() {
        let mut app: App<TestBackend> = App::default();
        test_case(&mut app, Color::Yellow, Color::White, vec![], 30, false);
        app.event_areas.push(EventArea::default());
        let lines = vec![
            "┌Log Groups [type to search]─┐query: [], mode: [Tail]                                               ",
            "│                            │┌Events──────────────────────────────────────────────────────────────┐",
            "│                            ││   Timestamp     Event                                              │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "└────────────────────────────┘└────────────────────────────────────────────────────────────────────┘",
            "                                                                                     initial message",
            "                                                                                                    ",
        ];
        test_case(&mut app, Color::Yellow, Color::White, lines, 30, true);
        // folding side menu
        app.toggle_side_fold();
        let lines = vec![
            "┌L┐query: [], mode: [Tail]                                                                          ",
            "│ │┌Events─────────────────────────────────────────────────────────────────────────────────────────┐",
            "│ ││   Timestamp           Event                                                                   │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "└─┘└───────────────────────────────────────────────────────────────────────────────────────────────┘",
            "                                                                                     initial message",
            "                                                                                                    ",
        ];
        test_case(&mut app, Color::Yellow, Color::White, lines, 3, true);
        // event area selected
        app.toggle_side_fold();
        app.select_state = SelectState::EventAreas(0);
        let lines = vec![
            "┌Log Groups [type to search]─┐query: [], mode: [Tail]                                               ",
            "│                            │┌Events──────────────────────────────────────────────────────────────┐",
            "│                            ││   Timestamp     Event                                              │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "└────────────────────────────┘└────────────────────────────────────────────────────────────────────┘",
            "                                                                                     initial message",
            "                                                                                                    ",
        ];
        test_case(&mut app, Color::White, Color::Yellow, lines, 30, true);
        // help dialog
        app.toggle_show_help();
        let lines = vec![
            "┌HELP──────────────────────────────────────────────────────────────────────────────────────────────┐",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "└──────────────────────────────────────────────────────────────────────────────────────────────────┘",
        ];
        test_case(&mut app, Color::Reset, Color::Reset, lines, 30, false);
    }

    #[tokio::test]
    async fn test_handle_event_basis() {
        let mut app: App<TestBackend> = App::default();
        app.event_areas.push(EventArea::default());
        assert!(!app.fold);
        assert!(
            app.handle_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
                .await
        );
        assert!(app.fold);
        app.select_state = SelectState::EventAreas(0);
        assert!(
            app.handle_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
                .await
        );
        assert!(!app.fold);
        app.event_areas.pop();
        assert!(
            app.handle_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
                .await
        );
        assert!(
            app.handle_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE))
                .await
        );
        assert!(app.show_help);
        app.event_areas.push(EventArea::default());
        app.select_state = SelectState::SideMenu;
        assert!(
            app.handle_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
                .await
        );
        assert_eq!(SelectState::EventAreas(0), app.select_state);
    }

    #[tokio::test]
    async fn test_handler_event_update_eventareas() {
        // setup
        let loggroup_state = Arc::new(Mutex::new(LogGroupsState::new()));
        let mut app: App<TestBackend> = App {
            side_menu: SideMenu::new(Arc::clone(&loggroup_state)),
            ..Default::default()
        };
        // 3 log groups
        {
            let mut m_guard = loggroup_state.lock().unwrap();
            m_guard.log_groups = LogGroups::new(get_log_groups(0, 2, false));
            m_guard.get_list_items("", &[]);
        }
        // current cursor is the first log group
        app.select_state = SelectState::SideMenu;
        app.side_menu.set_select(true);
        app.side_menu
            .handle_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .await;

        // check the curernt state
        assert_eq!(0, app.event_areas.len());
        assert_eq!([true, true, true, true], app.free_idx);
        // fire the Enter event
        assert!(
            app.handle_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
                .await
        );
        assert_eq!(1, app.event_areas.len());
        assert_eq!([false, true, true, true], app.free_idx);
    }

    #[test]
    fn test_rotate_state() {
        let mut app: App<TestBackend> = App::default();
        app.event_areas.push(EventArea::default());
        app.event_areas.push(EventArea::default());
        app.event_areas.push(EventArea::default());
        app.event_areas.push(EventArea::default());
        // KeyCode::Left
        // EventArea(left-top) -> SideMenu
        app.select_state = SelectState::EventAreas(0);
        app.rotate_select_state(KeyCode::Left);
        assert_eq!(SelectState::SideMenu, app.select_state);
        // EventArea(right-top) -> EventArea(left-top)
        app.select_state = SelectState::EventAreas(1);
        app.rotate_select_state(KeyCode::Left);
        assert_eq!(SelectState::EventAreas(0), app.select_state);
        // EventArea(left-bottom) -> SideMenu
        app.select_state = SelectState::EventAreas(2);
        app.rotate_select_state(KeyCode::Left);
        assert_eq!(SelectState::SideMenu, app.select_state);
        // EventArea(right-bottom) -> EventArea(left-bottom)
        app.select_state = SelectState::EventAreas(3);
        app.rotate_select_state(KeyCode::Left);
        assert_eq!(SelectState::EventAreas(2), app.select_state);
        // KeyCode::Right
        // SideMenu -> EventArea(left-top)
        app.select_state = SelectState::SideMenu;
        app.rotate_select_state(KeyCode::Right);
        assert_eq!(SelectState::EventAreas(0), app.select_state);
        // EventArea(left-top) -> EventArea(right-top)
        app.select_state = SelectState::EventAreas(0);
        app.rotate_select_state(KeyCode::Right);
        assert_eq!(SelectState::EventAreas(1), app.select_state);
        // EventArea(right-top) -> no change
        app.select_state = SelectState::EventAreas(1);
        app.rotate_select_state(KeyCode::Right);
        assert_eq!(SelectState::EventAreas(1), app.select_state);
        // EventArea(left-bottom) -> EventArea(right-bottom)
        app.select_state = SelectState::EventAreas(2);
        app.rotate_select_state(KeyCode::Right);
        assert_eq!(SelectState::EventAreas(3), app.select_state);
        // EventArea(right-bottom) -> no change
        app.select_state = SelectState::EventAreas(3);
        app.rotate_select_state(KeyCode::Right);
        assert_eq!(SelectState::EventAreas(3), app.select_state);
        // KeyCode::Down
        // EventArea(left-top) -> EventArea(left-bottom)
        app.select_state = SelectState::EventAreas(0);
        app.rotate_select_state(KeyCode::Down);
        assert_eq!(SelectState::EventAreas(2), app.select_state);
        // EventArea(right-top) -> EventArea(right-bottom)
        app.select_state = SelectState::EventAreas(1);
        app.rotate_select_state(KeyCode::Down);
        assert_eq!(SelectState::EventAreas(3), app.select_state);
        // KeyCode::Up
        // EventArea(left-bottom) -> EventArea(left-top)
        app.select_state = SelectState::EventAreas(2);
        app.rotate_select_state(KeyCode::Up);
        assert_eq!(SelectState::EventAreas(0), app.select_state);
        // EventArea(right-bottom) -> EventArea(right-top)
        app.select_state = SelectState::EventAreas(3);
        app.rotate_select_state(KeyCode::Up);
        assert_eq!(SelectState::EventAreas(1), app.select_state);
    }

    #[test]
    fn test_get_next_idx() {
        let mut app: App<TestBackend> = App::default();
        app.free_idx = [false, true, false, true];
        let result = app.get_next_idx().unwrap();
        assert_eq!(1, result);
        app.free_idx = [false, false, false, false];
        assert!(app.get_next_idx().is_err());
    }
}
