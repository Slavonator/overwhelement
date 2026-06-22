use overwhelement::*;
use image::{open, RgbaImage};
use std::{env, rc::Rc};
use terminal_size::{terminal_size};

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
        buffer_offset_x: None,
        buffer_offset_y: None,
        buffer_width: None,
        buffer_height: None,
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
        output_height: out_h.0 as u32,
        background_color: [0, 0, 0],
        background_luminance: 0.0,
    };

    let buffer = discretize(&scene, &settings);

    // Вывод в терминал с ANSI-цветами
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
