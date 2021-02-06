use std::collections::BTreeMap;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::key_event_wrapper::KeyEventWrapper;

pub mod event_area;
pub mod help;
pub mod search_condition_dialog;
pub mod search_info;
pub mod side_menu;
pub mod status_bar;
pub mod textbox;

#[async_trait]
pub trait Drawable<B>
where
    B: Backend,
{
    /// all components must be drawable
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect);

    /// handles input key event
    /// and returns if parent component should handle other events or not
    async fn handle_event(&mut self, event: KeyEvent) -> bool;

    /// push the key mappings for this component
    fn push_key_maps<'a>(
        &self,
        maps: &'a mut BTreeMap<KeyEventWrapper, String>,
    ) -> &'a mut BTreeMap<KeyEventWrapper, String> {
        maps
    }
}
