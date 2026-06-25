use super::super::ElementShader;
use super::void_shader::VoidShader;
use std::rc::Rc;

/// Пул шейдеров, которые используются при дискретизации
#[derive(Clone)]
pub struct ShaderPool {
    /// Шейдер, который используется при невалидных ссылках
    pub fallback: Rc<dyn ElementShader>,
    shaders: Vec<Rc<dyn ElementShader>>,
}

impl ShaderPool {
    /// Создаёт новый пул шейдеров, с VoidShader в качестве fallback
    pub fn new() -> Self {
        ShaderPool{
            fallback: Rc::new(VoidShader),
            shaders: Vec::new(),
        }
    }

    /// Добавляет шейдер в пул и возвращает его индекс (0-based).
    pub fn add(&mut self, shader: Rc<dyn ElementShader>) -> u32 {
        let idx = self.shaders.len() as u32;
        self.shaders.push(shader);
        idx
    }

    /// Удаляет последний добавленный шейдер и возвращает его.
    /// Возвращает `None`, если пул пуст.
    pub fn pop(&mut self) -> Option<Rc<dyn ElementShader>> {
        self.shaders.pop()
    }

    /// Получить шейдер по индексу. Если индекс невалидный, возвращает fallback.
    pub fn get(&self, index: u32) -> Rc<dyn ElementShader> {
        self.shaders
            .get(index as usize)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }

    /// Возвращает количество шейдеров
    pub fn len(&self) -> usize {
        self.shaders.len()
    }
}
