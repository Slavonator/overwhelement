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
