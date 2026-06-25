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

    let mut scene = Scene::new();

    // Шейдер, который будет изображать плоскость
    let plane_shader = SolidShader {color: [130u8, 130u8, 230u8, 255u8]};

    // Шейдер, который будет изображать обводку плоскости
    let plane_outline = SolidShader {color: [90u8, 90u8, 190u8, 255u8]};

    // Шейдер, которй будет изображать вьюпорт
    let viewport_shader = SolidShader {color: [140u8, 230u8, 140u8, 255u8]};

    // Шейдер, которй будет изображать обводку вьюпорта
    let viewport_outline = SolidShader {color: [110u8, 190u8, 110u8, 255u8]};

    scene.shader_pool.add(Rc::new(plane_shader));
    scene.shader_pool.add(Rc::new(viewport_shader));
    scene.shader_pool.add(Rc::new(plane_outline));
    scene.shader_pool.add(Rc::new(viewport_outline));

    scene.vertices.push(Vertex::with_depth(1.0, 1.0, 1.0));
    scene.vertices.push(Vertex::with_depth(10.0, 1.0, 1.0));
    scene.vertices.push(Vertex::with_depth(14.0, 11.0, 1.0));
    scene.vertices.push(Vertex::with_depth(4.0, 11.0, 1.0));

    // Визуализируемая плоскость
    scene.triangles.push(
        Triangle { 
            id: 0,
            vertices: [0, 3, 2], 
            local_shader_id: 0,
        }
    );

    scene.triangles.push(
        Triangle { 
            id: 0,
            vertices: [0, 1, 2], 
            local_shader_id: 0,
        }
    );


    scene.vertices.push(Vertex::with_depth(1.0, 1.0, 0.9));
    scene.vertices.push(Vertex::with_depth(10.0, 1.0, 0.9));
    scene.vertices.push(Vertex::with_depth(14.0, 11.0, 0.9));
    scene.vertices.push(Vertex::with_depth(4.0, 11.0, 0.9));


    let outline_thickness = 4.0;

    scene.lines.push(Line {
        id: 10,
        vertices: [4, 5],
        local_shader_id: 2,
        thickness: outline_thickness,
    });
    scene.lines.push(Line {
        id: 11,
        vertices: [5, 6],
        local_shader_id: 2,
        thickness: outline_thickness,
    });
    scene.lines.push(Line {
        id: 12,
        vertices: [6, 7],
        local_shader_id: 2,
        thickness: outline_thickness,
    });
    scene.lines.push(Line {
        id: 13,
        vertices: [7, 4],
        local_shader_id: 2,
        thickness: outline_thickness,
    });

    // Визуализируемый вьюпорт

    scene.vertices.push(Vertex::with_depth(1.5, 1.25, 0.3));
    scene.vertices.push(Vertex::with_depth(3.15, 6.8, 0.3));
    scene.vertices.push(Vertex::with_depth(7.1, 6.8, 0.3));
    scene.vertices.push(Vertex::with_depth(5.0, 1.25, 0.3));

    scene.triangles.push(
        Triangle { 
            id: 1,
            vertices: [8, 9, 10], 
            local_shader_id: 1,
        }
    );
    scene.triangles.push(
        Triangle { 
            id: 1,
            vertices: [8, 10, 11], 
            local_shader_id: 1,
        }
    );

    scene.vertices.push(Vertex::with_depth(1.5, 1.25, 0.2));
    scene.vertices.push(Vertex::with_depth(3.15, 6.8, 0.2));
    scene.vertices.push(Vertex::with_depth(7.1, 6.8, 0.2));
    scene.vertices.push(Vertex::with_depth(5.0, 1.25, 0.2));


    scene.lines.push(Line {
        id: 10,
        vertices: [12, 13],
        local_shader_id: 3,
        thickness: outline_thickness,
    });
    scene.lines.push(Line {
        id: 11,
        vertices: [13, 14],
        local_shader_id: 3,
        thickness: outline_thickness,
    });
    scene.lines.push(Line {
        id: 12,
        vertices: [14, 15],
        local_shader_id: 3,
        thickness: outline_thickness,
    });
    scene.lines.push(Line {
        id: 13,
        vertices: [15, 12],
        local_shader_id: 3,
        thickness: outline_thickness,
    });

    // Плоскость
    let plane = Plane {
        id: 0,
        triangles: vec![0, 1, 2, 3],
        lines: vec![0, 1, 2, 3, 4, 5, 6, 7],
        viewport_indices: vec![0],
    };

    // Вьюпорт 
    let vp = Viewport{
        x: 15.0,
        y: 0.0,
        width: -15.0,
        height: 15.0,
        scaling_mode: ScalingMode::Contain,
        element_aspect_ratio: 1.0,
        shader_map: vec![0, 1, 2, 3],
        rotation_angle: 3.1415,
        buffer_offset_x: None,
        buffer_offset_y: None,
        buffer_width: None,
        buffer_height: None,
    };

    scene.viewports.push(vp);

    scene.planes.push(plane);

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
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");

    let scale = PxScale { x: 30.0, y: 30.0 };
    let black = Rgba([0u8, 0u8, 0u8, 255u8]);

    let plane_text_pos = (226_i32, 226_i32);
    let viewport_text_pos = (180_i32, 450_i32);

    draw_text_mut(&mut img, black, plane_text_pos.0, plane_text_pos.1, scale, &font, "Plane");
    draw_text_mut(&mut img, black, viewport_text_pos.0, viewport_text_pos.1, scale, &font, "Viewport");

    img.save("viewport.png").expect("Failed to save PNG with labels");

}

