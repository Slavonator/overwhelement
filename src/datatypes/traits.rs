use super::{ShaderInput, ShaderOutput};

/// Трейт, который должны реализовать шейдеры
pub trait ElementShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput;
}
