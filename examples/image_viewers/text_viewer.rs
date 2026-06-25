use overwhelement::*;
use image::{open, RgbaImage};
use std::{env, rc::Rc};
use terminal_size::terminal_size;

struct TextureShader {
    image: RgbaImage,
}

impl ElementShader for TextureShader {
    fn shade(&self, input: &ShaderInput) -> ShaderOutput {
        let u = input.uv.0.clamp(0.0, 1.0);
        let v = input.uv.1.clamp(0.0, 1.0);
        // Прямое сопоставление UV с пикселями (V=0 вверху)
        let x = (u * (self.image.width() - 1) as f32) as u32;
        let y = (v * (self.image.height() - 1) as f32) as u32;
        let pixel = self.image.get_pixel(x, y);
        let [r, g, b, a] = pixel.0;

        ShaderOutput {
            color: [r, g, b, a],
            luminance: None,
            object_id: None,
        }
    }
}

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

    let offset_x = ((out_w as f32 - scaled_viewport_width) / 2.0).ceil() as u32;
    let offset_y = ((out_h as f32 - scaled_viewport_height) / 2.0).ceil() as u32;
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
    let (out_w, out_h) = terminal_size().expect("Unable to get output size");

    // 1. Создаём сцену
    let mut scene = Scene::new();

    // 2. Добавляем шейдер в пул сцены и получаем его индекс
    let shader_idx = scene.shader_pool.add(Rc::new(TextureShader { image: img }));

    let (offset_x, offset_y) = offset_to_center(out_w.0 as u32, out_h.0 as u32, width, height, 0.5);
    println!("{}", offset_x);
    println!("{}", offset_y);
    // 3. Создаём вьюпорт (без horizontal_alignment и vertical_alignment)
    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        width,
        height,
        scaling_mode: ScalingMode::Contain,
        element_aspect_ratio: 0.5,
        shader_map: vec![shader_idx],
        rotation_angle: 0.0,
        buffer_offset_x: Some(offset_x),
        buffer_offset_y: Some(offset_y),
        buffer_width: None,
        buffer_height: None,
    };
    let vp_idx = scene.add_viewport(vp);

    // 4. Создаём вершины (добавляем их в сцену)
    let v0 = scene.add_vertex(Vertex {
        x: 0.0,
        y: 0.0,
        depth: 0.0,
        u: 0.0,
        v: 0.0,
        ..Default::default()
    });
    let v1 = scene.add_vertex(Vertex {
        x: width,
        y: 0.0,
        depth: 0.0,
        u: 1.0,
        v: 0.0,
        ..Default::default()
    });
    let v2 = scene.add_vertex(Vertex {
        x: width,
        y: height,
        depth: 0.0,
        u: 1.0,
        v: 1.0,
        ..Default::default()
    });
    let v3 = scene.add_vertex(Vertex {
        x: 0.0,
        y: height,
        depth: 0.0,
        u: 0.0,
        v: 1.0,
        ..Default::default()
    });

    // 5. Создаём треугольники (используя индексы вершин) и добавляем их в сцену
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

    // 6. Создаём плоскость, ссылающуюся на треугольники по индексам
    let plane = Plane {
        id: 0,
        triangles: vec![t1_idx, t2_idx],
        lines: Vec::new(),
        viewport_indices: vec![vp_idx],
    };
    scene.add_plane(plane);
    let settings = Settings {
        output_width: out_w.0 as u32,
        output_height: out_h.0 as u32,
        background_color: [0, 0, 0],
        background_luminance: 0.0,
    };

    let buffer = discretize(&scene, &settings);

    // 8. Вывод в терминал с ANSI-цветами
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let elem = buffer.get(x, y).unwrap();
            if elem.object_id == EMPTY_OBJECT_ID {
                print!("\x1b[48;2;0;0;0m \x1b[0m");
            } else {
                print!(
                    "\x1b[48;2;{};{};{}m \x1b[0m",
                    elem.color[0], elem.color[1], elem.color[2]
                );
            }
        }
        println!();
    }
}