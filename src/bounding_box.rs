use crate::{
    f32::{max_f32, min_f32},
    point_object::PointObject,
};

pub struct BoundingBox {
    pub min: [f32; 3],
    pub x_width: f32,
    pub y_width: f32,
    pub z_width: f32,
}

impl BoundingBox {
    pub fn new<T>(points: &[T]) -> Self
    where
        T: PointObject,
    {
        let mut x_min = f32::INFINITY;
        let mut y_min = f32::INFINITY;
        let mut z_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        let mut y_max = f32::NEG_INFINITY;
        let mut z_max = f32::NEG_INFINITY;

        for p in points {
            x_min = min_f32(p.position()[0], x_min);
            y_min = min_f32(p.position()[1], y_min);
            z_min = min_f32(p.position()[2], z_min);
            x_max = max_f32(p.position()[0], x_max);
            y_max = max_f32(p.position()[1], y_max);
            z_max = max_f32(p.position()[2], z_max);
        }

        BoundingBox {
            min: [x_min, y_min, z_min],
            x_width: x_max - x_min,
            y_width: y_max - y_min,
            z_width: z_max - z_min,
        }
    }
}
