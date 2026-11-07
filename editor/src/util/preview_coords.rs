use plottery_lib::{Rect, V2};

/// Maps a click on the preview `<img>` back to layer coordinates (cm).
///
/// The preview SVG's viewBox is the layer's bounding box with the y-axis flipped (see
/// `Layer::to_svg`), and the browser letterboxes it centred inside the element. Returns `None`
/// for clicks on the letterbox bars.
pub fn preview_click_to_layer(click_px: V2, element_px: V2, layer_bbox: Rect) -> Option<V2> {
    let content = layer_bbox.size();
    if content.x <= 0.0 || content.y <= 0.0 || element_px.x <= 0.0 || element_px.y <= 0.0 {
        return None;
    }
    let scale = (element_px.x / content.x).min(element_px.y / content.y);
    let rendered = content * scale;
    let offset = (element_px - rendered) / 2.0;
    let local = click_px - offset;
    if local.x < 0.0 || local.y < 0.0 || local.x > rendered.x || local.y > rendered.y {
        return None;
    }
    Some(V2::new(
        layer_bbox.bl().x + local.x / scale,
        layer_bbox.tr().y - local.y / scale,
    ))
}
