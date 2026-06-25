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
