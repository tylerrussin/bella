use crate::math::vec3::Vec3;

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn look_direction(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();

        Vec3 {
            x: -self.yaw.sin() * cos_pitch,
            y: self.pitch.sin(),
            z: self.yaw.cos() * cos_pitch,
            w: 0.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let look = self.look_direction();

        Vec3 {
            x: look.x,
            y: 0.0,
            z: look.z,
            w: 0.0,
        }
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        let up = Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
            w: 0.0,
        };

        up.cross(self.forward()).normalize()
    }

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch += pitch_delta;

        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }
}