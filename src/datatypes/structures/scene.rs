use super::ShaderPool;
use super::Viewport;
use super::Plane;
use super::Vertex;
use super::Triangle;
use super::Line;

/// Сцена, контейнер высшего уровня, содержащий в себе всю информацию об изначальной сцене
#[derive(Clone)]
pub struct Scene {
    /// Список всех шейдеров
    pub shader_pool: ShaderPool,
    /// Список всех вьюпортов
    pub viewports: Vec<Viewport>,
    /// Список всех плоскостей
    pub planes: Vec<Plane>,
    /// Список всех вершин
    pub vertices: Vec<Vertex>,
    /// Список всех треугольников
    pub triangles: Vec<Triangle>,
    /// Список всех линий
    pub lines: Vec<Line>,
}


impl Scene {
    /// Создаёт и возвращает новый экземпляр сцены, с пустыми полями
    pub fn new() -> Self {
        Self{
            shader_pool: ShaderPool::new(),
            viewports: Vec::new(),
            planes: Vec::new(),
            vertices: Vec::new(),
            triangles: Vec::new(),
            lines: Vec::new()
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

    /// Добавляет плоскость в список плоскостей сцены
    pub fn add_vertex(&mut self, vertex: Vertex) -> u32 {
        self.vertices.push(vertex);
        (self.vertices.len() - 1) as u32
    }

    /// Добавляет плоскость в список плоскостей сцены
    pub fn add_triangle(&mut self, triangle: Triangle) -> u32 {
        self.triangles.push(triangle);
        (self.triangles.len() - 1) as u32
    }

    /// Добавляет плоскость в список плоскостей сцены
    pub fn add_line(&mut self, line: Line) -> u32 {
        self.lines.push(line);
        (self.lines.len() - 1) as u32
    }
}