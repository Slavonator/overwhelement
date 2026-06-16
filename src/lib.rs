use std::rc::Rc;

/// Значение object_id для пустого элемента
pub const EMPTY_OBJECT_ID: u32 = u32::MAX;

// ──── Настройки вывода ─────────────────────────────────────────

#[derive(Copy, Clone, Debug)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScalingMode {
    /// Растянуть до размеров выходного буфера, игнорируя пропорции.
    Stretch,
    /// Вписать с сохранением пропорций, пустые места заливаются фоном.
    Contain,
    /// Заполнить всё с сохранением пропорций, обрезая выступающие части.
    Cover,
    /// Не масштабировать, координаты 1:1.
    None,
}

/// Настройки дискретизации.
#[derive(Clone, Debug)]
pub struct Settings {
    pub output_width: u32,
    pub output_height: u32,
    pub background_color: [u8; 3],
    pub background_luminance: f32,
}

// ──── Элементарный шейдер ───────────────────────────────────────
#[derive(Clone, Debug)]
pub struct ShaderInput<'a> {
    pub uv: (f32, f32),
    pub normal: [f32; 3],
    pub luminance: f32,
    pub background_element: &'a Element,
    pub fragment_depth: f32,
    pub fragment_layer: u32,
    pub object_id: u32,
}

#[derive(Clone, Debug)]
pub struct ShaderOutput {
    pub color: [u8; 4],
    pub luminance: Option<f32>,
    pub object_id: Option<u32>,
}

pub trait ElementShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput;
}

struct VoidShader;
impl ElementShader for VoidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput { color: [0,0,0,0], luminance: None, object_id: None }
    }
}

#[derive(Clone)]
pub struct ShaderPool {
    pub fallback: Rc<dyn ElementShader>,
    shaders: Vec<Rc<dyn ElementShader>>,
}

impl ShaderPool {
    pub fn new() -> Self {
        ShaderPool {
            fallback: Rc::new(VoidShader),
            shaders: Vec::new(),
        }
    }

    /// Добавляет шейдер в пул и возвращает его индекс (0-based).
    pub fn add(&mut self, shader: Rc<dyn ElementShader>) -> u32 {
        let idx = self.shaders.len() as u32;
        self.shaders.push(shader);
        idx
    }

    /// Удаляет последний добавленный шейдер и возвращает его.
    /// Возвращает `None`, если пул пуст.
    pub fn pop(&mut self) -> Option<Rc<dyn ElementShader>> {
        self.shaders.pop()
    }

    /// Получить шейдер по индексу. Если индекс невалидный, возвращает fallback.
    pub fn get(&self, index: u32) -> Rc<dyn ElementShader> {
        self.shaders
            .get(index as usize)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }

    pub fn len(&self) -> usize {
        self.shaders.len()
    }
}

// ──── Вьюпорт ─────────────────────────────────────────────────
#[derive(Clone, Debug)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub scaling_mode: ScalingMode,
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_alignment: VerticalAlignment,
    pub element_aspect_ratio: f32,
    /// Таблица: локальный shader_id (индекс) -> глобальный индекс в ShaderPool
    pub shader_map: Vec<u32>,
    pub rotation_angle: f32,
}

// ──── Геометрия ────────────────────────────────────────────────

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
    pub u: f32,
    pub v: f32,
    pub normal: [f32; 3],
    pub luminance: f32,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            x: 0.0,
            y: 0.0,
            depth: 0.0,
            u: 0.0,
            v: 0.0,
            normal: [0.0; 3],
            luminance: 1.0,
        }
    }
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Self {
        Vertex { x, y, ..Default::default() }
    }
    pub fn with_depth(x: f32, y: f32, depth: f32) -> Self {
        Vertex { x, y, depth, ..Default::default() }
    }
    pub fn with_uv(x: f32, y: f32, u: f32, v: f32) -> Self {
        Vertex { x, y, u, v, ..Default::default() }
    }
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub id: u32,
    pub vertices: [Vertex; 3],
    /// Локальный индекс шейдера, интерпретируется через Viewport::shader_map
    pub local_shader_id: u32,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub id: u32,
    pub vertices: [Vertex; 2],
    pub local_shader_id: u32,
    pub thickness: f32,
}

// ──── Плоскость (только геометрия и ссылки на вьюпорты) ────────
#[derive(Clone, Debug)]
pub struct Plane {
    pub id: u32,
    pub triangles: Vec<Triangle>,
    pub lines: Vec<Line>,
    /// Индексы в ViewportPool
    pub viewport_indices: Vec<u32>,
}

