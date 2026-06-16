use std::rc::Rc;
use ab_glyph::{FontRef, PxScale};
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use::overwhelement::*;


const W: u32 = 800;
const H: u32 = 800;

struct SolidShader {
    color: [u8; 4],
}

impl ElementShader for SolidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput { color: self.color, luminance: None, object_id: None }
    }
}

fn main() {

    let settings = Settings {
        output_width: W,
        output_height: H,
        background_color: [240u8, 240u8, 240u8],
        background_luminance: 0.0,
    };

    let mut scene = Scene { 
        shader_pool: ShaderPool::new(),
        viewports: Vec::new(),
        planes: Vec::new(),
    };

    // Шейдер, который будет изображать плоскость
    let plane_shader = SolidShader {color: [130u8, 130u8, 230u8, 255u8]};

    // Шейдер, который будет изображать обводку плоскости
    let plane_outline = SolidShader { color: [90u8, 90u8, 190u8, 255u8]};

    // Шейдер, которй будет изображать вьюпорт
    let viewport_shader = SolidShader {color: [140u8, 230u8, 140u8, 255u8]};

    // Шейдер, которй будет изображать обводку вьюпорта
    let viewport_outline = SolidShader {color: [110u8, 190u8, 110u8, 255u8]};

    scene.shader_pool.add(Rc::new(plane_shader));
    scene.shader_pool.add(Rc::new(viewport_shader));
    scene.shader_pool.add(Rc::new(plane_outline));
    scene.shader_pool.add(Rc::new(viewport_outline));

    // Вьюпорт на весь экран
    let vp = Viewport{
        x: 15.0,
        y: 0.0,
        width: -15.0,
        height: 15.0,
        scaling_mode: ScalingMode::Contain,
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        element_aspect_ratio: 1.0,
        shader_map: vec![0, 1, 2, 3],
        rotation_angle: 3.1415,
    };

    scene.viewports.push(vp);

    // Плоскость
    let mut actual_plane = Plane {
        id: 0,
        triangles: Vec::new(),
        lines: Vec::new(),
        viewport_indices: vec![0],
    };

    // Треугольники

    let v1 = Vertex::with_depth(1.0, 1.0, 1.0);
    let v2 = Vertex::with_depth(10.0, 1.0, 1.0);
    let v3 = Vertex::with_depth(14.0, 11.0, 1.0);
    let v4 = Vertex::with_depth(4.0, 11.0, 1.0);

    // Визуализируемая плоскость
    actual_plane.triangles.push(
        Triangle { 
            id: 0,
            vertices: [v1, v4, v3], 
            local_shader_id: 0,
        }
    );
    actual_plane.triangles.push(
        Triangle { 
            id: 0,
            vertices: [v1, v2, v3], 
            local_shader_id: 0,
        }
    );

    let v1 = Vertex::with_depth(1.0, 1.0, 0.5);
    let v2 = Vertex::with_depth(10.0, 1.0, 0.5);
    let v3 = Vertex::with_depth(14.0, 11.0, 0.5);
    let v4 = Vertex::with_depth(4.0, 11.0, 0.5);

    let outline_thickness = 4.0;

    actual_plane.lines.push(Line {
        id: 10,
        vertices: [v1, v2],
        local_shader_id: 2,
        thickness: outline_thickness,
    });
    actual_plane.lines.push(Line {
        id: 11,
        vertices: [v2, v3],
        local_shader_id: 2,
        thickness: outline_thickness,
    });
    actual_plane.lines.push(Line {
        id: 12,
        vertices: [v3, v4],
        local_shader_id: 2,
        thickness: outline_thickness,
    });
    actual_plane.lines.push(Line {
        id: 13,
        vertices: [v4, v1],
        local_shader_id: 2,
        thickness: outline_thickness,
    });

    // Визуализируемый вьюпорт

    let v1 = Vertex::with_depth(1.5, 1.25, 0.3);
    let v2 = Vertex::with_depth(3.15, 6.8, 0.3);
    let v3 = Vertex::with_depth(7.1, 6.8, 0.3);
    let v4 = Vertex::with_depth(5.0, 1.25, 0.3);

    actual_plane.triangles.push(
        Triangle { 
            id: 1,
            vertices: [v1, v2, v3], 
            local_shader_id: 1,
        }
    );
    actual_plane.triangles.push(
        Triangle { 
            id: 1,
            vertices: [v1, v3, v4], 
            local_shader_id: 1,
        }
    );

    let v1 = Vertex::with_depth(1.5, 1.25, 0.1);
    let v2 = Vertex::with_depth(3.15, 6.8, 0.1);
    let v3 = Vertex::with_depth(7.1, 6.8, 0.1);
    let v4 = Vertex::with_depth(5.0, 1.25, 0.1);

    actual_plane.lines.push(Line {
        id: 10,
        vertices: [v1, v2],
        local_shader_id: 3,
        thickness: outline_thickness,
    });
    actual_plane.lines.push(Line {
        id: 11,
        vertices: [v2, v3],
        local_shader_id: 3,
        thickness: outline_thickness,
    });
    actual_plane.lines.push(Line {
        id: 12,
        vertices: [v3, v4],
        local_shader_id: 3,
        thickness: outline_thickness,
    });
    actual_plane.lines.push(Line {
        id: 13,
        vertices: [v4, v1],
        local_shader_id: 3,
        thickness: outline_thickness,
    });

    scene.planes.push(actual_plane);

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
    // Шрифт (положите файл .ttf в папку assets/ или используйте системный)
    let font_data = include_bytes!("assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");

    let scale = PxScale { x: 30.0, y: 30.0 };
    let black = Rgba([0u8, 0u8, 0u8, 255u8]);

    let plane_text_pos = (226_i32, 226_i32);
    let viewport_text_pos = (180_i32, 450_i32);

    draw_text_mut(&mut img, black, plane_text_pos.0, plane_text_pos.1, scale, &font, "Plane");
    draw_text_mut(&mut img, black, viewport_text_pos.0, viewport_text_pos.1, scale, &font, "Viewport");

    img.save("viewport.png").expect("Failed to save PNG with labels");

}

