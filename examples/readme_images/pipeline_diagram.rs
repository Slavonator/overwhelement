use::overwhelement::*;
use image::{ImageBuffer, Rgba};
use ab_glyph::{FontRef, PxScale};
use std::rc::Rc;

const W: u32 = 800;
const H: u32 = 240;

// Сплошной шейдер с фиксированным цветом и object_id
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
    
    let vw = 80;
    let vh = 24;


    let mut scene = Scene {
        shader_pool: ShaderPool::new(),
        viewports: Vec::new(),
        planes: Vec::new(),
    };

    
    // Шейдеры

    let block_shader = SolidShader {color: [50, 120, 220, 255]};
    
    let arrow_shader = SolidShader {color: [180, 180, 180, 255]};


    scene.shader_pool.add(Rc::new(block_shader));
    scene.shader_pool.add(Rc::new(arrow_shader));

    // Вьюпорт на весь экран
    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        width: vw as f32,
        height: vh as f32,
        scaling_mode: ScalingMode::Cover,
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        element_aspect_ratio: 1.0,
        shader_map: vec![0, 1],
        rotation_angle: 0.0,
        buffer_offset_x: None,
        buffer_offset_y: None,
        buffer_width: None,
        buffer_height: None,
    };

    scene.viewports.push(vp);

    // Параметры блоков
    let box_w = 16.0;
    let box_h = 8.0;
    let margin = 8.0;

    let y = (vh as f32 - box_h) / 2.0;

    // Позиции блоков
    let x1 = 8.0;
    let x2 = x1 + box_w + margin;
    let x3 = x2 + box_w + margin;

    // Плоскость диаграммы
    let mut plane = Plane {
        id: 1,
        triangles: Vec::new(),
        lines: Vec::new(),
        viewport_indices: vec![0],
    };

    let thickness = 4.0;

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
                local_shader_id: 0,
                thickness: thickness,
            });
        }
    };


    add_rect(x1, y, box_w, box_h);
    add_rect(x2, y, box_w, box_h);
    add_rect(x3, y, box_w, box_h);

    // Стрелки между блоками
    let arrow_y = y + box_h / 2.0;
    let tip_w = 1.0;
    let tip_h = 1.0;

    // Стрелка 1 → 2
    plane.lines.push(Line {
        id: 100,
        vertices: [
            Vertex::new(x1 + box_w, arrow_y),
            Vertex::new(x2 - (thickness / 10.0), arrow_y),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    plane.lines.push(Line {
        id: 101,
        vertices: [
            Vertex::new(x2 - (thickness / 10.0), arrow_y),
            Vertex::new(x2 - (thickness / 10.0) - tip_h, arrow_y - tip_w),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    plane.lines.push(Line {
        id: 102,
        vertices: [
            Vertex::new(x2 - (thickness / 10.0), arrow_y),
            Vertex::new(x2 - (thickness / 10.0) - tip_h, arrow_y + tip_w),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });

    // Стрелка 2 → 3
    plane.lines.push(Line {
        id: 200,
        vertices: [
            Vertex::new(x2 + box_w, arrow_y),
            Vertex::new(x3 - (thickness / 10.0), arrow_y),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    plane.lines.push(Line {
        id: 201,
        vertices: [
            Vertex::new(x3 - (thickness / 10.0), arrow_y),
            Vertex::new(x3 - (thickness / 10.0) - tip_h, arrow_y - tip_w),
        ],
        local_shader_id: 1,
        thickness: 3.0,
    });
    plane.lines.push(Line {
        id: 202,
        vertices: [
            Vertex::new(x3 - (thickness / 10.0), arrow_y),
            Vertex::new(x3 - (thickness / 10.0) - tip_h, arrow_y + tip_w),
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

    let words = ["провайдер", "дискретизатор", "интерпретатор"];
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");

    let scale = PxScale { x: 20.0, y: 20.0 };
    let grey = Rgba([70u8, 70u8, 70u8, 255u8]);

    // Позиции центров блоков (те же, что использовались для рамок)
    let box_w = 160.0;
    let box_h = 80.0;
    let y = (H as f32 - box_h) / 2.0;
    let x1 = 77.0;
    let x2 = x1 + box_w + 80.0;
    let x3 = x2 + box_w + 78.0;

    let centers = [
        (x1 + box_w / 2.0, y + box_h / 2.0),
        (x2 + box_w / 2.0, y + box_h / 2.0),
        (x3 + box_w / 2.0, y + box_h / 2.0),
    ];

    for (i, (cx, cy)) in centers.iter().enumerate() {
        let text = words[i];
        // Очень грубая оценка ширины текста: каждая буква примерно 10px при таком масштабе
        let text_width = text.chars().count() as i32 * 10;
        let x = (*cx as i32) - text_width / 2;
        let y = (*cy as i32) - 12; // чуть выше центра
        imageproc::drawing::draw_text_mut(&mut img, grey, x, y, scale, &font, text);
    }

    img.save("pipeline.png").expect("Failed to save PNG with text");
    println!("Saved pipeline.png");
}