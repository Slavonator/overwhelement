use super::structures::{shader_input::ShaderInput, shader_output::ShaderOutput};

/// Трейт, который должны реализовать шейдеры
pub trait ElementShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput;
}