// ──── Сцена (контейнер высшего уровня) ─────────────────────────
#[derive(Clone)]
pub struct Scene {
    pub shader_pool: ShaderPool,
    pub viewports: Vec<Viewport>,
    pub planes: Vec<Plane>,
}

// ──── Элемент и буфер ──────────────────────────────────────────

#[derive(Copy, Clone, Debug)]
pub struct Element {
    pub object_id: u32,
    pub depth: f32,
    pub layer: u32,
    pub color: [u8; 3],
    pub luminance: f32,
}

#[derive(Clone)]
pub struct ElementBuffer {
    pub width: u32,
    pub height: u32,
    pub elements: Vec<Element>,
}

impl ElementBuffer {
    pub fn new(width: u32, height: u32, bg_color: [u8; 3], bg_luminance: f32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            elements: vec![
                Element {
                    object_id: EMPTY_OBJECT_ID,
                    depth: f32::INFINITY,
                    layer: 0,
                    color: bg_color,
                    luminance: bg_luminance,
                };
                size
            ],
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn blend(&mut self, x: u32, y: u32, src_color: [u8; 4], luminance: f32, object_id: u32) {
        let idx = self.index(x, y);
        let elem = &mut self.elements[idx];

        let src_a = src_color[3] as u32;
        let src_r = src_color[0] as u32;
        let src_g = src_color[1] as u32;
        let src_b = src_color[2] as u32;

        let dst_r = elem.color[0] as u32;
        let dst_g = elem.color[1] as u32;
        let dst_b = elem.color[2] as u32;

        let out_r = ((src_r * src_a + dst_r * (255 - src_a)) / 255) as u8;
        let out_g = ((src_g * src_a + dst_g * (255 - src_a)) / 255) as u8;
        let out_b = ((src_b * src_a + dst_b * (255 - src_a)) / 255) as u8;

        elem.color = [out_r, out_g, out_b];
        elem.luminance = luminance;
        if object_id != EMPTY_OBJECT_ID {
            elem.object_id = object_id;
        }
    }

    /// Возвращает копию элемента по координатам (x, y).
    /// Если координаты выходят за границы буфера, возвращает None.
    pub fn get(&self, x: u32, y: u32) -> Option<Element> {
        if x < self.width && y < self.height {
            Some(self.elements[self.index(x, y)])
        } else {
            None
        }
    }
}

// ──── Трансформация вьюпорта ────────────────────────────────────

/// Вычисляет масштаб и смещение для отображения viewport -> output.
fn compute_viewport_transform(settings: &Settings, vp: &Viewport) -> (f32, f32, f32, f32) {
    let out_w = settings.output_width as f32;
    let out_h = settings.output_height as f32;
    let vp_w = vp.width;
    let vp_h = vp.height;

    // Соотношение сторон с учётом element_aspect_ratio
    let out_aspect = out_w / out_h * vp.element_aspect_ratio;
    let vp_aspect = vp_w / vp_h;

    let (scale_x, scale_y) = match vp.scaling_mode {
        ScalingMode::Stretch => (out_w / vp_w, out_h / vp_h),
        ScalingMode::None => (1.0, 1.0),
        ScalingMode::Contain | ScalingMode::Cover => {
            if (vp.scaling_mode == ScalingMode::Contain && vp_aspect > out_aspect)
                || (vp.scaling_mode == ScalingMode::Cover && vp_aspect <= out_aspect)
            {
                let scale = out_w / vp_w;
                (scale, scale)
            } else {
                let scale = out_h / vp_h;
                (scale, scale)
            }
        }
    };

    let scaled_w = vp_w * scale_x;
    let scaled_h = vp_h * scale_y;

    let offset_x = match vp.horizontal_alignment {
        HorizontalAlignment::Left => -vp.x * scale_x,
        HorizontalAlignment::Center => -vp.x * scale_x + (out_w - scaled_w) / 2.0,
        HorizontalAlignment::Right => -vp.x * scale_x + (out_w - scaled_w),
    };

    let offset_y = match vp.vertical_alignment {
        VerticalAlignment::Top => -vp.y * scale_y,
        VerticalAlignment::Center => -vp.y * scale_y + (out_h - scaled_h) / 2.0,
        VerticalAlignment::Bottom => -vp.y * scale_y + (out_h - scaled_h),
    };

    (scale_x, scale_y, offset_x, offset_y)
}

fn apply_viewport(
    vertex: &mut Vertex,
    vp: &Viewport,
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
) {
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
    
    // Затем масштабирование и сдвиг (как раньше)
    vertex.x = vertex.x * scale_x + offset_x;
    vertex.y = vertex.y * scale_y + offset_y;
    if vp.element_aspect_ratio != 1.0 {
        vertex.y /= vp.element_aspect_ratio;
    }
}

/// Отсекает треугольник: возвращает true, если треугольник полностью вне буфера.
fn triangle_outside_buffer(tri: &Triangle, width: u32, height: u32) -> bool {
    let min_x = tri.vertices[0].x.min(tri.vertices[1].x).min(tri.vertices[2].x);
    let max_x = tri.vertices[0].x.max(tri.vertices[1].x).max(tri.vertices[2].x);
    let min_y = tri.vertices[0].y.min(tri.vertices[1].y).min(tri.vertices[2].y);
    let max_y = tri.vertices[0].y.max(tri.vertices[1].y).max(tri.vertices[2].y);
    max_x < 0.0 || min_x >= width as f32 || max_y < 0.0 || min_y >= height as f32
}

fn line_outside_buffer(line: &Line, width: u32, height: u32) -> bool {
    let min_x = line.vertices[0].x.min(line.vertices[1].x);
    let max_x = line.vertices[0].x.max(line.vertices[1].x);
    let min_y = line.vertices[0].y.min(line.vertices[1].y);
    let max_y = line.vertices[0].y.max(line.vertices[1].y);
    // Учитываем толщину
    let half = line.thickness * 0.5 + 0.5; // как в растеризации
    max_x + half < 0.0 || min_x - half >= width as f32 || max_y + half < 0.0 || min_y - half >= height as f32
}

// ──── Основная функция дискретизации ───────────────────────────

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

