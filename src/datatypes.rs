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

// Трейты

/// Трейт, который должны реализовать шейдеры
pub trait ElementShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput;
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
        let idx = (y as u64 * self.width as u64 + x as u64) as usize;
        if x < self.width && y < self.height {
            Some(self.elements[idx])
        } else {
            None
        }
    }

    /// Возвращает изменяемую ссылку на дискрету в буфере по заданному индексу
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Element> {
        let idx = (y as u64 * self.width as u64 + x as u64) as usize;
        if x < self.width && y < self.height {
            Some(&mut self.elements[idx])
        } else {
            None
        }
    }

    /// Возвращает ссылку на весь буфер дискрет
    pub fn as_slice(&self) -> &[Element] {
        &self.elements
    }

    /// Возвращает буфер дискрет как итерируемый объект
    pub fn iter(&self) -> std::slice::Iter<'_, Element> {
        self.elements.iter()
    }

}

/// Видимая область плоскости
#[derive(Clone, Debug)]
pub struct Viewport {
    /// Позиция точки x на плоскости
    pub x: f32,
    /// Позиция точки y на плоскости
    pub y: f32,
    /// Ширина области видимости на плоскости
    pub width: f32,
    /// Высота области видимости на плоскости
    pub height: f32,
    /// Масштабирование, определяющее как вписать область видимости в выходной буфер
    pub scaling_mode: ScalingMode,
    /// Горизонтальное выравнивание геометрии на выходном буфере
    pub horizontal_alignment: HorizontalAlignment,
    /// Вертикальное выравнивание геометрии на выходном буфере
    pub vertical_alignment: VerticalAlignment,
    /// Соотношение сторон одной дискреты на выходном буфере
    pub element_aspect_ratio: f32,
    /// Таблица перенаправления индексов шейдеров от объектов к пулу сцены
    pub shader_map: Vec<u32>,
    /// Угол поворота вьюпорта
    pub rotation_angle: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            x: 0.0, 
            y: 0.0, 
            width: 0.0,
            height: 0.0,
            scaling_mode: ScalingMode::None, 
            horizontal_alignment: HorizontalAlignment::Center, 
            vertical_alignment: VerticalAlignment::Center, 
            element_aspect_ratio: 1.0, 
            shader_map: Vec::new(), 
            rotation_angle: 0.0 }
    }
}

impl Viewport {
    /// Создаёт и возвращает экземпляр вьюпорта с заданными размерами
    pub fn new(x: f32, y:f32, width: f32, height: f32) -> Self {
        Viewport { x, y, width, height, ..Default::default()}
    }

    /// Создаёт и возвращает экземпляр вьюпорта с заданными размерами и режимом масштабирования
    pub fn new_with_scaling(x: f32, y: f32, width: f32, height: f32, scaling_mode: ScalingMode) -> Self {
        Viewport { x, y, width, height, scaling_mode, ..Default::default()}
    }

    /// Создаёт и возвращает экземпляр вьюпорта с заданными размерами и соотношением сторон дискреты
    pub fn new_with_aspect(x: f32, y: f32, width: f32, height: f32, element_aspect_ratio: f32) -> Self {
        Viewport { x, y, width, height, element_aspect_ratio, ..Default::default()}
    }
}

/// Вершина геометрии на плоскости
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    /// Координата x вершины
    pub x: f32,
    /// Координата y вершины
    pub y: f32,
    /// Удалённость вершины от наблюдателя
    pub depth: f32,
    /// Текстурная координата u вершины
    pub u: f32,
    /// Текстурная координата v вершины
    pub v: f32,
    /// Нормали вершины
    pub normal: [f32; 3],
    /// Освещённость вершины
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
    /// Создаёт и возвращает новый экземпляр вершины с заданными координатами
    pub fn new(x: f32, y: f32) -> Self {
        Vertex { x, y, ..Default::default() }
    }
    /// Создаёт и возвращает новый экземпляр вершины с заданными координатами и глубиной
    pub fn with_depth(x: f32, y: f32, depth: f32) -> Self {
        Vertex { x, y, depth, ..Default::default() }
    }
    /// Создаёт и возвращает новый экземпляр вершины с заданными координатами и текстурными координатами
    pub fn with_uv(x: f32, y: f32, u: f32, v: f32) -> Self {
        Vertex { x, y, u, v, ..Default::default() }
    }
}

/// Треугольник, примитивный объект геометрии, имеющий площадь
#[derive(Clone, Debug)]
pub struct Triangle {
    /// Идентификатор объекта
    pub id: u32,
    /// Вершины объекта
    pub vertices: [Vertex; 3],
    /// Локальный индекс шейдера, интерпретируется через Viewport::shader_map
    pub local_shader_id: u32,
}

/// Линия, примитивный объект геометрии, не имеющий площади
#[derive(Clone, Debug)]
pub struct Line {
    /// Идентификатор объекта
    pub id: u32,
    /// Вершины объекта
    pub vertices: [Vertex; 2],
    /// Локальный индекс шейдера, интерпретируется через Viewport::shader_map
    pub local_shader_id: u32,
    /// Толщина линии
    pub thickness: f32,
}

/// Плоскость, контейнер геометрии
#[derive(Clone, Debug)]
pub struct Plane {
    /// Идентификатор плоскости
    pub id: u32,
    /// Список треугольников на плоскости
    pub triangles: Vec<Triangle>,
    /// Список линий на плоскости
    pub lines: Vec<Line>,
    /// Индексы в ViewportPool
    pub viewport_indices: Vec<u32>,
}

/// Сцена, контейнер высшего уровня, содержащий в себе всю информацию об изначальной сцене
#[derive(Clone)]
pub struct Scene {
    /// Список всех шейдеров
    pub shader_pool: ShaderPool,
    /// Список всех вьюпортов
    pub viewports: Vec<Viewport>,
    /// Список всех плоскостей
    pub planes: Vec<Plane>,
}


impl Scene {
    /// Создаёт и возвращает новый экземпляр сцены, с пустыми полями
    pub fn new() -> Self {
        Self{
            shader_pool: ShaderPool::new(),
            viewports: Vec::new(),
            planes: Vec::new(),
        }
    }
    
    /// Добавляет вьюпорт в список вьюпортов сцены
    pub fn add_viewport(&mut self, vp: Viewport) -> u32 {
        self.viewports.push(vp);
        (self.viewports.len() - 1) as u32
    }

    /// Добавляет плоскость в список плоскостей сцены
    pub fn add_plane(&mut self, plane: Plane) -> u32 {
        self.planes.push(plane);
        (self.planes.len() - 1) as u32
    }
}
