use std::ops::{Add, Sub, Mul, Div};

use crate::math::mat4::Mat4x4;

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0, 
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w,
        }
    }
}

impl Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w,
        }
    }
}

impl Div for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w,
        }
    }
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z:f32) -> Self {
        Vec3 {x, y, z, w: 1.0}
    }
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn length(self) -> f32 {
        (self.dot(self)).sqrt()
    }
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l == 0.0 {
            return self
        }
        Vec3 {
            x: self.x / l,
            y: self.y / l,
            z: self.z / l,
            w: self.w,
        }
    }
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
            w: self.w,
        }
    }
    pub fn matrix_multiply_vector(self, m: &Mat4x4) -> Vec3 {
        Vec3 {
            x: self.x * m.get(0, 0) + self.y * m.get(1, 0) + self.z * m.get(2, 0) + self.w * m.get(3, 0),
            y: self.x * m.get(0, 1) + self.y * m.get(1, 1) + self.z * m.get(2, 1) + self.w * m.get(3, 1),
            z: self.x * m.get(0, 2) + self.y * m.get(1, 2) + self.z * m.get(2, 2) + self.w * m.get(3, 2),
            w: self.x * m.get(0, 3) + self.y * m.get(1, 3) + self.z * m.get(2, 3) + self.w * m.get(3, 3),
        }
    }
    pub fn intersect_plane(plane_p: Vec3, plane_n: Vec3, line_start: Vec3, line_end: Vec3) -> Vec3 {
        let plane_n: Vec3 = plane_n.normalize();
        let plane_d: f32 = -plane_n.dot(plane_p);
        let ad: f32 = line_start.dot(plane_n);
        let bd: f32 = line_end.dot(plane_n);
        let t: f32 = (-plane_d - ad) / (bd - ad);
        let line_start_to_end = line_end - line_start;
        let line_to_intersect = line_start_to_end * t;
        line_start + line_to_intersect
    }
}