                // Отсечение невидимых треугольников
                if triangle_outside_buffer(&transformed_tri, settings.output_width, settings.output_height) {
                    continue;
                }

                // Растеризация
                rasterize_triangle(&mut buffer, &transformed_tri, layer, &*shader, &mut transparent_fragments);
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

                if line_outside_buffer(&transformed_line, settings.output_width, settings.output_height) {
                    continue;
                }

                rasterize_line(&mut buffer, &transformed_line, layer, &*shader, &mut transparent_fragments);
            }
        }
    }

    // Сортировка и смешивание прозрачных фрагментов
    transparent_fragments.sort_by(|a, b| {
        a.layer.cmp(&b.layer).then_with(|| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
    });

    for frag in &transparent_fragments {
        buffer.blend(frag.x, frag.y, frag.color, frag.luminance, frag.object_id);
    }

    buffer
}

// ──── Структура для отложенных прозрачных фрагментов ──────────

struct TransparentFragment {
    x: u32,
    y: u32,
    depth: f32,
    layer: u32,
    color: [u8; 4],
    luminance: f32,
    object_id: u32,
}

// ──── Растеризация (с учётом coverage для линий, без сглаживания треугольников) ──

fn rasterize_triangle(
    buffer: &mut ElementBuffer,
    tri: &Triangle,
    layer: u32,
    shader: &dyn ElementShader,
    transparent_fragments: &mut Vec<TransparentFragment>,
) {
    let v0 = &tri.vertices[0];
    let v1 = &tri.vertices[1];
    let v2 = &tri.vertices[2];

    let min_x = v0.x.min(v1.x).min(v2.x).floor() as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor() as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil() as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil() as i32;

    let min_x = min_x.max(0) as u32;
    let min_y = min_y.max(0) as u32;
    let max_x = (max_x.min(buffer.width as i32 - 1)) as u32;
    let max_y = (max_y.min(buffer.height as i32 - 1)) as u32;

    let area = edge_function(v0, v1, v2);
    if area == 0.0 {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = edge_function(v1, v2, &Vertex { x: px, y: py, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 0.0 });
            let w1 = edge_function(v2, v0, &Vertex { x: px, y: py, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 0.0 });
            let w2 = edge_function(v0, v1, &Vertex { x: px, y: py, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 0.0 });

            let inside = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if !inside {
                continue;
            }

            let w0 = w0 / area;
            let w1 = w1 / area;
            let w2 = w2 / area;

            let depth = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let u = w0 * v0.u + w1 * v1.u + w2 * v2.u;
            let v = w0 * v0.v + w1 * v1.v + w2 * v2.v;
            let normal = [
                w0 * v0.normal[0] + w1 * v1.normal[0] + w2 * v2.normal[0],
                w0 * v0.normal[1] + w1 * v1.normal[1] + w2 * v2.normal[1],
                w0 * v0.normal[2] + w1 * v1.normal[2] + w2 * v2.normal[2],
            ];
            let luminance = w0 * v0.luminance + w1 * v1.luminance + w2 * v2.luminance;

            let idx = buffer.index(x, y);
            let current_element = buffer.elements[idx];

            let input = ShaderInput {
                uv: (u, v),
                normal,
                luminance,
                background_element: &current_element,
                fragment_depth: depth,
                fragment_layer: layer,
                object_id: tri.id,
            };

            let output = shader.shade(&input);

            let alpha = output.color[3];
            if alpha == 0 {
                continue;
            }

            let final_object_id = output.object_id.unwrap_or(tri.id);
            let final_luminance = output.luminance.unwrap_or(input.luminance);

            if alpha == 255 {
                if layer > current_element.layer || (layer == current_element.layer && depth < current_element.depth) {
                    let elem = &mut buffer.elements[idx];
                    elem.object_id = final_object_id;
                    elem.depth = depth;
                    elem.layer = layer;
                    elem.color = [output.color[0], output.color[1], output.color[2]];
                    elem.luminance = final_luminance;
                }
            } else {
                transparent_fragments.push(TransparentFragment {
                    x,
                    y,
                    depth,
                    layer,
                    color: output.color,
                    luminance: final_luminance,
                    object_id: final_object_id,
                });
            }
        }
    }
}

