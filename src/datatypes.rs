use std::rc::Rc;

// Константы

/// Значение object_id для пустого элемента
pub const EMPTY_OBJECT_ID: u32 = u32::MAX;

// Перечисления

/// Выравнивание по горизонтали
#[derive(Copy, Clone, Debug)]
pub enum HorizontalAlignment {
    /// Выравнивание по левому краю
    Left,
    /// Выравнивание по центру
    Center,
    /// Выравнивание по правому краю
    Right,
}

/// Выравнивание по вертикали
#[derive(Copy, Clone, Debug)]
pub enum VerticalAlignment {
    /// Выравнивание по верхнему краю
    Top,
    /// Выравнивание по центру
    Center,
    /// Выравнивание по нижнему краю
    Bottom
}

/// Масштабирование, если размер выходного буфера и вьюпорта не соответствуют
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScalingMode {
    /// Растянуть до размеров выходного буфера, игнорируя пропорции
    Stretch,
    /// Вписать в выходной буфер с сохранением пропорций, пустые места заливаются фоном
    Contain,
    /// Заполнить всё с сохранением пропорций, обрезая выступающие части
    Cover,
    /// Не масштабировать, координаты 1:1
    None,
}

// Структуры

/// Настройки дискретизации
#[derive(Clone, Debug)]
pub struct Settings {
    /// Ширина выходного буфера
    pub output_width: u32,
    /// Высота выходного буфера
    pub output_height: u32,
    /// Цвет незанятых областей
    pub background_color: [u8; 3],
    /// Яркость незанятых областей
    pub background_luminance: f32,
}

/// Структура для передачи шейдерам
#[derive(Clone, Debug)]
pub struct ShaderInput<'a> {
    /// Текстурные координаты фрагмента
    pub uv: (f32, f32),
    /// Интерполированные, ненормализованные нормали фрагмента
    pub normal: [f32; 3],
    /// Интерполированная яркость фрагмента
    pub luminance: f32,
    /// Элемент позади фрагмента
    pub background_element: &'a Element,
    /// Глубина фрагмента
    pub fragment_depth: f32, 
    /// Слой фрагмента
    pub fragment_layer: u32,
    /// Идентификатор объекта, попавшего в фрагмент
    pub object_id: u32
}

/// Структура, возвращаемая шейдерами
#[derive(Clone, Debug)]
pub struct ShaderOutput {
    /// Цвет фрагмента
    pub color: [u8; 4],
    /// Яркость фрагмента
    pub luminance: Option<f32>,
    /// Идентификатор объекта, попавшего в фрагмент
    pub object_id: Option<u32>,
}

/// Трейт, который должны реализовать шейдеры
pub trait ElementShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput;
}

/// Шейдер, который не рисует объекты. Используется как fallback по умолчанию в ShaderPool
struct VoidShader;

impl ElementShader for VoidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput {color: [0,0,0,0], luminance: None, object_id: None}
    }
}

/// Пул шейдеров, которые используются при дискретизации
#[derive(Clone)]
pub struct ShaderPool {
    /// Шейдер, который используется при невалидных ссылках
    pub fallback: Rc<dyn ElementShader>,
    shaders: Vec<Rc<dyn ElementShader>>,
}

impl ShaderPool {
    /// Создаёт новый пул шейдеров, с VoidShader в качестве fallback
    pub fn new() -> Self {
        ShaderPool{
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

    /// Возвращает количество шейдеров
    pub fn len(&self) -> usize {
        self.shaders.len()
    }
}

/// Дискрета
#[derive(Copy, Clone, Debug)]
pub struct Element {
    /// Идентификатор объекта, попавшего в дискрету. Равен u32::MAX, если объекта нет
    pub object_id: u32,
    /// Удалённость объекта, попавшего в дискрету от наблюдателя. Равен f32::INFINITY, если объекта нет
    pub depth: f32,
    /// Слой объекта, попавшего в дискрету. Равен 0, если слоёв нет
    pub layer: u32,
    /// Цвет объекта, попавшего в дискрету.
    pub color: [u8; 3],
    /// Освещенность объекта, попавшего в дискрету
    pub luminance: f32,
}

/// Буфер дискрет
#[derive(Clone)]
pub struct ElementBuffer {
    /// Ширина буфера
    pub width: u32,
    /// Высота буфера
    pub height: u32,
    elements: Vec<Element>,
}

impl ElementBuffer {
    /// Создаёт и возвращает новый буфер дискрет заданного размера 
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

    /// Вовзвращает ссылку на дискрету в буфере по заданному индексу
    pub fn get(&self, x: u32, y: u32) -> Option<Element> {
        if x < self.width && y < self.height {
            Some(self.elements[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Возвращает изменяемую ссылку на дискрету в буфере по заданному индексу
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Element> {
        if x < self.width && y < self.height {
            Some(&mut self.elements[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Возвращает ссылку на весь буфер дискрет
    pub fn as_slice(&self) -> &[Element] {
        &self.elements
    }

    /// 
    pub fn iter(&self) -> std::slice::Iter<'_, Element> {
        self.elements.iter()
    }

}

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
    pub shader_map: Vec<u32>,
    pub rotation_angle: f32,
}


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


impl Scene {
    pub fn new() -> Self {
        Self{
            shader_pool: ShaderPool::new(),
            viewports: Vec::new(),
            planes: Vec::new(),
        }
    }
}
