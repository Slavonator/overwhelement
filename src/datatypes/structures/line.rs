/// Линия, примитивный объект геометрии, не имеющий площади
#[derive(Clone, Debug)]
pub struct Line {
    /// Идентификатор объекта
    pub id: u32,
    /// Ссылка на вершины объекта
    pub vertices: [u32; 2],
    /// Локальный индекс шейдера, интерпретируется через Viewport::shader_map
    pub local_shader_id: u32,
    /// Толщина линии
    pub thickness: f32,
}