fn rasterize_line(
    buffer: &mut ElementBuffer,
    line: &Line,
    layer: u32,
    shader: &dyn ElementShader,
    transparent_fragments: &mut Vec<TransparentFragment>,
) {
    let v0 = &line.vertices[0];
    let v1 = &line.vertices[1];

    let half_thickness = line.thickness * 0.5;
    let max_dist = half_thickness + 0.5;

    let min_x = v0.x.min(v1.x) - max_dist;
    let min_y = v0.y.min(v1.y) - max_dist;
    let max_x = v0.x.max(v1.x) + max_dist;
    let max_y = v0.y.max(v1.y) + max_dist;

    let min_x = (min_x.floor() as i32).max(0) as u32;
    let min_y = (min_y.floor() as i32).max(0) as u32;
    let max_x = (max_x.ceil() as i32).min(buffer.width as i32 - 1) as u32;
    let max_y = (max_y.ceil() as i32).min(buffer.height as i32 - 1) as u32;

    let dx = v1.x - v0.x;
    let dy = v1.y - v0.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let t = ((px - v0.x) * dx + (py - v0.y) * dy) / len_sq;
            let t_clamped = t.clamp(0.0, 1.0);
            let closest_x = v0.x + t_clamped * dx;
            let closest_y = v0.y + t_clamped * dy;
            let dist = ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();

            if dist > max_dist {
                continue;
            }

            let t_actual = t.clamp(0.0, 1.0);
            let depth = (1.0 - t_actual) * v0.depth + t_actual * v1.depth;
            let u = (1.0 - t_actual) * v0.u + t_actual * v1.u;
            let v = (1.0 - t_actual) * v0.v + t_actual * v1.v;
            let normal = [
                (1.0 - t_actual) * v0.normal[0] + t_actual * v1.normal[0],
                (1.0 - t_actual) * v0.normal[1] + t_actual * v1.normal[1],
                (1.0 - t_actual) * v0.normal[2] + t_actual * v1.normal[2],
            ];
            let luminance = (1.0 - t_actual) * v0.luminance + t_actual * v1.luminance;

            let idx = buffer.index(x, y);
            let current_element = buffer.elements[idx];

            let input = ShaderInput {
                uv: (u, v),
                normal,
                luminance,
                background_element: &current_element,
                fragment_depth: depth,
                fragment_layer: layer,
                object_id: line.id,
            };

            let output = shader.shade(&input);

            let alpha = output.color[3];
            if alpha == 0 {
                continue;
            }

            let final_object_id = output.object_id.unwrap_or(line.id);
            let final_luminance = output.luminance.unwrap_or(input.luminance);

            if alpha == 255 {
                if layer > current_element.layer || (layer == current_element.layer && depth < current_element.depth) {
                    let elem = &mut buffer.elements[idx];
                    elem.object_id = final_object_id;
                    elem.depth = depth;
                    elem.layer = layer;
                    elem.color = [output.color[0], output.color[1], output.color[2]];
                    elem.luminance = final_luminance;
                }
            } else {
                transparent_fragments.push(TransparentFragment {
                    x,
                    y,
                    depth,
                    layer,
                    color: output.color,
                    luminance: final_luminance,
                    object_id: final_object_id,
                });
            }
        }
    }
}

fn edge_function(a: &Vertex, b: &Vertex, c: &Vertex) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}