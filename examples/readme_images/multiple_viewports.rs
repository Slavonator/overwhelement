use std::rc::Rc;
use image::{ImageBuffer, Rgba};
use::overwhelement::*;

struct SolidShader {
    color: [u8; 4]
}

impl ElementShader for SolidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput { color: self.color, luminance: None, object_id: None}
    }
}

const W: u32 = 800;
const H: u32 = 400;


fn make_outline(points: Vec<[f32; 2]>, depth: f32) -> Option<Vec<[Vertex; 2]>> {
    if points.len() < 3 {
        return None;
    }

    let mut lines: Vec<[Vertex; 2]> = Vec::new();

    for window in points.windows(2) {
        let v0 = Vertex::with_depth(window[0][0], window[0][1], depth);
        let v1 = Vertex::with_depth(window[1][0], window[1][1], depth);
        lines.push([v0, v1]);
    }

    let first = points[0];
    let last = points[points.len() - 1];
    lines.push([
        Vertex::with_depth(last[0], last[1], depth),
        Vertex::with_depth(first[0], first[1], depth),
    ]);

    Some(lines)
}


fn main() {

    let mut scene = Scene {
        shader_pool: ShaderPool::new(),
        viewports: Vec::new(),
        planes: Vec::new(),
    };

    // Шейдер для фоновой плоскости
    let plane_color = SolidShader{
        color: [130, 130, 230, 255]
    };

    // Шейдер для обводки
    let plane_outline = SolidShader{
        color: [90, 90, 190, 255]
    };

    // Два шейдера для внутренних треугольников
    let inner_triangle1_color = SolidShader {
        color: [200, 50, 50, 255]   // красный
    };
    let inner_triangle2_color = SolidShader {
        color: [50, 200, 50, 255]   // зелёный
    };

    let vp_outline1 = SolidShader{
        color: [90, 190, 190, 255]
    };

        let vp_outline2 = SolidShader{
        color: [60, 60, 60, 255]
    };

    scene.shader_pool.add(Rc::new(plane_color));
    scene.shader_pool.add(Rc::new(plane_outline));
    scene.shader_pool.add(Rc::new(inner_triangle1_color));
    scene.shader_pool.add(Rc::new(inner_triangle2_color));
    scene.shader_pool.add(Rc::new(vp_outline1));
    scene.shader_pool.add(Rc::new(vp_outline2));

    let plane = Plane{
        id: 0,
        triangles: Vec::new(),
        lines: Vec::new(),
        viewport_indices: vec![0],
    };

    scene.planes.push(plane);

    // Вьюпорт на весь экран (с небольшим отступом и поворотом)
    let vp = Viewport{
        x: 20.0,
        y: 0.0,
        width: -20.0,
        height: 20.0,
        scaling_mode: ScalingMode::Contain,
        horizontal_alignment: HorizontalAlignment::Left,
        vertical_alignment: VerticalAlignment::Center,
        element_aspect_ratio: 1.0,
        shader_map: vec![0, 1, 2, 3, 4, 5],
        rotation_angle: 3.1415,
    };

    scene.viewports.push(vp);

    let points: Vec<[f32; 2]> = vec!([1.0, 1.0], [2.0, 19.0], [18.0, 19.0], [17.0, 1.0]);

    // Два треугольника, образующие фоновую плоскость
    scene.planes[0].triangles.push(Triangle {
        id: 0,
        vertices: [
            Vertex::with_depth(1.0, 1.0, 1.5),
            Vertex::with_depth(2.0, 19.0, 1.5),
            Vertex::with_depth(18.0, 19.0, 1.5),
        ],
        local_shader_id: 0,
    });

    scene.planes[0].triangles.push(Triangle {
        id: 0,
        vertices: [
            Vertex::with_depth(1.0, 1.0, 1.5),
            Vertex::with_depth(17.0, 1.0, 1.5),
            Vertex::with_depth(18.0, 19.0, 1.5),
        ],
        local_shader_id: 0,
    });

    // Внутренние треугольники (другого цвета)
    scene.planes[0].triangles.push(Triangle {
        id: 0,
        vertices: [
            Vertex::with_depth(3.0, 6.0, 1.4),
            Vertex::with_depth(4.0, 15.0, 1.4),
            Vertex::with_depth(13.0, 14.0, 1.4),
        ],
        local_shader_id: 2,   // красный шейдер
    });

    scene.planes[0].triangles.push(Triangle {
        id: 0,
        vertices: [
            Vertex::with_depth(6.0, 4.0, 1.3),
            Vertex::with_depth(13.0, 4.0, 1.3),
            Vertex::with_depth(11.0, 16.0, 1.3),
        ],
        local_shader_id: 3,   // зелёный шейдер
    });

    // Обводка исходного контура
    let lines = make_outline(points, 1.0).unwrap();
    let thickness = 5.0;

    for line in lines {
        scene.planes[0].lines.push(Line {
            id: 0,
            vertices: line,
            local_shader_id: 1,
            thickness: thickness,
        });
    }

    let points: Vec<[f32; 2]> = vec!([3.0, 6.0], [3.0, 8.0], [6.0, 8.0], [6.0, 6.0]);

    let lines = make_outline(points, 1.0).unwrap();
    let thickness = 5.0;

    for line in lines {
        scene.planes[0].lines.push(Line {
            id: 0,
            vertices: line,
            local_shader_id: 4,
            thickness: thickness,
        });
    }


    let points: Vec<[f32; 2]> = vec!([7.0, 7.0], [7.0, 9.0], [10.0, 9.0], [10.0, 7.0]);

    let lines = make_outline(points, 1.0).unwrap();
    let thickness = 5.0;

    for line in lines {
        scene.planes[0].lines.push(Line {
            id: 0,
            vertices: line,
            local_shader_id: 5,
            thickness: thickness,
        });
    }




    let settings = Settings {
        output_width: W,
        output_height: H,
        background_color: [240u8, 240u8, 240u8],
        background_luminance: 0.0,
    };

    // Дискретизация
    let buffer = discretize(&scene, &settings);

    // Сохранение в PNG
    let mut img = ImageBuffer::new(W, H);
    for y in 0..H {
        for x in 0..W {
            let elem = buffer.get(x, y).unwrap();
            let pixel = if elem.object_id == EMPTY_OBJECT_ID {
                Rgba([settings.background_color[0], settings.background_color[1], settings.background_color[2], 255])
            } else {
                Rgba([elem.color[0], elem.color[1], elem.color[2], 255])
            };
            img.put_pixel(x, y, pixel);
        }
    }

    img.save("multiple_viewport.png").expect("Failed to save PNG with labels");
}