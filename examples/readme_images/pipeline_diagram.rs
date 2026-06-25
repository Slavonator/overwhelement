use overwhelement::*;
use image::{ImageBuffer, Rgba};
use ab_glyph::{FontRef, PxScale};
use std::rc::Rc;

const W: u32 = 800;
const H: u32 = 240;
struct SolidShader {
    color: [u8; 4],
}

impl ElementShader for SolidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput {
            color: self.color,
            luminance: None,
            object_id: None,
        }
    }
}

fn main() {
    let vw = 80.0;
    let vh = 24.0;

    let mut scene = Scene::new();

    let block_shader = SolidShader { color: [50, 120, 220, 255] };
    let arrow_shader = SolidShader { color: [180, 180, 180, 255] };

    let block_idx = scene.shader_pool.add(Rc::new(block_shader));
    let arrow_idx = scene.shader_pool.add(Rc::new(arrow_shader));

    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        width: vw,
        height: vh,
        scaling_mode: ScalingMode::Cover,
        element_aspect_ratio: 1.0,
        shader_map: vec![block_idx, arrow_idx],
        rotation_angle: 0.0,
        buffer_offset_x: None,
        buffer_offset_y: None,
        buffer_width: None,
        buffer_height: None,
    };
    let vp_idx = scene.add_viewport(vp);

    let box_w = 16.0;
    let box_h = 8.0;
    let margin = 8.0;
    let y = (vh - box_h) / 2.0;
    let x1 = 8.0;
    let x2 = x1 + box_w + margin;
    let x3 = x2 + box_w + margin;

    let thickness = 4.0;

    let mut add_line = |x1: f32, y1: f32, x2: f32, y2: f32, shader_id: u32, thick: f32| -> u32 {
        let v1 = Vertex { x: x1, y: y1, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 1.0 };
        let v2 = Vertex { x: x2, y: y2, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 1.0 };
        let i1 = scene.add_vertex(v1);
        let i2 = scene.add_vertex(v2);
        let line = Line {
            id: 0,
            vertices: [i1, i2],
            local_shader_id: shader_id,
            thickness: thick,
        };
        scene.add_line(line)
    };

    let mut line_indices = Vec::new();

    let rect_lines = [
        (x1, y, x1 + box_w, y),
        (x1 + box_w, y, x1 + box_w, y + box_h),
        (x1 + box_w, y + box_h, x1, y + box_h),
        (x1, y + box_h, x1, y),
    ];
    for (x1, y1, x2, y2) in rect_lines {
        line_indices.push(add_line(x1, y1, x2, y2, block_idx, thickness));
    }

    let rect_lines = [
        (x2, y, x2 + box_w, y),
        (x2 + box_w, y, x2 + box_w, y + box_h),
        (x2 + box_w, y + box_h, x2, y + box_h),
        (x2, y + box_h, x2, y),
    ];
    for (x1, y1, x2, y2) in rect_lines {
        line_indices.push(add_line(x1, y1, x2, y2, block_idx, thickness));
    }

    let rect_lines = [
        (x3, y, x3 + box_w, y),
        (x3 + box_w, y, x3 + box_w, y + box_h),
        (x3 + box_w, y + box_h, x3, y + box_h),
        (x3, y + box_h, x3, y),
    ];
    for (x1, y1, x2, y2) in rect_lines {
        line_indices.push(add_line(x1, y1, x2, y2, block_idx, thickness));
    }

    let arrow_y = y + box_h / 2.0;
    let tip_w = 1.0;
    let tip_h = 1.0;

    let tip_x = x2 - (thickness / 10.0);
    line_indices.push(add_line(x1 + box_w, arrow_y, tip_x, arrow_y, arrow_idx, 3.0));
    line_indices.push(add_line(tip_x, arrow_y, tip_x - tip_h, arrow_y - tip_w, arrow_idx, 3.0));
    line_indices.push(add_line(tip_x, arrow_y, tip_x - tip_h, arrow_y + tip_w, arrow_idx, 3.0));

    let tip_x2 = x3 - (thickness / 10.0);
    line_indices.push(add_line(x2 + box_w, arrow_y, tip_x2, arrow_y, arrow_idx, 3.0));
    line_indices.push(add_line(tip_x2, arrow_y, tip_x2 - tip_h, arrow_y - tip_w, arrow_idx, 3.0));
    line_indices.push(add_line(tip_x2, arrow_y, tip_x2 - tip_h, arrow_y + tip_w, arrow_idx, 3.0));

    let plane = Plane {
        id: 0,
        triangles: Vec::new(),
        lines: line_indices,
        viewport_indices: vec![vp_idx],
    };

    scene.add_plane(plane);

    let settings = Settings {
        output_width: W,
        output_height: H,
        background_color: [240, 240, 240],
        background_luminance: 1.0,
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

    // Текст поверх изображения
    let words = ["провайдер", "дискретизатор", "интерпретатор"];
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let scale = PxScale { x: 20.0, y: 20.0 };
    let grey = Rgba([70u8, 70u8, 70u8, 255u8]);

    let scale_factor = 10.0;

    let centers = [
        (x1 + box_w / 2.0, y + box_h / 2.0),
        (x2 + box_w / 2.0, y + box_h / 2.0),
        (x3 + box_w / 2.0, y + box_h / 2.0),
    ];

    for (i, (cx, cy)) in centers.iter().enumerate() {
        let text = words[i];
        let px = (cx * scale_factor) as i32;
        let py = (cy * scale_factor) as i32;
        // Грубая оценка ширины текста
        let text_width = text.chars().count() as i32 * 10;
        let x_pos = px - text_width / 2;
        let y_pos = py - 12; // чуть выше центра
        imageproc::drawing::draw_text_mut(&mut img, grey, x_pos, y_pos, scale, &font, text);
    }

    img.save("pipeline.png").expect("Failed to save PNG with text");
    println!("Saved pipeline.png");
}