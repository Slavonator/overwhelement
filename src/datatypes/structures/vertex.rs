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
    /// Создаёт и возвращает новый экземпляр вершины с заданными координатами, глубиной и текструрными координатами
    pub fn with_depth_uv(x: f32, y: f32, depth: f32, u: f32, v: f32) -> Self {
        Vertex { x, y, depth, u, v, ..Default::default()}
    }
}
