use overwhelement::*;
use image::{open, RgbaImage};
use std::{env, rc::Rc};
use terminal_size::{terminal_size};

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

// ---- 3. Функции для ANSI-кодов (только 8 цветов) ----
fn ansi_fg(index: u8) -> String {
    // index должен быть от 0 до 7
    format!("\x1b[{}m", 30 + index)
}

fn ansi_bg(index: u8) -> String {
    format!("\x1b[{}m", 40 + index)
}

// ---- 4. Поиск оптимальной комбинации (только 8 цветов) ----
fn find_best_combination_8(target: [u8; 3]) -> (u8, u8, char) {
    let mut best_bg = 0;
    let mut best_fg = 0;
    let mut best_ch = ' ';
    let mut best_dist = f32::MAX;

    // Перебираем 8 фонов × 8 текстов × 5 вариантов смешивания
    for bg in 0u8..8 {
        for fg in 0..8 {
            for (ch, ratio, swap) in VARIANTS {
                // Определяем фактические индексы с учётом swap
                let (fg_actual, bg_actual) = if swap {
                    (bg, fg)
                } else {
                    (fg, bg)
                };

                let fg_color = ANSI_PALETTE_8[fg_actual as usize];
                let bg_color = ANSI_PALETTE_8[bg_actual as usize];

                // Смешивание в линейном RGB
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
        
        // Преобразуем в координаты пикселей с плавающей точкой
        let x = u * (w - 1.0);
        let y = v * (h - 1.0);
        
        // Находим четыре ближайших пикселя
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.image.width() - 1);
        let y1 = (y0 + 1).min(self.image.height() - 1);
        
        // Дробные части для весов
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;
        
        let c00 = self.image.get_pixel(x0, y0);
        let c10 = self.image.get_pixel(x1, y0);
        let c01 = self.image.get_pixel(x0, y1);
        let c11 = self.image.get_pixel(x1, y1);
        
        // Билинейная интерполяция
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

    let mut shaders = ShaderPool::new();
    
    let width = img.width() as f32;
    let height = img.height() as f32;

    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        width: width,
        height: height,
        scaling_mode: ScalingMode::Contain,  
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        element_aspect_ratio: 0.5,      
        shader_map: vec![0],
        rotation_angle: 0.0,
    };
    
    shaders.add(Rc::new(TextureShader { image: img }));

    // Плоскость с двумя треугольниками (UV от 0 до 1)
    let mut plane = Plane {
        id: 0,
        triangles: Vec::new(),
        lines: Vec::new(),
        viewport_indices: vec![0],
    };

    plane.triangles.push(Triangle {
        id: 1,
        vertices: [
            Vertex { x: 0.0, y: 0.0, depth: 0.0, u: 0.0, v: 0.0, ..Default::default() },
            Vertex { x: width, y: 0.0, depth: 0.0, u: 1.0, v: 0.0, ..Default::default() },
            Vertex { x: width, y: height, depth: 0.0, u: 1.0, v: 1.0, ..Default::default() },
        ],
        local_shader_id: 0,
    });
    plane.triangles.push(Triangle {
        id: 2,
        vertices: [
            Vertex { x: 0.0, y: 0.0, depth: 0.0, u: 0.0, v: 0.0, ..Default::default() },
            Vertex { x: width, y: height, depth: 0.0, u: 1.0, v: 1.0, ..Default::default() },
            Vertex { x: 0.0, y: height, depth: 0.0, u: 0.0, v: 1.0, ..Default::default() },
        ],
        local_shader_id: 0,
    });

    // Сцена
    let scene = Scene {
        shader_pool: shaders,
        viewports: vec![vp],
        planes: vec![plane],
    };

    let (out_w , out_h) = terminal_size().expect("Unable to get output size");


    let settings = Settings {
        output_width: out_w.0 as u32,
        output_height: out_h.0 as u32 - 4,
        background_color: [0, 0, 0],
        background_luminance: 0.0,
    };

    let buffer = discretize(&scene, &settings);

    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let elem = buffer.get(x, y).unwrap();
            if elem.object_id == EMPTY_OBJECT_ID {
                print!("\x1b[0m ");
                continue;
            }

            // Порог Байера (0..1)
            let bayer = BAYER[(y % 4) as usize][(x % 4) as usize] / 16.0;
            // Смещение: от -0.5 до +0.5, умножаем на 20 для заметного эффекта
            let offset = (bayer - 0.5) * 10.0; 

            let r = (elem.color[0] as f32 + offset).clamp(0.0, 255.0) as u8;
            let g = (elem.color[1] as f32 + offset).clamp(0.0, 255.0) as u8;
            let b = (elem.color[2] as f32 + offset).clamp(0.0, 255.0) as u8;

            let (fg, bg, ch) = find_best_combination_8([r, g, b]);
            print!("{}{}{}\x1b[0m", ansi_fg(fg), ansi_bg(bg), ch);
        }
        println!();
    }
    // Сброс в конце
    print!("\x1b[0m");
}
