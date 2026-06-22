use crate::{datatypes::*, internal_datatypes::*};


pub(crate) fn process_fragment(
    buffer: &mut ElementBuffer,
    x: u32,
    y: u32,
    layer: u32,
    shader: &dyn ElementShader,
    transparent_fragments: &mut Vec<TransparentFragment>,
    data: FragmentData,
) {
    let current_element = buffer.get(x, y).unwrap();

    let input = ShaderInput {
        uv: (data.u, data.v),
        normal: data.normal,
        luminance: data.luminance,
        background_element: &current_element,
        fragment_depth: data.depth,
        fragment_layer: layer,
        object_id: data.object_id,
    };

    let output = shader.shade(&input);

    let alpha = output.color[3];
    if alpha == 0 {
        return;
    }

    let final_object_id = output.object_id.unwrap_or(data.object_id);
    let final_luminance = output.luminance.unwrap_or(input.luminance);

    if alpha == 255 {
        if layer > current_element.layer || (layer == current_element.layer && data.depth < current_element.depth) {
            let elem = buffer.get_mut(x, y).unwrap();
            elem.object_id = final_object_id;
            elem.depth = data.depth;
            elem.layer = layer;
            elem.color = [output.color[0], output.color[1], output.color[2]];
            elem.luminance = final_luminance;
        }
    } else {
        transparent_fragments.push(TransparentFragment {
            x,
            y,
            depth: data.depth,
            layer,
            color: output.color,
            luminance: final_luminance,
            object_id: final_object_id,
        });
    }
}

pub(crate) fn discretize_triangle(
    buffer: &mut ElementBuffer,
    tri: &Triangle,
    layer: u32,
    shader: &dyn ElementShader,
    transparent_fragments: &mut Vec<TransparentFragment>,
) {
    let v0 = &tri.vertices[0];
    let v1 = &tri.vertices[1];
    let v2 = &tri.vertices[2];

    let min_x = v0.x.min(v1.x).min(v2.x).floor() as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor() as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil() as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil() as i32;

    let min_x = min_x.max(0) as u32;
    let min_y = min_y.max(0) as u32;
    let max_x = (max_x.min(buffer.width as i32 - 1)) as u32;
    let max_y = (max_y.min(buffer.height as i32 - 1)) as u32;

    let area = edge_function(v0, v1, v2);
    if area.abs() < 1e-12 { return; }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = edge_function(v1, v2, &Vertex { x: px, y: py, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 0.0 });
            let w1 = edge_function(v2, v0, &Vertex { x: px, y: py, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 0.0 });
            let w2 = edge_function(v0, v1, &Vertex { x: px, y: py, depth: 0.0, u: 0.0, v: 0.0, normal: [0.0; 3], luminance: 0.0 });

            let inside = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if !inside {
                continue;
            }

            let w0 = w0 / area;
            let w1 = w1 / area;
            let w2 = w2 / area;

            let depth = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let u = w0 * v0.u + w1 * v1.u + w2 * v2.u;
            let v = w0 * v0.v + w1 * v1.v + w2 * v2.v;
            let normal = [
                w0 * v0.normal[0] + w1 * v1.normal[0] + w2 * v2.normal[0],
                w0 * v0.normal[1] + w1 * v1.normal[1] + w2 * v2.normal[1],
                w0 * v0.normal[2] + w1 * v1.normal[2] + w2 * v2.normal[2],
            ];
            let luminance = w0 * v0.luminance + w1 * v1.luminance + w2 * v2.luminance;

            process_fragment(
                buffer,
                x,
                y,
                layer,
                shader,
                transparent_fragments,
                FragmentData {
                    depth,
                    u,
                    v,
                    normal,
                    luminance,
                    object_id: tri.id,
                },
            );
        }
    }
}

fn edge_function(a: &Vertex, b: &Vertex, c: &Vertex) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

pub(crate) fn discretize_line(
    buffer: &mut ElementBuffer,
    line: &Line,
    layer: u32,
    shader: &dyn ElementShader,
    transparent_fragments: &mut Vec<TransparentFragment>,
) {
    let v0 = &line.vertices[0];
    let v1 = &line.vertices[1];

    let half_thickness = line.thickness * 0.5;
    let max_dist = half_thickness + 0.5;

    let min_x = v0.x.min(v1.x) - max_dist;
    let min_y = v0.y.min(v1.y) - max_dist;
    let max_x = v0.x.max(v1.x) + max_dist;
    let max_y = v0.y.max(v1.y) + max_dist;

    let min_x = (min_x.floor() as i32).max(0) as u32;
    let min_y = (min_y.floor() as i32).max(0) as u32;
    let max_x = (max_x.ceil() as i32).min(buffer.width as i32 - 1) as u32;
    let max_y = (max_y.ceil() as i32).min(buffer.height as i32 - 1) as u32;

    let dx = v1.x - v0.x;
    let dy = v1.y - v0.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let t = ((px - v0.x) * dx + (py - v0.y) * dy) / len_sq;
            let t_clamped = t.clamp(0.0, 1.0);
            let closest_x = v0.x + t_clamped * dx;
            let closest_y = v0.y + t_clamped * dy;
            let dist = ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();

            if dist > max_dist {
                continue;
            }

            let t_actual = t.clamp(0.0, 1.0);
            let depth = (1.0 - t_actual) * v0.depth + t_actual * v1.depth;
            let u = (1.0 - t_actual) * v0.u + t_actual * v1.u;
            let v = (1.0 - t_actual) * v0.v + t_actual * v1.v;
            let normal = [
                (1.0 - t_actual) * v0.normal[0] + t_actual * v1.normal[0],
                (1.0 - t_actual) * v0.normal[1] + t_actual * v1.normal[1],
                (1.0 - t_actual) * v0.normal[2] + t_actual * v1.normal[2],
            ];
            let luminance = (1.0 - t_actual) * v0.luminance + t_actual * v1.luminance;

            process_fragment(
                buffer,
                x,
                y,
                layer,
                shader,
                transparent_fragments,
                FragmentData {
                    depth,
                    u,
                    v,
                    normal,
                    luminance,
                    object_id: line.id,
                },
            );
        }
    }
}

