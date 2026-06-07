use bevy::prelude::*;

/// Scroll a vertical list so the item at `item_top` with `item_height` (physical px from
/// the top of the scrollable content) stays inside the viewport.
pub fn scroll_to_show_range_y(
    list_computed: &ComputedNode,
    scroll_position: &mut ScrollPosition,
    item_top: f32,
    item_height: f32,
) {
    let scale = list_computed.inverse_scale_factor();
    let viewport_h = list_computed.content_box().height();
    let max_offset = (list_computed.content_size() - list_computed.size()).max(Vec2::ZERO) * scale;

    let mut scroll_physical = scroll_position.y / scale;
    if item_top < scroll_physical {
        scroll_physical = item_top;
    } else if item_top + item_height > scroll_physical + viewport_h {
        scroll_physical = item_top + item_height - viewport_h;
    }

    scroll_position.y = (scroll_physical * scale).clamp(0.0, max_offset.y);
}

/// Walk direct `children` in order and return the top offset and height of the first match.
pub fn child_range_where(
    children: &Children,
    child_query: &Query<&ComputedNode>,
    list_computed: &ComputedNode,
    row_gap_logical: f32,
    mut is_match: impl FnMut(Entity) -> bool,
    mut trailing_spacing_logical: impl FnMut(Entity) -> f32,
) -> Option<(f32, f32)> {
    let scale = list_computed.inverse_scale_factor();
    let row_gap = row_gap_logical / scale;
    let mut offset = list_computed.padding.min_inset.y;

    for child in children.iter() {
        let Ok(computed) = child_query.get(child) else {
            continue;
        };

        if is_match(child) {
            return Some((offset, computed.size().y));
        }

        offset += computed.size().y + row_gap + trailing_spacing_logical(child) / scale;
    }

    None
}
