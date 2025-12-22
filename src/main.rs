use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use pixels::{Pixels, SurfaceTexture};

use std::time::{Instant};
use std::io::{stdout, Write};

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone)]
struct Triangle {
    p: [Vec3; 3],
}

struct Mesh {
    tris: Vec<Triangle>,
}

struct Mat4x4 {
    m: [[f32; 4]; 4],
}

impl Mat4x4 {
    fn new() -> Self {
        Self { m: [[0.0; 4]; 4] }
    }
}

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


fn set_pixel(frame: &mut [u8], x: usize, y: usize, width: usize, r: u8, g: u8, b: u8) {
    let i = (y * width + x) * 4;
    frame[i] = r;
    frame[i + 1] = g;
    frame[i + 2] = b;
    frame[i + 3] = 255;
}

fn reset_screen(frame: &mut [u8]) {
    // Black out the screen
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 0; // R
        pixel[1] = 0; // G
        pixel[2] = 0; // B
        pixel[3] = 255; // A
    }
}

// set_pixel(frame, x, y, width, r, g, b);

fn multiply_matrix_vector(i: &Vec3, m: &Mat4x4) -> Vec3 {
    let mut o = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    o.x = i.x * m.m[0][0] + i.y * m.m[1][0] + i.z * m.m[2][0] + m.m[3][0];
    o.y = i.x * m.m[0][1] + i.y * m.m[1][1] + i.z * m.m[2][1] + m.m[3][1];
    o.z = i.x * m.m[0][2] + i.y * m.m[1][2] + i.z * m.m[2][2] + m.m[3][2];
    let w = i.x * m.m[0][3] + i.y * m.m[1][3] + i.z * m.m[2][3] + m.m[3][3];

    if w != 0.0 {
        o.x /= w;
        o.y /= w;
        o.z /= w;
    }

    o
}

fn draw(frame: &mut [u8], x: usize, y: usize, color: (u8, u8, u8), width: usize, height: usize) {
    if x < width && x >= 0 && y < height && y >= 0 {
        set_pixel(frame, x, y, width, color.0, color.1, color.2);
    }
}

fn draw_line(frame: &mut [u8], x1: i32, y1: i32, x2: i32, y2: i32, color: (u8, u8, u8), width: i32, height: i32) {
    let mut x;
    let mut y;
    let mut xe;
    let mut ye;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dx1_abs = dx.abs();
    let dy1_abs = dy.abs();
    let mut px = 2 * dy1_abs - dx1_abs;
    let mut py = 2 * dx1_abs - dy1_abs;

    if dy1_abs <= dx1_abs {
        if dx >= 0 {
            x = x1;
            y = y1;
            xe = x2;
        } else {
            x = x2;
            y = y2;
            xe = x1;
        }
        draw(frame, x as usize, y as usize, color, width as usize, height as usize);
        for _ in x..xe {
            x += 1;
            if px < 0 {
                px = px + 2 * dy1_abs;
            } else {
                if  (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                    y = y + 1;
                } else {
                    y = y - 1;
                }
                px = px + 2 * (dy1_abs - dx1_abs);
            }
            draw(frame, x as usize, y as usize, color, width as usize, height as usize);
        }
    } else {
        if dy >= 0 {
            x = x1;
            y = y1;
            ye = y1;
        } else {
            x = x2;
            y = y2;
            ye = y1;
        }
        draw(frame, x as usize, y as usize, color, width as usize, height as usize);
        for _ in y..ye {
            y += 1;
            if py <= 0 {
                py = py + 2 * dx1_abs;
            } else {
                if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                    x = x + 1;
                } else {
                    x = x - 1;
                }
                py = py + 2 * (dx1_abs -dy1_abs);
            }
            draw(frame, x as usize, y as usize, color, width as usize, height as usize);
        }
    }

}

