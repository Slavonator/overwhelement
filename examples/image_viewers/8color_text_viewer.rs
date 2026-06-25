use overwhelement::*;
use image::{open, RgbaImage};
use std::{env, rc::Rc};
use terminal_size::terminal_size;

// 8 базовых цветов
const ANSI_PALETTE_8: [[u8; 3]; 8] = [
    [0, 0, 0],       // 0 Black
    [128, 0, 0],     // 1 Red
    [0, 128, 0],     // 2 Green
    [128, 128, 0],   // 3 Yellow
    [0, 0, 128],     // 4 Blue
    [128, 0, 128],   // 5 Magenta
    [0, 128, 128],   // 6 Cyan
    [192, 192, 192], // 7 White
];

const BAYER: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

// Используемые цвета
const VARIANTS: [(char, f32, bool); 5] = [
    (' ', 0.0, false),          // 0% текста, 100% фона
    ('\u{2591}', 0.25, false),  // 25% текста, 75% фона (░)
    ('\u{2592}', 0.5, false),   // 50% текста, 50% фона (▒)
    ('\u{2591}', 0.25, true),   // 75% текста, 25% фона (инвертированный ░, так как ▓ у меня не работал)
    ('\u{2588}', 1.0, false),   // 100% текста, 0% фона (█)
];

// ---- Функции для ANSI-кодов (только 8 цветов) ----
fn ansi_fg(index: u8) -> String {
    format!("\x1b[{}m", 30 + index)
}

fn ansi_bg(index: u8) -> String {
    format!("\x1b[{}m", 40 + index)
}

// ---- Поиск оптимальной комбинации (только 8 цветов) ----
fn find_best_combination_8(target: [u8; 3]) -> (u8, u8, char) {
    let mut best_bg = 0;
    let mut best_fg = 0;
    let mut best_ch = ' ';
    let mut best_dist = f32::MAX;

    for bg in 0u8..8 {
        for fg in 0..8 {
            for (ch, ratio, swap) in VARIANTS {
                let (fg_actual, bg_actual) = if swap {
                    (bg, fg)
                } else {
                    (fg, bg)
                };

                let fg_color = ANSI_PALETTE_8[fg_actual as usize];
                let bg_color = ANSI_PALETTE_8[bg_actual as usize];

                let r = fg_color[0] as f32 * ratio + bg_color[0] as f32 * (1.0 - ratio);
                let g = fg_color[1] as f32 * ratio + bg_color[1] as f32 * (1.0 - ratio);
                let b = fg_color[2] as f32 * ratio + bg_color[2] as f32 * (1.0 - ratio);

                let d = (r - target[0] as f32).powi(2)
                      + (g - target[1] as f32).powi(2)
                      + (b - target[2] as f32).powi(2);

                if d < best_dist {
                    best_dist = d;
                    best_fg = fg_actual;
                    best_bg = bg_actual;
                    best_ch = ch;
                }
            }
        }
    }

    (best_fg, best_bg, best_ch)
}

struct TextureShader {
    image: RgbaImage,
}

impl ElementShader for TextureShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput {
        let u = input.uv.0.clamp(0.0, 1.0);
        let v = input.uv.1.clamp(0.0, 1.0);
        let w = self.image.width() as f32;
        let h = self.image.height() as f32;

        let x = u * (w - 1.0);
        let y = v * (h - 1.0);

        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.image.width() - 1);
        let y1 = (y0 + 1).min(self.image.height() - 1);

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let c00 = self.image.get_pixel(x0, y0);
        let c10 = self.image.get_pixel(x1, y0);
        let c01 = self.image.get_pixel(x0, y1);
        let c11 = self.image.get_pixel(x1, y1);

        let r = (1.0-fx)*(1.0-fy)*c00[0] as f32 + fx*(1.0-fy)*c10[0] as f32 
                + (1.0-fx)*fy*c01[0] as f32 + fx*fy*c11[0] as f32;
        let g = (1.0-fx)*(1.0-fy)*c00[1] as f32 + fx*(1.0-fy)*c10[1] as f32 
                + (1.0-fx)*fy*c01[1] as f32 + fx*fy*c11[1] as f32;
        let b = (1.0-fx)*(1.0-fy)*c00[2] as f32 + fx*(1.0-fy)*c10[2] as f32 
                + (1.0-fx)*fy*c01[2] as f32 + fx*fy*c11[2] as f32;

        ShaderOutput {
            color: [r.round() as u8, g.round() as u8, b.round() as u8, 255],
            luminance: None,
            object_id: None,
        }
    }
}

