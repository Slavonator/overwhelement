use crate::datatypes::*;

pub(crate) fn compute_viewport_transform(settings: &Settings, vp: &Viewport) -> (f32, f32, f32, f32) {
    let out_w = vp.buffer_width
        .filter(|&w| w > 0)
        .unwrap_or(settings.output_width) as f32;
    let out_h = vp.buffer_height
        .filter(|&h| h > 0)
        .unwrap_or(settings.output_height) as f32;

    let abs_w = if vp.width.abs() == 0.0 { 1.0 } else { vp.width.abs() };
    let abs_h = if vp.height.abs() == 0.0 { 1.0 } else { vp.height.abs() * vp.element_aspect_ratio };

    let out_aspect = out_w / out_h;
    let vp_aspect = abs_w / abs_h;

    let (base_scale_x, base_scale_y) = match vp.scaling_mode {
        ScalingMode::Stretch => (out_w / abs_w, out_h / abs_h),
        ScalingMode::None => (1.0, 1.0),
        ScalingMode::Contain | ScalingMode::Cover => {
            if (vp.scaling_mode == ScalingMode::Contain && vp_aspect > out_aspect)
                || (vp.scaling_mode == ScalingMode::Cover && vp_aspect <= out_aspect)
            {
                let scale = out_w / abs_w;
                (scale, scale)
            } else {
                let scale = out_h / abs_h;
                (scale, scale)
            }
        }
    };

    let scaled_w = abs_w * base_scale_x;
    let scaled_h = abs_h * base_scale_y;

    let offset_px_x = vp.buffer_offset_x.unwrap_or(0) as f32;
    let offset_px_y = vp.buffer_offset_y.unwrap_or(0) as f32;

    // Выравнивания больше нет, просто смещаем
    let offset_x_base = -vp.x * base_scale_x;
    let offset_y_base = -vp.y * base_scale_y;

    let (scale_x, offset_x) = if vp.width < 0.0 {
        (-base_scale_x, out_w - (offset_x_base + scaled_w) + offset_px_x)
    } else {
        (base_scale_x, offset_x_base + offset_px_x)
    };

    let (scale_y, offset_y) = if vp.height < 0.0 {
        (-base_scale_y, out_h - (offset_y_base + scaled_h) + offset_px_y)
    } else {
        (base_scale_y, offset_y_base + offset_px_y)
    };

    (scale_x, scale_y, offset_x, offset_y)
}

pub(crate) fn apply_viewport(
    vertex: &mut Vertex,
    vp: &Viewport,
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
) {
    if vp.element_aspect_ratio != 1.0 {
        vertex.y *= vp.element_aspect_ratio;
    }
    if vp.rotation_angle != 0.0 {
        let cx = vp.x + vp.width / 2.0;
        let cy = vp.y + vp.height / 2.0;
        let cos = vp.rotation_angle.cos();
        let sin = vp.rotation_angle.sin();
        let dx = vertex.x - cx;
        let dy = vertex.y - cy;
        vertex.x = cx + dx * cos - dy * sin;
        vertex.y = cy + dx * sin + dy * cos;
    }
    vertex.x = vertex.x * scale_x + offset_x;
    vertex.y = vertex.y * scale_y + offset_y;
}