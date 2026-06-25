use super::super::ElementShader;
use super::ShaderInput;
use super::ShaderOutput;

/// Шейдер, который не рисует объекты. Используется как fallback по умолчанию в ShaderPool
/// Является приватным, так как используется только в ShaderPool
pub(super) struct VoidShader;

impl ElementShader for VoidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput {color: [0,0,0,0], luminance: None, object_id: None}
    }
}
