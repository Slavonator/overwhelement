use std::rc::Rc;
use ab_glyph::{FontRef, PxScale};
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use overwhelement::*;

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

const W: u32 = 800;
const H: u32 = 400;

fn main() {
    let mut scene = Scene::new();

    // Шейдеры
    let plane_color = SolidShader {
        color: [130, 130, 230, 255],
    };
    let plane_outline = SolidShader {
        color: [90, 90, 190, 255],
    };
    let inner_triangle1_color = SolidShader {
        color: [200, 50, 50, 255],
    };
    let inner_triangle2_color = SolidShader {
        color: [140, 230, 140, 255],
    };
    let vp_outline1 = SolidShader {
        color: [90, 190, 190, 255],
    };
    let vp_outline2 = SolidShader {
        color: [60, 60, 60, 255],
    };

    scene.shader_pool.add(Rc::new(plane_color));          // 0
    scene.shader_pool.add(Rc::new(plane_outline));        // 1
    scene.shader_pool.add(Rc::new(inner_triangle1_color));// 2
    scene.shader_pool.add(Rc::new(inner_triangle2_color));// 3
    scene.shader_pool.add(Rc::new(vp_outline1));          // 4
    scene.shader_pool.add(Rc::new(vp_outline2));          // 5

    
    // Основная плоскость
    let v0 = scene.add_vertex(Vertex::with_depth(1.0, 1.0, 1.5));
    let v1 = scene.add_vertex(Vertex::with_depth(2.0, 19.0, 1.5));
    let v2 = scene.add_vertex(Vertex::with_depth(18.0, 19.0, 1.5));
    let v3 = scene.add_vertex(Vertex::with_depth(17.0, 1.0, 1.5));

    // Внутренний треугольник 1 (красный)
    let v4 = scene.add_vertex(Vertex::with_depth(3.0, 6.0, 1.4));
    let v5 = scene.add_vertex(Vertex::with_depth(4.0, 15.0, 1.4));
    let v6 = scene.add_vertex(Vertex::with_depth(13.0, 14.0, 1.4));

    // Внутренний треугольник 2 (зелёный)
    let v7 = scene.add_vertex(Vertex::with_depth(6.0, 4.0, 1.3));
    let v8 = scene.add_vertex(Vertex::with_depth(13.0, 4.0, 1.3));
    let v9 = scene.add_vertex(Vertex::with_depth(11.0, 16.0, 1.3));

    // Треугольники
    // Основная плоскость (два треугольника)
    scene.add_triangle(Triangle {
        id: 0,
        vertices: [v0, v1, v2],
        local_shader_id: 0,
    });
    scene.add_triangle(Triangle {
        id: 0,
        vertices: [v0, v3, v2],
        local_shader_id: 0,
    });

    // Внутренний треугольник 1 (красный)
    scene.add_triangle(Triangle {
        id: 0,
        vertices: [v4, v5, v6],
        local_shader_id: 2, // inner_triangle1_color
    });

    // Внутренний треугольник 2 (зелёный)
    scene.add_triangle(Triangle {
        id: 0,
        vertices: [v7, v8, v9],
        local_shader_id: 3, // inner_triangle2_color
    });

    // Линии
    // Вспомогательная функция для добавления линии из двух точек
    let mut add_line = |x1, y1, x2, y2, shader_id, thickness| {
        let v1 = scene.add_vertex(Vertex::with_depth(x1, y1, 0.0));
        let v2 = scene.add_vertex(Vertex::with_depth(x2, y2, 0.0));
        scene.add_line(Line {
            id: 0,
            vertices: [v1, v2],
            local_shader_id: shader_id,
            thickness,
        })
    };

    // Вспомогательная функция для добавления замкнутого контура по точкам
    let mut add_outline = |points: &[(f32, f32)], shader_id, thickness| -> Vec<u32> {
        let mut line_indices = Vec::new();
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            line_indices.push(add_line(x1, y1, x2, y2, shader_id, thickness));
        }
        line_indices
    };

    // 1. Обводка основной плоскости (шейдер 1)
    let main_outline_points = [(1.0, 1.0), (2.0, 19.0), (18.0, 19.0), (17.0, 1.0)];
    let main_outline_lines = add_outline(&main_outline_points, 1, 5.0);

    // 2. Обводка вокруг внутреннего треугольника 1 (прямоугольник) – шейдер 4
    let rect1_points = [(3.0, 6.0), (3.0, 8.0), (6.0, 8.0), (6.0, 6.0)];
    let rect1_lines = add_outline(&rect1_points, 4, 5.0);

    // 3. Обводка вокруг внутреннего треугольника 2 (прямоугольник) – шейдер 5
    let rect2_points = [(7.0, 7.0), (7.0, 9.0), (10.0, 9.0), (10.0, 7.0)];
    let rect2_lines = add_outline(&rect2_points, 5, 5.0);

    // ---- Плоскость (содержит все треугольники и линии) ----
    let mut plane = Plane {
        id: 0,
        triangles: Vec::new(), // индексы треугольников будут добавлены позже
        lines: Vec::new(),
        viewport_indices: vec![0, 1, 2], // ссылается на все вьюпорты
    };

    // Добавляем все треугольники (их индексы мы не сохраняли, но они уже есть в scene.triangles)
    // Нам нужно добавить их индексы в plane.triangles. Поскольку мы добавляли треугольники
    // последовательно, их индексы: 0,1,2,3. Но если мы хотим быть точными, можно сохранять при добавлении.
    // Для простоты укажем их вручную:
    plane.triangles = vec![0, 1, 2, 3];

    // Добавляем линии
    plane.lines.extend(main_outline_lines);
    plane.lines.extend(rect1_lines);
    plane.lines.extend(rect2_lines);

    scene.add_plane(plane);

    // ---- Вьюпорты ----
    // Первый вьюпорт (с поворотом)
    let vp = Viewport {
        x: 20.0,
        y: 0.0,
        width: -20.0,   // отрицательная ширина — это нормально
        height: 20.0,
        scaling_mode: ScalingMode::Contain,
        element_aspect_ratio: 1.0,
        shader_map: vec![0, 1, 2, 3, 4, 5], // все шейдеры
        rotation_angle: 3.1415,
        buffer_offset_x: Some(0),
        buffer_offset_y: Some(0),
        buffer_width: Some(400),
        buffer_height: Some(400),
    };
    scene.add_viewport(vp);

    // Второй и третий вьюпорты (перекрывающиеся)
    let mut vp1 = Viewport::new_with_scaling(10.05, 6.85, -2.95, 1.95, ScalingMode::Stretch)
        .with_buffer_offset(390, 50)
        .with_buffer_size(400, 300);
    vp1.rotation_angle = 3.1415;
    vp1.shader_map = vec![0, 1, 2, 3];
    scene.add_viewport(vp1);

    let mut vp2 = Viewport::new_with_scaling(6.05, 5.85, -2.95, 1.95, ScalingMode::Stretch)
        .with_buffer_offset(390, 50)
        .with_buffer_size(400, 300);
    vp2.rotation_angle = 3.1415;
    vp2.shader_map = vec![0, 1, 2, 3];
    scene.add_viewport(vp2);

    // ---- Настройки дискретизации ----
    let settings = Settings {
        output_width: W,
        output_height: H,
        background_color: [240, 240, 240],
        background_luminance: 0.0,
    };

    let buffer = discretize(&scene, &settings);

    // ---- Сохранение в PNG ----
    let mut img = ImageBuffer::new(W, H);
    for y in 0..H {
        for x in 0..W {
            let elem = buffer.get(x, y).unwrap();
            let pixel = if elem.object_id == EMPTY_OBJECT_ID {
                Rgba([
                    settings.background_color[0],
                    settings.background_color[1],
                    settings.background_color[2],
                    255,
                ])
            } else {
                Rgba([elem.color[0], elem.color[1], elem.color[2], 255])
            };
            img.put_pixel(x, y, pixel);
        }
    }

    // ---- Текст поверх изображения ----
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let scale = PxScale { x: 20.0, y: 20.0 };
    let black = Rgba([0, 0, 0, 255]);

    draw_text_mut(&mut img, black, 50, 25, scale, &font, "Plane");
    draw_text_mut(&mut img, black, 400, 30, scale, &font, "Viewports overlap");

    img.save("multiple_viewports.png").expect("Failed to save PNG");
    println!("Saved multiple_viewports.png");
}