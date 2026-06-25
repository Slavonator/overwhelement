/// Плоскость, контейнер геометрии
#[derive(Clone, Debug)]
pub struct Plane {
    /// Идентификатор плоскости
    pub id: u32,
    /// Список ссылок на треугольники на плоскости
    pub triangles: Vec<u32>,
    /// Список ссылок на лини на плоскости
    pub lines: Vec<u32>,
    /// Индексы в ViewportPool
    pub viewport_indices: Vec<u32>,
}

