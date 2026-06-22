use crate::datatypes::*;


/// Вычисляет масштаб и смещение для отображения viewport -> output.
pub(crate) fn compute_viewport_transform(settings: &Settings, vp: &Viewport) -> (f32, f32, f32, f32) {
    let out_w = settings.output_width as f32;
    let out_h = settings.output_height as f32;

    // Абсолютные размеры для расчёта пропорций и масштаба
    let abs_w = if vp.width.abs() == 0.0 { 1.0 } else { vp.width.abs() };
    let abs_h = if vp.height.abs() == 0.0 { 1.0 } else { vp.height.abs() * vp.element_aspect_ratio};
    // Домножение на соотношение сторон дискрет выходного буфера


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

    // Размер в пикселях после масштабирования (всегда положительный)
    let scaled_w = abs_w * base_scale_x;
    let scaled_h = abs_h * base_scale_y;

    // Выравнивание считаем ДЛЯ НЕОТРАЖЁННОГО вьюпорта (как будто width и height положительны)
    let offset_x_base = match vp.horizontal_alignment {
        HorizontalAlignment::Right => -vp.x * base_scale_x,
        HorizontalAlignment::Center => -vp.x * base_scale_x + (out_w - scaled_w) / 2.0,
        HorizontalAlignment::Left => -vp.x * base_scale_x + (out_w - scaled_w),
    };

    let offset_y_base = match vp.vertical_alignment {
        VerticalAlignment::Top => -vp.y * base_scale_y,
        VerticalAlignment::Center => -vp.y * base_scale_y + (out_h - scaled_h) / 2.0,
        VerticalAlignment::Bottom => -vp.y * base_scale_y + (out_h - scaled_h),
    };

    let (scale_x, offset_x) = if vp.width < 0.0 {
        (
            -base_scale_x,
            out_w - (offset_x_base + scaled_w),
        )
    } else {
        (base_scale_x, offset_x_base)
    };

    // Аналогично для Y
    let (scale_y, offset_y) = if vp.height < 0.0 {
        (
            -base_scale_y,
            out_h - (offset_y_base + scaled_h),
        )
    } else {
        (base_scale_y, offset_y_base)
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
    // Трансформация геометрии под соотношение сторон выходного буфера
    if vp.element_aspect_ratio != 1.0 {
        vertex.y *= vp.element_aspect_ratio;
    }
    // Если есть поворот, вращаем вокруг центра вьюпорта
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
