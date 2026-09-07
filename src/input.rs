use winit::event::{
    DeviceEvent,
    ElementState,
    VirtualKeyCode,
    WindowEvent,
};

pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,

    pub mouse_dx: f32,
    pub mouse_dy: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,

            mouse_dx: 0.0,
            mouse_dy: 0.0,
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            if let Some(keycode) = input.virtual_keycode {
                let pressed = input.state == ElementState::Pressed;

                match keycode {
                    VirtualKeyCode::W => {
                        self.forward = pressed;
                    }

                    VirtualKeyCode::S => {
                        self.backward = pressed;
                    }

                    VirtualKeyCode::A => {
                        self.left = pressed;
                    }

                    VirtualKeyCode::D => {
                        self.right = pressed;
                    }

                    _ => {}
                }
            }
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_dx += delta.0 as f32;
            self.mouse_dy += delta.1 as f32;
        }
    }

    pub fn take_mouse_delta(&mut self) -> (f32, f32) {
        let delta = (self.mouse_dx, self.mouse_dy);

        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;

        delta
    }
}