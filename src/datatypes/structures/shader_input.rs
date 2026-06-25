use super::Element;

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
