use tui::layout::Rect;

pub fn get_inner_area(area: &Rect) -> Rect {
    let mut area_cloned = *area;
    area_cloned.width = area.width - 2;
    area_cloned.height = area.height - 2;
    area_cloned.x = area.x + 1;
    area_cloned.y = area.y + 1;
    area_cloned
}
