use crate::math::vec3::Vec3;

#[derive(Clone)]
pub struct Triangle {
    pub p: [Vec3; 3],
    pub c: (u8, u8, u8),
    pub avg_z: f32,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            p: [
                Vec3 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                Vec3 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                Vec3 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            ],
            c: (255, 255, 255),
            avg_z: 0.0,
        }
    }
}