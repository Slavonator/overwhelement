/// Треугольник, примитивный объект геометрии, имеющий площадь
#[derive(Clone, Debug)]
pub struct Triangle {
    /// Идентификатор объекта
    pub id: u32,
    /// Вершины объекта
    pub vertices: [u32; 3],
    /// Локальный индекс шейдера, интерпретируется через Viewport::shader_map
    pub local_shader_id: u32,
}
