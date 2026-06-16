use overwhelement::*;
use image::{ImageBuffer, Rgba};
use ab_glyph::{FontRef, PxScale};
use std::rc::Rc;

const W: u32 = 800;
const H: u32 = 240;

// Сплошной шейдер с фиксированным цветом и object_id
struct SolidShader {
    color: [u8; 4],
    object_id: u32,
}

impl ElementShader for SolidShader {
    fn shade(&self, _input: &ShaderInput) -> ShaderOutput {
        ShaderOutput {
            color: self.color,
            luminance: None,
            object_id: Some(self.object_id),
        }
    }
}

fn main() {
    let mut scene = Scene {
        shader_pool: ShaderPool::new(),
        viewports: Vec::new(),
        planes: Vec::new(),
    };

    // Шейдеры
    let block_shader = scene.shader_pool.add(Rc::new(SolidShader {
        color: [50, 120, 220, 255], // синий
        object_id: 1,
    }));
    let arrow_shader = scene.shader_pool.add(Rc::new(SolidShader {
        color: [180, 180, 180, 255], // серый
        object_id: 2,
    }));

    // Вьюпорт на весь экран
    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        width: W as f32,
        height: H as f32,
        scaling_mode: ScalingMode::Stretch,
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        element_aspect_ratio: 1.0,
        shader_map: vec![block_shader, arrow_shader],
        rotation_angle: 0.0,
    };
    let vp_idx = scene.viewports.len() as u32;
    scene.viewports.push(vp);

    // Параметры блоков
    let box_w = 160.0;
    let box_h = 80.0;
    let margin = 120.0;
    let y = (H as f32 - box_h) / 2.0;

    // Позиции блоков
    let x1 = 40.0;
    let x2 = x1 + box_w + margin;
    let x3 = x2 + box_w + margin;

    // Плоскость диаграммы
    let mut plane = Plane {
        id: 1,
        triangles: Vec::new(),
        lines: Vec::new(),
        viewport_indices: vec![vp_idx],
    };

    // Функция для создания линий рамки блока
    let mut add_rect = |x: f32, y: f32, w: f32, h: f32| {
        let right = x + w;
        let bottom = y + h;
        let lines = [
            (Vertex::new(x, y), Vertex::new(right, y)),
            (Vertex::new(right, y), Vertex::new(right, bottom)),
            (Vertex::new(right, bottom), Vertex::new(x, bottom)),
            (Vertex::new(x, bottom), Vertex::new(x, y)),
        ];
        for (start, end) in lines {
            plane.lines.push(Line {
                id: 1,
                vertices: [start, end],
                local_shader_id: 0, // block_shader
                thickness: 4.0,     // толстые линии для блоков
            });
        }
    };


    add_rect(x1, y, box_w, box_h);
    add_rect(x2, y, box_w, box_h);
    add_rect(x3, y, box_w, box_h);

    // Стрелки между блоками
    let arrow_y = y + box_h / 2.0;
    // Стрелка 1 → 2
    plane.lines.push(Line {
        id: 100,
        vertices: [
            Vertex::new(x1 + box_w, arrow_y),
            Vertex::new(x2 - 10.0, arrow_y),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    // Наконечник стрелки
    let tip = x2 - 10.0;
    plane.lines.push(Line {
        id: 101,
        vertices: [
            Vertex::new(tip, arrow_y),
            Vertex::new(tip - 12.0, arrow_y - 8.0),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    plane.lines.push(Line {
        id: 102,
        vertices: [
            Vertex::new(tip, arrow_y),
            Vertex::new(tip - 12.0, arrow_y + 8.0),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });

    // Стрелка 2 → 3
    plane.lines.push(Line {
        id: 200,
        vertices: [
            Vertex::new(x2 + box_w, arrow_y),
            Vertex::new(x3 - 10.0, arrow_y),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    let tip2 = x3 - 10.0;
    plane.lines.push(Line {
        id: 201,
        vertices: [
            Vertex::new(tip2, arrow_y),
            Vertex::new(tip2 - 12.0, arrow_y - 8.0),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    plane.lines.push(Line {
        id: 202,
        vertices: [
            Vertex::new(tip2, arrow_y),
            Vertex::new(tip2 - 12.0, arrow_y + 8.0),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });

    scene.planes.push(plane);

    // Настройки
    let settings = Settings {
        output_width: W,
        output_height: H,
        background_color: [240, 240, 240], // светлый фон
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
    img.save("pipeline.png").expect("Failed to save PNG");
    println!("Saved pipeline.png");

    let words = ["провайдер", "дискретизатор", "интерпретатор"];
    let font_data = include_bytes!("assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");

    let scale = PxScale { x: 20.0, y: 20.0 };
    let grey = Rgba([70u8, 70u8, 70u8, 255u8]);

    // Позиции центров блоков (те же, что использовались для рамок)
    let box_w = 160.0;
    let box_h = 80.0;
    let y = (H as f32 - box_h) / 2.0;
    let x1 = 40.0;
    let x2 = x1 + box_w + 120.0;
    let x3 = x2 + box_w + 120.0;

    let centers = [
        (x1 + box_w / 2.0, y + box_h / 2.0),
        (x2 + box_w / 2.0, y + box_h / 2.0),
        (x3 + box_w / 2.0, y + box_h / 2.0),
    ];

    let mut img = image::open("pipeline.png").expect("Failed to open PNG").to_rgba8();

    for (i, (cx, cy)) in centers.iter().enumerate() {
        let text = words[i];
        // Очень грубая оценка ширины текста: каждая буква примерно 10px при таком масштабе
        let text_width = text.chars().count() as i32 * 10;
        let x = (*cx as i32) - text_width / 2;
        let y = (*cy as i32) - 12; // чуть выше центра
        imageproc::drawing::draw_text_mut(&mut img, grey, x, y, scale, &font, text);
    }

    img.save("pipeline.png").expect("Failed to save PNG with text");
    println!("Saved pipeline.png with numbers");
}