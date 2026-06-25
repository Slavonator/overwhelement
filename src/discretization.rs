// src/discretization.rs
use crate::{buffer_writing::*, datatypes::*, internal_datatypes::*, viewport_process::*};

/// Проверяет, находится ли треугольник полностью вне заданной области (в пикселях).
/// Принимает срез вершин и три индекса.
fn triangle_outside_buffer(
    vertices: &[Vertex],
    v0_idx: u32,
    v1_idx: u32,
    v2_idx: u32,
    width: f32,
    height: f32,
) -> bool {
    let v0 = &vertices[v0_idx as usize];
    let v1 = &vertices[v1_idx as usize];
    let v2 = &vertices[v2_idx as usize];
    let min_x = v0.x.min(v1.x).min(v2.x);
    let max_x = v0.x.max(v1.x).max(v2.x);
    let min_y = v0.y.min(v1.y).min(v2.y);
    let max_y = v0.y.max(v1.y).max(v2.y);
    max_x < 0.0 || min_x >= width || max_y < 0.0 || min_y >= height
}

/// Проверяет, находится ли линия полностью вне заданной области.
fn line_outside_buffer(
    vertices: &[Vertex],
    v0_idx: u32,
    v1_idx: u32,
    thickness: f32,
    width: f32,
    height: f32,
) -> bool {
    let v0 = &vertices[v0_idx as usize];
    let v1 = &vertices[v1_idx as usize];
    let min_x = v0.x.min(v1.x);
    let max_x = v0.x.max(v1.x);
    let min_y = v0.y.min(v1.y);
    let max_y = v0.y.max(v1.y);
    let half = (thickness / 2.0).ceil();
    max_x + half < 0.0 || min_x - half >= width || max_y + half < 0.0 || min_y - half >= height
}

/// Основная функция дискретизации
pub fn discretize(scene: &Scene, settings: &Settings) -> ElementBuffer {
    let mut buffer = ElementBuffer::new(
        settings.output_width,
        settings.output_height,
        settings.background_color,
        settings.background_luminance,
    );
    let mut transparent_fragments: Vec<TransparentFragment> = Vec::new();

    // Слои определяются порядком плоскостей (индекс в `scene.planes`)
    for (layer_index, plane) in scene.planes.iter().enumerate() {
        let layer = layer_index as u32;

        // Перебираем вьюпорты, привязанные к этой плоскости
        for &vp_idx in &plane.viewport_indices {
            let vp = match scene.viewports.get(vp_idx as usize) {
                Some(v) => v,
                None => continue,
            };

            // Вычисляем трансформацию для этого вьюпорта
            let (scale_x, scale_y, offset_x, offset_y) = compute_viewport_transform(settings, vp);

            // Определяем область отсечения (в пикселях)
            let clip_x = vp.buffer_offset_x.unwrap_or(0);
            let clip_y = vp.buffer_offset_y.unwrap_or(0);
            let clip_w = vp.buffer_width.filter(|&w| w > 0).unwrap_or(settings.output_width);
            let clip_h = vp.buffer_height.filter(|&h| h > 0).unwrap_or(settings.output_height);

            // --- Обработка треугольников ---
            for &tri_idx in &plane.triangles {
                let tri = &scene.triangles[tri_idx as usize];

                // Получаем глобальный индекс шейдера и сам шейдер
                let global_shader_idx = vp.shader_map
                    .get(tri.local_shader_id as usize)
                    .copied()
                    .unwrap_or(u32::MAX);
                let shader = scene.shader_pool.get(global_shader_idx);

                // Копируем вершины из пула
                let mut transformed_vertices = [
                    scene.vertices[tri.vertices[0] as usize],
                    scene.vertices[tri.vertices[1] as usize],
                    scene.vertices[tri.vertices[2] as usize],
                ];

                // Применяем трансформацию вьюпорта к каждой вершине
                for v in &mut transformed_vertices {
                    apply_viewport(v, vp, scale_x, scale_y, offset_x, offset_y);
                }

                // Отсечение: если треугольник полностью вне области, пропускаем
                if triangle_outside_buffer(
                    &transformed_vertices,
                    0, 1, 2,
                    clip_w as f32,
                    clip_h as f32,
                ) {
                    continue;
                }

                // Растеризация
                discretize_triangle(
                    &mut buffer,
                    &transformed_vertices,
                    0, 1, 2,          // индексы внутри нашего временного массива
                    tri.id,
                    layer,
                    &*shader,
                    &mut transparent_fragments,
                    clip_x,
                    clip_y,
                    clip_w,
                    clip_h,
                );
            }

            // --- Обработка линий ---
            for &line_idx in &plane.lines {
                let line = &scene.lines[line_idx as usize];

                let global_shader_idx = vp.shader_map
                    .get(line.local_shader_id as usize)
                    .copied()
                    .unwrap_or(u32::MAX);
                let shader = scene.shader_pool.get(global_shader_idx);

                let mut transformed_vertices = [
                    scene.vertices[line.vertices[0] as usize],
                    scene.vertices[line.vertices[1] as usize],
                ];

                for v in &mut transformed_vertices {
                    apply_viewport(v, vp, scale_x, scale_y, offset_x, offset_y);
                }

                if line_outside_buffer(
                    &transformed_vertices,
                    0, 1,
                    line.thickness,
                    clip_w as f32,
                    clip_h as f32,
                ) {
                    continue;
                }

                discretize_line(
                    &mut buffer,
                    &transformed_vertices,
                    0, 1,
                    line.id,
                    line.thickness,
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
        a.layer.cmp(&b.layer)
            .then_with(|| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal))
    });

    for frag in &transparent_fragments {
        let background = buffer.get(frag.x, frag.y).unwrap();
        if frag.layer >= background.layer && frag.depth < background.depth {
            buffer.blend(frag.x, frag.y, frag.color, frag.luminance, frag.object_id);
        }
    }

    buffer
}