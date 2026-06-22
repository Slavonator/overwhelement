use crate::datatypes::*;

impl ElementBuffer {

    pub(crate) fn blend(&mut self, x: u32, y: u32, src_color: [u8; 4], luminance: f32, object_id: u32) {
        let elem = &mut self.get_mut(x, y).unwrap();
        let src_a = src_color[3] as u32;
        let src_r = src_color[0] as u32;
        let src_g = src_color[1] as u32;
        let src_b = src_color[2] as u32;
        let dst_r = elem.color[0] as u32;
        let dst_g = elem.color[1] as u32;
        let dst_b = elem.color[2] as u32;
        elem.color = [
            ((src_r * src_a + dst_r * (255 - src_a)) / 255) as u8,
            ((src_g * src_a + dst_g * (255 - src_a)) / 255) as u8,
            ((src_b * src_a + dst_b * (255 - src_a)) / 255) as u8,
        ];
        elem.luminance = luminance;
        if object_id != EMPTY_OBJECT_ID {
            elem.object_id = object_id;
        }
    }
}

pub(crate) struct TransparentFragment {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) depth: f32,
    pub(crate) layer: u32,
    pub(crate) color: [u8; 4],
    pub(crate) luminance: f32,
    pub(crate) object_id: u32,
}

pub(crate) struct FragmentData {
    pub(crate) depth: f32,
    pub(crate) u: f32,
    pub(crate) v: f32,
    pub(crate) normal: [f32; 3],
    pub(crate) luminance: f32,
    pub(crate) object_id: u32,
}
