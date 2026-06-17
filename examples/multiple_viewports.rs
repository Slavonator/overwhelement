use std::rc::Rc;

use::overwhelement::*;

struct SolidShader {
    color: [u8; 4]
}

impl ElementShader for SolidShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput {
        ShaderOutput { color: self.color, luminance: None, object_id: None}
    }
}

fn make_outline(points: Vec<[f32; 2]>) -> Option<Vec<[Vertex; 2]>> {
    if points.len() < 3 {
        return None;
    }

    let mut lines: Vec<[Vertex; 2]> = Vec::new();

    // Проходим по всем соседним парам (0-1, 1-2, 2-3, ...)
    for window in points.windows(2) {
        let v0 = Vertex::new(window[0][0], window[0][1]);
        let v1 = Vertex::new(window[1][0], window[1][1]);
        lines.push([v0, v1]);
    }

    // Замыкаем контур: последняя точка -> первая точка
    let first = points[0];
    let last = points[points.len() - 1];
    lines.push([
        Vertex::new(last[0], last[1]),
        Vertex::new(first[0], first[1]),
    ]);

    Some(lines)
}


fn main() {

    let mut scene = Scene {
        shader_pool: ShaderPool::new(),
        viewports: Vec::new(),
        planes: Vec::new(),
    };

    let plane_color = SolidShader{
        color: [130, 130, 230, 255]
    };

    let plane_outline = SolidShader{
        color: [90, 90, 190, 255]
    };

    scene.shader_pool.add(Rc::new(plane_color));
    scene.shader_pool.add(Rc::new(plane_outline));

}