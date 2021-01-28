use tui::layout::Rect;

pub fn get_inner_area(area: &Rect) -> Rect {
    let mut area_cloned = *area;
    area_cloned.width = area.width - 2;
    area_cloned.height = area.height - 2;
    area_cloned.x = area.x + 1;
    area_cloned.y = area.y + 1;
    area_cloned
}

#[cfg(test)]
mod tests {
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
}
