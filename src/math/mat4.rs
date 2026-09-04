use crate::math::vec3::Vec3;

pub struct Mat4x4 {
    m: [f32; 16],
}



impl Mat4x4 {
    pub fn new() -> Self {
        Self { m: [0.0; 16] }
    }
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.m[row * 4 + col]
    }
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.m[row * 4 + col] = value;
    }
}

pub fn multiply_matrix(a: &Mat4x4, b: &Mat4x4) -> Mat4x4 {
    let mut result = Mat4x4::new();

    for row in 0..4 {
        for col in 0..4 {
            let mut sum = 0.0;
            for i in 0..4 {
                sum += a.get(row, i) * b.get(i, col);
            }
            result.set(row, col, sum);
        }
    }

    result
}

pub fn matrix_point_at(pos: Vec3, target: Vec3, up: Vec3) -> Mat4x4 {
   // Calculate new forward direction
   let mut new_forward = target - pos;
   new_forward = new_forward.normalize();

   // Calculate new up direction
   let a = new_forward * up.dot(new_forward);
   let mut new_up = up - a;
   new_up = new_up.normalize();

   // New right direction is just the cross product
   let new_right = new_up.cross(new_forward);

   // Construct dimensioning and translation matrix
   let mut matrix = Mat4x4::new();
   matrix.set(0, 0, new_right.x);
   matrix.set(0, 1, new_right.y);
   matrix.set(0, 2, new_right.z);
   matrix.set(0, 3, 0.0);

   matrix.set(1, 0, new_up.x);
   matrix.set(1, 1, new_up.y);
   matrix.set(1, 2, new_up.z);
   matrix.set(1, 3, 0.0);

   matrix.set(2, 0, new_forward.x);
   matrix.set(2, 1, new_forward.y);
   matrix.set(2, 2, new_forward.z);
   matrix.set(2, 3, 0.0);

   matrix.set(3, 0, pos.x);
   matrix.set(3, 1, pos.y);
   matrix.set(3, 2, pos.z);
   matrix.set(3, 3, 1.0);

   matrix
}

pub fn identity() -> Mat4x4 {
    let mut mat = Mat4x4::new();
    mat.set(0, 0, 1.0);
    mat.set(1, 1, 1.0);
    mat.set(2, 2, 1.0);
    mat.set(3, 3, 1.0);
    mat
}
pub fn rotation_x(angle_radian: f32) -> Mat4x4 {
    let mut mat = Mat4x4::new();
    mat.set(0, 0, 1.0);
    mat.set(1, 1, angle_radian.cos());
    mat.set(1, 2, angle_radian.sin());
    mat.set(2, 1, -angle_radian.sin());
    mat.set(2, 2, angle_radian.cos());
    mat.set(3, 3, 1.0);
    mat
}
pub fn rotation_y(angle_radian: f32) -> Mat4x4 {
    let mut mat = Mat4x4::new();
    mat.set(0, 0, angle_radian.cos());
    mat.set(0, 2, angle_radian.sin());
    mat.set(2, 0, -angle_radian.sin());
    mat.set(1, 1, 1.0);
    mat.set(2, 2, angle_radian.cos());
    mat.set(3, 3, 1.0);
    mat
}
pub fn rotation_z(angle_radian: f32) -> Mat4x4 {
    let mut mat = Mat4x4::new();
    mat.set(0, 0, angle_radian.cos());
    mat.set(0, 1, angle_radian.sin());
    mat.set(1, 0, -angle_radian.sin());
    mat.set(1, 1, angle_radian.cos());
    mat.set(2, 2, 1.0);
    mat.set(3, 3, 1.0);
    mat
}
pub fn translation(x: f32, y: f32, z: f32) -> Mat4x4 {
    let mut mat = Mat4x4::new();
    mat.set(0, 0, 1.0);
    mat.set(1, 1, 1.0);
    mat.set(2, 2, 1.0);
    mat.set(3, 0, x);
    mat.set(3, 1, y);
    mat.set(3, 2, z);
    mat.set(3, 3, 1.0);
    mat
}
pub fn make_projection(fov_deg: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4x4 {
    let fov_rad = 1.0 / (fov_deg * 0.5 / 180.0 * 3.14159).tan();
    let mut mat = Mat4x4::new();
    mat.set(0, 0, aspect_ratio * fov_rad);
    mat.set(1, 1, fov_rad);
    mat.set(2, 2, far / (far - near));
    mat.set(3, 2, (-far * near) / (far - near));
    mat.set(2, 3, 1.0);
    mat.set(3, 3, 0.0);
    mat
}
pub fn matrix_quick_inverse(m: Mat4x4) -> Mat4x4 {
    let mut mat = Mat4x4::new();
    mat.set(0, 0, m.get(0, 0));
    mat.set(0, 1, m.get(1, 0));
    mat.set(0, 2, m.get(2, 0));
    mat.set(0, 3, 0.0);
    mat.set(1, 0, m.get(0, 1));
    mat.set(1, 1, m.get(1, 1));
    mat.set(1, 2, m.get(2, 1));
    mat.set(1, 3, 0.0);
    mat.set(2, 0, m.get(0, 2));
    mat.set(2, 1, m.get(1, 2));
    mat.set(2, 2, m.get(2, 2));
    mat.set(2, 3, 0.0);
    mat.set(3, 0, -(m.get(3, 0) * mat.get(0, 0) + m.get(3, 1) * mat.get(1, 0) + m.get(3, 2) * mat.get(2, 0)));
    mat.set(3, 1, -(m.get(3, 0) * mat.get(0, 1) + m.get(3, 1) * mat.get(1, 1) + m.get(3, 2) * mat.get(2, 1)));
    mat.set(3, 2, -(m.get(3, 0) * mat.get(0, 2) + m.get(3, 1) * mat.get(1, 2) + m.get(3, 2) * mat.get(2, 2)));
    mat.set(3, 3, 1.0);
    mat
}