// Вычисляет смещение для центрирования вьюпорта в буфере с учётом соотношения сторон элемента
fn offset_to_center(buffer_width: u32, buffer_height: u32, vp_width: f32, vp_height: f32, aspect: f32) -> (u32, u32) {
    let out_w = buffer_width as f32;
    let out_h = buffer_height as f32;
    let abs_w = vp_width.abs();
    let abs_h = vp_height.abs() * aspect;

    let out_aspect = out_w / out_h;
    let vp_aspect = abs_w / abs_h;

    let scale_w: f32;
    let scale_h: f32;
    if vp_aspect > out_aspect { 
        let scale = out_w / abs_w;
        (scale_w, scale_h) = (scale, scale);
    } else {
        let scale = out_h / abs_h;
        (scale_w, scale_h) = (scale, scale);
    }

    let scaled_viewport_width = abs_w * scale_w;
    let scaled_viewport_height = abs_h * scale_h;

    let offset_x = ((out_w - scaled_viewport_width) / 2.0).ceil() as u32;
    let offset_y = ((out_h - scaled_viewport_height) / 2.0).ceil() as u32;
    (offset_x, offset_y)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: viewer [filename]");
        return;
    }
    let filename = &args[1];
    println!("Loading: '{}'", filename);
    let img = open(filename).expect("Failed to open image").into_rgba8();
    println!("Image loaded: {}x{}", img.width(), img.height());

    let width = img.width() as f32;
    let height = img.height() as f32;
    let element_aspect = 0.5; // соотношение сторон одного элемента (символа)

    // 1. Создаём сцену
    let mut scene = Scene::new();

    // 2. Добавляем шейдер в пул сцены
    let shader_idx = scene.shader_pool.add(Rc::new(TextureShader { image: img }));

    // 3. Получаем размеры терминала для вычисления смещения
    let (out_w, out_h) = terminal_size().expect("Unable to get output size");
    let buffer_width = out_w.0 as u32;
    let buffer_height = out_h.0 as u32 - 4; // учитываем отступ

    // Вычисляем смещение для центрирования
    let (offset_x, offset_y) = offset_to_center(buffer_width, buffer_height, width, height, element_aspect);

    // Создаём вьюпорт без полей выравнивания, но с явным смещением
    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        width,
        height,
        scaling_mode: ScalingMode::Contain, // масштабирование с сохранением пропорций
        element_aspect_ratio: element_aspect,
        shader_map: vec![shader_idx],
        rotation_angle: 0.0,
        buffer_offset_x: Some(offset_x),
        buffer_offset_y: Some(offset_y),
        buffer_width: None, // используем весь буфер
        buffer_height: None,
    };
    let vp_idx = scene.add_viewport(vp);

    // 4. Создаём вершины
    let v0 = scene.add_vertex(Vertex {
        x: 0.0, y: 0.0, depth: 0.0, u: 0.0, v: 0.0,
        ..Default::default()
    });
    let v1 = scene.add_vertex(Vertex {
        x: width, y: 0.0, depth: 0.0, u: 1.0, v: 0.0,
        ..Default::default()
    });
    let v2 = scene.add_vertex(Vertex {
        x: width, y: height, depth: 0.0, u: 1.0, v: 1.0,
        ..Default::default()
    });
    let v3 = scene.add_vertex(Vertex {
        x: 0.0, y: height, depth: 0.0, u: 0.0, v: 1.0,
        ..Default::default()
    });

    // 5. Создаём треугольники (с индексами вершин) и добавляем их в сцену
    let t1 = Triangle {
        id: 1,
        vertices: [v0, v1, v2],
        local_shader_id: 0,
    };
    let t2 = Triangle {
        id: 2,
        vertices: [v0, v2, v3],
        local_shader_id: 0,
    };
    let t1_idx = scene.add_triangle(t1);
    let t2_idx = scene.add_triangle(t2);

    // 6. Создаём плоскость, ссылающуюся на треугольники
    let plane = Plane {
        id: 0,
        triangles: vec![t1_idx, t2_idx],
        lines: Vec::new(),
        viewport_indices: vec![vp_idx],
    };
    scene.add_plane(plane);

    // 7. Настройки дискретизации
    let settings = Settings {
        output_width: buffer_width,
        output_height: buffer_height,
        background_color: [0, 0, 0],
        background_luminance: 0.0,
    };

    let buffer = discretize(&scene, &settings);

    // 8. Вывод в терминал с палитрой 8 цветов и дизерингом
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let elem = buffer.get(x, y).unwrap();
            if elem.object_id == EMPTY_OBJECT_ID {
                print!("\x1b[0m ");
                continue;
            }

            // Порог Байера
            let bayer = BAYER[(y % 4) as usize][(x % 4) as usize] / 16.0;
            let offset = (bayer - 0.5) * 10.0;

            let r = (elem.color[0] as f32 + offset).clamp(0.0, 255.0) as u8;
            let g = (elem.color[1] as f32 + offset).clamp(0.0, 255.0) as u8;
            let b = (elem.color[2] as f32 + offset).clamp(0.0, 255.0) as u8;

            let (fg, bg, ch) = find_best_combination_8([r, g, b]);
            print!("{}{}{}\x1b[0m", ansi_fg(fg), ansi_bg(bg), ch);
        }
        println!();
    }
    print!("\x1b[0m");
}