fn draw_triangle(frame: &mut [u8], x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: (u8, u8, u8), width: i32, height: i32) {
    draw_line(frame, x1, y1, x2, y2, color, width, height);
    draw_line(frame, x2, y2, x3, y3, color, width, height);
    draw_line(frame, x3, y3, x1, y1, color, width, height);
}

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

    let mut mesh_cube = Mesh {
        tris: vec![
            // South
            Triangle { 
                p: [
                    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 0.0 },
                ] 
            },
            Triangle { 
                p: [
                    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                ] 
            },

            // East
            Triangle { 
                p: [
                    Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                ] 
            },
            Triangle { 
                p: [
                    Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                    Vec3 { x: 1.0, y: 0.0, z: 1.0 },
                ] 
            },

            // North
            Triangle { 
                p: [
                    Vec3 { x: 1.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 1.0 },
                ] 
            },
            Triangle { 
                p: [
                    Vec3 { x: 1.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 0.0, z: 1.0 },
                ] 
            },

            // West
            Triangle { 
                p: [
                    Vec3 { x: 0.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                ] 
            },
            Triangle { 
                p: [
                    Vec3 { x: 0.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                ] 
            },

            // Top
            Triangle { 
                p: [
                    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                    Vec3 { x: 0.0, y: 1.0, z: 1.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                ] 
            },
            Triangle { 
                p: [
                    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                    Vec3 { x: 1.0, y: 1.0, z: 0.0 },
                ] 
            },

            // Bottom
            Triangle { 
                p: [
                    Vec3 { x: 1.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                ] 
            },
            Triangle { 
                p: [
                    Vec3 { x: 1.0, y: 0.0, z: 1.0 },
                    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                    Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                ] 
            },
        ],
    };



    // Projection Matrix
    let mut mat_proj = Mat4x4::new();
    mat_proj.m[0][0] = aspect_ratio * fov_rad;
    mat_proj.m[1][1] = fov_rad;
    mat_proj.m[2][2] = FAR / (FAR - NEAR);
    mat_proj.m[3][2] = (-FAR * NEAR) / (FAR - NEAR);
    mat_proj.m[2][3] = 1.0;
    mat_proj.m[3][3] = 0.0;


    let mut theta: f32 = 0.0;

    let mut mat_rot_z = Mat4x4::new();
    let mut mat_rot_x = Mat4x4::new();

    

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("3D Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(width, height, surface_texture).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        let tp2: Instant = Instant::now();
        let elapsed_time: f64 = tp2.duration_since(tp1).as_secs_f64();
        tp1 = tp2;



        // Print stats
        print!("\rA={:.2} X={:.2} Y={:.2} FPS={:.2}     ", player_a, player_x, player_y, 1.0 / elapsed_time);
        stdout().flush().unwrap();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        match (keycode, input.state) {
                            (VirtualKeyCode::Escape, ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit;
                            }
                            (VirtualKeyCode::A, ElementState::Pressed) => {
                                player_a -= SPEED * 0.75 * DELTA;
                            }
                            (VirtualKeyCode::D, ElementState::Pressed) => {
                                player_a += SPEED * 0.75 * DELTA;
                            }

                            (VirtualKeyCode::W, ElementState::Pressed) => {
                                player_x += player_a.sin() * SPEED * DELTA;
                                player_y += player_a.cos() * SPEED * DELTA;

                            }

                            (VirtualKeyCode::S, ElementState::Pressed) => {
                                player_x -= player_a.sin() * SPEED * DELTA;
                                player_y -= player_a.cos() * SPEED * DELTA;
    
                            }

                            (VirtualKeyCode::Q, ElementState::Pressed) => {
                                player_x -= (player_a + 1.5).sin() * SPEED * DELTA;
                                player_y -= (player_a + 1.5).cos() * SPEED * DELTA;

                            }
                            
                            (VirtualKeyCode::E, ElementState::Pressed) => {
                                player_x -= (player_a - 1.5).sin() * SPEED * DELTA;
                                player_y -= (player_a - 1.5).cos() * SPEED * DELTA;
  
                            }

                            _ => {}
                        }
                    }
                }

                _ => {} // <-- catch all other WindowEvent variants
            },


            Event::RedrawRequested(_) => {
                let frame: &mut [u8] = pixels.frame_mut();

                reset_screen(frame);




                

                theta += DELTA as f32;



                    // Rotation Z
                mat_rot_z.m[0][0] = theta.cos();
                mat_rot_z.m[0][1] = theta.sin();
                mat_rot_z.m[1][0] = -theta.sin();
                mat_rot_z.m[1][1] = theta.cos();
                mat_rot_z.m[2][2] = 1.0;
                mat_rot_z.m[3][3] = 1.0;

                //Rotation X
                mat_rot_x.m[0][0] = 1.0;
                mat_rot_x.m[1][1] = (theta * 0.5).cos();
                mat_rot_x.m[1][2] = (theta * 0.5).sin();
                mat_rot_x.m[2][1] = -(theta * 0.5).sin();
                mat_rot_x.m[2][2] = (theta * 0.5).cos();
                mat_rot_x.m[3][3] = 1.0;



                

                // left here (draw triangles)
                for tri in &mesh_cube.tris {

                    let mut tri_rotated_z = Triangle {
                        p: [
                            multiply_matrix_vector(&tri.p[0], &mat_rot_z),
                            multiply_matrix_vector(&tri.p[1], &mat_rot_z),
                            multiply_matrix_vector(&tri.p[2], &mat_rot_z),
                        ],
                    };

                    let mut tri_rotated_zx = Triangle {
                        p: [
                            multiply_matrix_vector(&tri_rotated_z.p[0], &mat_rot_x),
                            multiply_matrix_vector(&tri_rotated_z.p[1], &mat_rot_x),
                            multiply_matrix_vector(&tri_rotated_z.p[2], &mat_rot_x),
                        ],
                    };
                    
                    let mut tri_translated = tri_rotated_zx.clone();
                    tri_translated.p[0].z = tri_rotated_zx.p[0].z + 3.0;
                    tri_translated.p[1].z = tri_rotated_zx.p[1].z + 3.0;
                    tri_translated.p[2].z = tri_rotated_zx.p[2].z + 3.0;

                    let mut tri_projected = Triangle {
                        p: [
                            multiply_matrix_vector(&tri_translated.p[0], &mat_proj),
                            multiply_matrix_vector(&tri_translated.p[1], &mat_proj),
                            multiply_matrix_vector(&tri_translated.p[2], &mat_proj),
                        ],
                    };

                    // Scale into view
                    tri_projected.p[0].x += 1.0;
                    tri_projected.p[1].x += 1.0;
                    tri_projected.p[2].x += 1.0;
    
                    tri_projected.p[0].y += 1.0;
                    tri_projected.p[1].y += 1.0;
                    tri_projected.p[2].y += 1.0;

                    tri_projected.p[0].x *= 0.5 * width as f32;
                    tri_projected.p[1].x *= 0.5 * width as f32;
                    tri_projected.p[2].x *= 0.5 * width as f32;

                    tri_projected.p[0].y *= 0.5 * height as f32;
                    tri_projected.p[1].y *= 0.5 * height as f32;
                    tri_projected.p[2].y *= 0.5 * height as f32;

                    


                    draw_triangle(frame, tri_projected.p[0].x as i32, tri_projected.p[0].y as i32, tri_projected.p[1].x as i32, tri_projected.p[1].y as i32, tri_projected.p[2].x as i32, tri_projected.p[2].y as i32, WHITE, width as i32, height as i32);

                }
                
                // set_pixel(frame, x, y, width, r, g, b);

                pixels.render().unwrap();
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        }
    });
}