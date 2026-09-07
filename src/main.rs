mod math;
mod geometry;
mod loader;
mod renderer;

use math::vec3::Vec3;

use math::mat4::{
    Mat4x4,
    multiply_matrix,
    matrix_point_at,
    identity,
    rotation_x,
    rotation_y,
    rotation_z,
    translation,
    make_projection,
    matrix_quick_inverse,
};

use geometry::{Triangle, Mesh};

use loader::load_obj;

use renderer::{
    reset_screen,
    fill_triangle,
    triangle_clip_against_plane,
    calculate_lighting,
    render_screen,
};

use winit::{
    event::{
        Event,
        WindowEvent,
        DeviceEvent,
        ElementState,
        VirtualKeyCode,
    },
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode},
};

use pixels::{Pixels, SurfaceTexture};

use std::time::{Instant};
use std::io::{stdout, Write, BufRead};


use std::ops::{
    Add,
    Sub,
    Mul,
    Div,
};







const NEAR: f32 = 0.1;
const FAR: f32 = 1000.0;
const FOV: f32 = 90.0;


const DEPTH: f64 = 16.0;          // Maximum rendering distance
const SPEED: f64 = 5.0;           // Walking Speed
const DELTA: f64 = 0.05;

// Colors (approximate RGB)
const BLACK: (u8, u8, u8)        = (0, 0, 0);
const DARK_BLUE: (u8, u8, u8)    = (0, 0, 128);
const DARK_GREEN: (u8, u8, u8)   = (0, 128, 0);
const DARK_CYAN: (u8, u8, u8)    = (0, 128, 128);
const DARK_RED: (u8, u8, u8)     = (128, 0, 0);
const DARK_MAGENTA: (u8, u8, u8) = (128, 0, 128);
const DARK_YELLOW: (u8, u8, u8)  = (128, 128, 0);
const GREY: (u8, u8, u8)         = (192, 192, 192);
const DARK_GREY: (u8, u8, u8)    = (128, 128, 128);
const BLUE: (u8, u8, u8)         = (0, 0, 255);
const GREEN: (u8, u8, u8)        = (0, 255, 0);
const CYAN: (u8, u8, u8)         = (0, 255, 255);
const RED: (u8, u8, u8)          = (255, 0, 0);
const MAGENTA: (u8, u8, u8)      = (255, 0, 255);
const YELLOW: (u8, u8, u8)       = (255, 255, 0);
const WHITE: (u8, u8, u8)        = (255, 255, 255);


const PELTA: f32 = 0.05;




fn main() {
    let width: u32 = 800;
    let height: u32 = 600;

    let aspect_ratio: f32 = height as f32 / width as f32;
    let fov_rad: f32 = 1.0 / (FOV * 0.5 / 180.0 * 3.14159).tan();
    
    // Player position
    let mut player_a: f64 = 0.0;         // Player Start Rotation
    let mut player_x: f64 = 13.0;        // Player Start Position
    let mut player_y: f64 = 5.0;

    let mut tp1: Instant = Instant::now();

    let mesh = load_obj("cube.obj");



    // Projection Matrix
    let mat_proj = make_projection(90.0, height as f32 / width as f32, 0.1, 1000.0); 


    let mut theta: f32 = 0.0;



    let mut v_camera = Vec3 {
        x: 5.0,
        y: 1.7,
        z: 5.0,
        w: 1.0,
    };

    let mut v_look_dir = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };


    use std::time::{Instant, Duration};

    let mut last_time = Instant::now();
    let mut frame_count = 0;


    
    // Create window and buffer
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("3D Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    window
    .set_cursor_grab(CursorGrabMode::Locked)
    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
    .ok();

    window.set_cursor_visible(false);

    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(width, height, surface_texture).unwrap();

    let mut depth_buffer = vec![f32::INFINITY; (width * height) as usize];

    let mut move_forward = false;
    let mut move_backward = false;
    let mut move_left = false;
    let mut move_right = false;

    let mut yaw: f32 = 0.0;
    let mut pitch: f32 = 0.0;

    let mouse_sensitivity: f32 = 0.0025;
    let movement_speed: f32 = 5.0;

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        frame_count += 1;

        let elapsed = last_time.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = frame_count as f64 / elapsed.as_secs_f64();
            // println!("FPS: {:.1}", fps);


            frame_count = 0;
            last_time = Instant::now();
        }


        // Handle input and player movement
        match event {

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                yaw -= delta.0 as f32 * mouse_sensitivity;
                pitch -= delta.1 as f32 * mouse_sensitivity;

                pitch = pitch.clamp(-1.5, 1.5);
            }
            Event::WindowEvent { event, .. } => match event {





                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    let pressed = input.state == ElementState::Pressed;

                    match keycode {
                        VirtualKeyCode::Escape if pressed => {
                            *control_flow = ControlFlow::Exit;
                        }

                        VirtualKeyCode::W => {
                            move_forward = pressed;
                        }

                        VirtualKeyCode::S => {
                            move_backward = pressed;
                        }

                        VirtualKeyCode::A => {
                            move_left = pressed;
                        }

                        VirtualKeyCode::D => {
                            move_right = pressed;
                        }

                        _ => {}
                    }
                }
            }

                _ => {} // <-- catch all other WindowEvent variants
            },


            Event::RedrawRequested(_) => {
                // Define screen
                let frame: &mut [u8] = pixels.frame_mut();

                render_screen(
                    frame,
                    &mut depth_buffer,
                    &mesh,
                    v_camera,
                    v_look_dir,
                    &mat_proj,
                    theta,
                    width,
                    height,
                );


                pixels.render().unwrap();
            }

            Event::MainEventsCleared => {


                let cos_pitch = pitch.cos();

                v_look_dir = Vec3 {
                    x: -yaw.sin() * cos_pitch,
                    y: pitch.sin(),
                    z: yaw.cos() * cos_pitch,
                    w: 0.0,
                };

                let forward = Vec3 {
                    x: v_look_dir.x,
                    y: 0.0,
                    z: v_look_dir.z,
                    w: 0.0,
                }.normalize();

                let up = Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                    w: 0.0,
                };

                let right = up.cross(forward).normalize();

                let now = Instant::now();

                let dt = now
                    .duration_since(last_frame)
                    .as_secs_f32()
                    .min(0.05);

                last_frame = now;

                let distance = movement_speed * dt;

                if move_forward {
                    v_camera = v_camera + forward * distance;
                }

                if move_backward {
                    v_camera = v_camera - forward * distance;
                }

                if move_right {
                    v_camera = v_camera + right * distance;
                }

                if move_left {
                    v_camera = v_camera - right * distance;
                }

 

                // let mut cur_min = 50.0;
                // for tri in &mesh.tris {
                //     for vec in tri.p {
                //         if vec.y < cur_min {
                //             cur_min = vec.y;

                //         }

                        
                //     }
                    
                // }

                // if v_camera.y >= 0.0 || v_camera.z > 20.0 {

                //     // Gravity
                //     v_camera.y -= 0.1;
                // }
                // println!("x:{} y:{} z:{}", v_camera.x, v_camera.y, v_camera.z);

                window.request_redraw();
                
            }

            _ => {}
        }
    });
}
