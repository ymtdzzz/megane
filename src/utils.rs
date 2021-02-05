use std::collections::BTreeMap;

use tui::layout::Rect;

use crate::key_event_wrapper::KeyEventWrapper;

pub fn get_inner_area(area: &Rect) -> Rect {
    let mut area_cloned = *area;
    area_cloned.width = area.width - 2;
    area_cloned.height = area.height - 2;
    area_cloned.x = area.x + 1;
    area_cloned.y = area.y + 1;
    area_cloned
}

pub fn key_maps_stringify(maps: &BTreeMap<KeyEventWrapper, String>) -> String {
    let mut datas = vec![];
    for (k, v) in maps.iter() {
        let key = k.to_string();
        datas.push(format!("{}: {}", key, v));
    }
    datas.join("/")
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    #[test]
    fn test_get_inner_area() {
        let rect = Rect {
            width: 100,
            height: 100,
            x: 0,
            y: 0,
            ..Default::default()
        };
        let expected = Rect {
            width: 98,
            height: 98,
            x: 1,
            y: 1,
            ..Default::default()
        };
        assert_eq!(expected, get_inner_area(&rect));
    }

    #[test]
    fn test_key_maps_string() {
        let mut input: BTreeMap<KeyEventWrapper, String> = BTreeMap::new();
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            String::from("test description 1"),
        );
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            String::from("test description 2"),
        );
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            String::from("test description 3"),
        );
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            String::from("test description 4"),
        );
        let result = key_maps_stringify(&input);
        let expected = String::from(
            "BackSpace: test description 3/C: test description 4/C+Ctrl: test description 1/SPC: test description 2",
        );
        assert_eq!(expected, result);
    }
}
