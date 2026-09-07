use std::time::Instant;

use winit::event::{
    DeviceEvent,
    ElementState,
    VirtualKeyCode,
    WindowEvent,
};

use crate::camera::Camera;
use crate::geometry::Mesh;
use crate::input::InputState;
use crate::loader::load_obj;
use crate::math::mat4::{make_projection, Mat4x4};
use crate::math::vec3::Vec3;
use crate::renderer::render_screen;

pub struct App {
    pub width: u32,
    pub height: u32,

    mesh: Mesh,
    camera: Camera,
    input: InputState,

    projection: Mat4x4,
    depth_buffer: Vec<f32>,

    mouse_sensitivity: f32,
    movement_speed: f32,

    last_frame: Instant,

    theta: f32,
}

impl App {
    pub fn new(width: u32, height: u32) -> Self {
        let mesh = load_obj("assets/maps/test_map.obj");

        let camera = Camera::new(Vec3 {
            x: 5.0,
            y: 1.7,
            z: 5.0,
            w: 1.0,
        });

        let projection = make_projection(
            90.0,
            height as f32 / width as f32,
            0.1,
            1000.0,
        );

        let depth_buffer =
            vec![f32::INFINITY; (width * height) as usize];

        Self {
            width,
            height,

            mesh,
            camera,
            input: InputState::new(),

            projection,
            depth_buffer,

            mouse_sensitivity: 0.0025,
            movement_speed: 5.0,

            last_frame: Instant::now(),

            theta: 0.0,
        }
    }

    pub fn handle_device_event(
        &mut self,
        event: &DeviceEvent,
    ) {
        self.input.handle_device_event(event);
    }

    pub fn handle_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> bool {
        // Return true if the application should exit.
        match event {
            WindowEvent::CloseRequested => {
                return true;
            }

            WindowEvent::KeyboardInput {
                input: key_input,
                ..
            } => {
                if key_input.virtual_keycode
                    == Some(VirtualKeyCode::Escape)
                    && key_input.state == ElementState::Pressed
                {
                    return true;
                }
            }

            _ => {}
        }

        self.input.handle_window_event(event);

        false
    }

    pub fn update(&mut self) {
        // Mouse look
        let (mouse_dx, mouse_dy) =
            self.input.take_mouse_delta();

        self.camera.rotate(
            -mouse_dx * self.mouse_sensitivity,
            -mouse_dy * self.mouse_sensitivity,
        );

        // Movement directions
        let forward = self.camera.forward();
        let right = self.camera.right();

        // Delta time
        let now = Instant::now();

        let dt = now
            .duration_since(self.last_frame)
            .as_secs_f32()
            .min(0.05);

        self.last_frame = now;

        let distance = self.movement_speed * dt;

        if self.input.forward {
            self.camera.position =
                self.camera.position + forward * distance;
        }

        if self.input.backward {
            self.camera.position =
                self.camera.position - forward * distance;
        }

        if self.input.right {
            self.camera.position =
                self.camera.position + right * distance;
        }

        if self.input.left {
            self.camera.position =
                self.camera.position - right * distance;
        }

        // Gravity experiments can eventually go here.
        //
        // if self.camera.position.y >= 0.0
        //     || self.camera.position.z > 20.0
        // {
        //     self.camera.position.y -= 0.1;
        // }
    }

    pub fn render(&mut self, frame: &mut [u8]) {
        render_screen(
            frame,
            &mut self.depth_buffer,
            &self.mesh,
            self.camera.position,
            self.camera.look_direction(),
            &self.projection,
            self.theta,
            self.width,
            self.height,
        );
    }
}