use super::Element;
use super::super::{EMPTY_OBJECT_ID, VOID_DEPTH};

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
                    depth: VOID_DEPTH,
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
