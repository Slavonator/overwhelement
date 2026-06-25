use super::super::ScalingMode;

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
    /// Соотношение сторон одной дискреты на выходном буфере
    pub element_aspect_ratio: f32,
    /// Таблица перенаправления индексов шейдеров от объектов к пулу сцены
    pub shader_map: Vec<u32>,
    /// Угол поворота вьюпорта
    pub rotation_angle: f32,
    /// Смещение по X внутри выходного буфера (в пикселях). Если не задано, используется 0.
    pub buffer_offset_x: Option<u32>,
    /// Смещение по Y внутри выходного буфера (в пикселях). Если не задано, используется 0.
    pub buffer_offset_y: Option<u32>,
    /// Ширина области буфера, отведённой под этот вьюпорт (в пикселях). Если не задано, используется `settings.output_width`.
    pub buffer_width: Option<u32>,
    /// Высота области буфера, отведённой под этот вьюпорт (в пикселях). Если не задано, используется `settings.output_height`.
    pub buffer_height: Option<u32>,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            x: 0.0, 
            y: 0.0, 
            width: 0.0,
            height: 0.0,
            scaling_mode: ScalingMode::None,
            element_aspect_ratio: 1.0, 
            shader_map: Vec::new(), 
            rotation_angle: 0.0, 
            buffer_offset_x: None,
            buffer_offset_y: None,
            buffer_width: None,
            buffer_height: None,
        }
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

    /// Создаёт и возвращает экземпляр вьюпорта с заданными размерами и режимом масштабирования
    pub fn with_scaling(mut self, scaling_mode: ScalingMode) -> Self {
        self.scaling_mode = scaling_mode;
        self
    }

    /// Создаёт и возвращает экземпляр вьюпорта с заданными размерами и соотношением сторон дискреты
    pub fn new_with_aspect(x: f32, y: f32, width: f32, height: f32, element_aspect_ratio: f32) -> Self {
        Viewport { x, y, width, height, element_aspect_ratio, ..Default::default()}
    }

    /// Изменяет и возвращает экземпляр вьюпорта с заданным смещением выходного буфера 
    pub fn with_buffer_offset(mut self, x: u32, y: u32) -> Self {
        self.buffer_offset_x = Some(x);
        self.buffer_offset_y = Some(y);
        self
    }

    /// Изменяет и возвращает экземпляр вьюпорта с заданными размерами выходного буфера
    pub fn with_buffer_size(mut self, width: u32, height: u32) -> Self {
        self.buffer_width = Some(width);
        self.buffer_height = Some(height);
        self
    }
}
