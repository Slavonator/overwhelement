use crate::{buffer_writing::*, datatypes::*, internal_datatypes::*, viewport_process::*};


/// Отсекает треугольник относительно заданной области (width×height).
fn triangle_outside_buffer(tri: &Triangle, width: f32, height: f32) -> bool {
    let min_x = tri.vertices[0].x.min(tri.vertices[1].x).min(tri.vertices[2].x);
    let max_x = tri.vertices[0].x.max(tri.vertices[1].x).max(tri.vertices[2].x);
    let min_y = tri.vertices[0].y.min(tri.vertices[1].y).min(tri.vertices[2].y);
    let max_y = tri.vertices[0].y.max(tri.vertices[1].y).max(tri.vertices[2].y);
    max_x < 0.0 || min_x >= width || max_y < 0.0 || min_y >= height
}

fn line_outside_buffer(line: &Line, width: f32, height: f32) -> bool {
    let min_x = line.vertices[0].x.min(line.vertices[1].x);
    let max_x = line.vertices[0].x.max(line.vertices[1].x);
    let min_y = line.vertices[0].y.min(line.vertices[1].y);
    let max_y = line.vertices[0].y.max(line.vertices[1].y);
    let half = (line.thickness / 2.0).ceil();
    max_x + half < 0.0 || min_x - half >= width || max_y + half < 0.0 || min_y - half >= height
}

// ──── Основная функция дискретизации ───────────────────────────

/// Преобразует сцену из нескольких плоскостей с заданными параметрами в буфер дискрет
pub fn discretize(scene: &Scene, settings: &Settings) -> ElementBuffer {
    let mut buffer = ElementBuffer::new(
        settings.output_width,
        settings.output_height,
        settings.background_color,
        settings.background_luminance,
    );
    let mut transparent_fragments: Vec<TransparentFragment> = Vec::new();

    for (layer_index, plane) in scene.planes.iter().enumerate() {
        let layer = layer_index as u32;

        // Перебираем все вьюпорты, на которые ссылается эта плоскость
        for &vp_idx in &plane.viewport_indices {
            let vp = match scene.viewports.get(vp_idx as usize) {
                Some(v) => v,
                None => continue,
            };
            // Вычисляем трансформацию вьюпорта
            let (scale_x, scale_y, offset_x, offset_y) = compute_viewport_transform(settings, vp);
            // Определяем область вьюпорта в пикселях
            let clip_x = vp.buffer_offset_x.unwrap_or(0);
            let clip_y = vp.buffer_offset_y.unwrap_or(0);
            let clip_w = vp.buffer_width.filter(|&w| w > 0).unwrap_or(settings.output_width);
            let clip_h = vp.buffer_height.filter(|&h| h > 0).unwrap_or(settings.output_height);

            // Обрабатываем треугольники
            for tri in &plane.triangles {
                // Разрешаем глобальный индекс шейдера
                let global_shader_idx = vp.shader_map
                    .get(tri.local_shader_id as usize)
                    .copied()
                    .unwrap_or(u32::MAX); // fallback на VoidShader
                let shader = scene.shader_pool.get(global_shader_idx);

                // Клонируем треугольник и применяем вьюпорт к его вершинам
                let mut transformed_tri = tri.clone();
                for v in &mut transformed_tri.vertices {
                    apply_viewport(v, vp, scale_x, scale_y, offset_x, offset_y);
                }

                if triangle_outside_buffer(&transformed_tri, clip_w as f32, clip_h as f32) {
                    continue;
                }
                discretize_triangle(
                    &mut buffer,
                    &transformed_tri,
                    layer,
                    &*shader,
                    &mut transparent_fragments,
                    clip_x,
                    clip_y,
                    clip_w,
                    clip_h,
                );
            }

            // Обрабатываем линии (аналогично)
            for line in &plane.lines {
                let global_shader_idx = vp.shader_map
                    .get(line.local_shader_id as usize)
                    .copied()
                    .unwrap_or(u32::MAX);
                let shader = scene.shader_pool.get(global_shader_idx);

                let mut transformed_line = line.clone();
                for v in &mut transformed_line.vertices {
                    apply_viewport(v, vp, scale_x, scale_y, offset_x, offset_y);
                }

                if line_outside_buffer(&transformed_line, clip_w as f32, clip_h as f32) {
                    continue;
                }
                discretize_line(
                    &mut buffer,
                    &transformed_line,
                    layer,
                    &*shader,
                    &mut transparent_fragments,
                    clip_x,
                    clip_y,
                    clip_w,
                    clip_h,
                );
            }
        }
    }

    // Сортировка и смешивание прозрачных фрагментов
    transparent_fragments.sort_by(|a, b| {
        a.layer.cmp(&b.layer).then_with(|| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal))
    });

    for frag in &transparent_fragments {
        let background = &buffer.get(frag.x, frag.y).unwrap();
        // Прозрачный фрагмент видим, только если он находится на том же или более высоком слое
        // и его глубина МЕНЬШЕ (ближе) глубины фона.
        if frag.layer >= background.layer && frag.depth < background.depth {
            buffer.blend(frag.x, frag.y, frag.color, frag.luminance, frag.object_id);
        }
    }

    buffer
}
