mod math;

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




fn triangle_clip_against_plane(plane_p: Vec3, mut plane_n: Vec3, in_tri: &Triangle, out_tri1: &mut Triangle, out_tri2: &mut Triangle) -> usize {
    // Confirm plane is normalised
    plane_n = plane_n.normalize();

    // Return signed shortest distance from point to plane, plane normal must be normalised
    let dist = |p: Vec3| -> f32 {
        plane_n.dot(p) - plane_n.dot(plane_p)
    };

    // Create two temporary storage arrays to classify points either side of plane
    // If distance sign is positive, piont lies on "inside" of plane
    let mut inside_points: Vec<Vec3> = Vec::new();
    let mut outside_points: Vec<Vec3> = Vec::new();

    // Get signed distance of each point in triangle to plane
    let d0: f32 = dist(in_tri.p[0]);
    let d1: f32 = dist(in_tri.p[1]);
    let d2: f32 = dist(in_tri.p[2]);

    // Classify each point
    for &p in &in_tri.p {
        let d = dist(p);
        if d >= 0.0 {
            inside_points.push(p);
        } else {
            outside_points.push(p);
        }
    }

    // clissify triangle points, and break the input triangleing
    // into smaller output triangles. there are four possible outcomes
    
    if inside_points.len() == 0 {
        // All points lie on outside so can clip whole triangle
        return 0;
    }
    if inside_points.len() == 3 {
        // All points lie on the inside of plane, so do nothing
        *out_tri1 = in_tri.clone();
        return 1;
    }
    if inside_points.len() == 1 && outside_points.len() == 2 {
        // Triangle should be clipped. as two points ie outside
        // the plane, the triangle simple becoms a smaller triangle
        
        // Copy apperance info to new triangle
        out_tri1.c = in_tri.c; //RED;
        out_tri1.avg_z = in_tri.avg_z;

        // The inside point is valid, so keep that...
        out_tri1.p[0] = inside_points[0];

        // but the two new points are at the locations where the 
        // original sides of the triangle(lines) intersect with the plane
        out_tri1.p[1] = Vec3::intersect_plane(plane_p, plane_n, inside_points[0], outside_points[0]);
        out_tri1.p[2] = Vec3::intersect_plane(plane_p, plane_n, inside_points[0], outside_points[1]);

        return 1; // return newly formed single tringle
    }
    if inside_points.len() == 2 && outside_points.len() == 1 {
        // Triangle should be clippled. two points lie inside the plane,
        // the clipped triangle becomes a quad. fortunetly, we can 
        // represetn a quad with two new triangles

        // Copy appearance info to new triangles
        out_tri1.c = in_tri.c; // BLUE;
        out_tri1.avg_z = in_tri.avg_z;

        out_tri2.c = in_tri.c; // GREEN;
        out_tri2.avg_z = in_tri.avg_z;

        //the first tri consists of the two inside points and a new
        //point determined by the locatio nwhere one side of the triangle
        //intersects with the plane
        out_tri1.p[0] = inside_points[0];
        out_tri1.p[1] = inside_points[1];
        out_tri1.p[2] = Vec3::intersect_plane(plane_p, plane_n, inside_points[0], outside_points[0]);

        // the second triangle is composed of one of the inside points, a
        // new point determined by the intersectio of the other side of the 
        // triangle and the plane, and the newl created point avove
        out_tri2.p[0] = inside_points[1];
        out_tri2.p[1] = out_tri1.p[2];
        out_tri2.p[2] = Vec3::intersect_plane(plane_p, plane_n, inside_points[1], outside_points[0]);

        return 2; // return two newly formed triangles which form a quad
    }
    return 0;
}








#[derive(Clone)]
struct Triangle {
    p: [Vec3; 3],
    c: (u8, u8, u8),
    avg_z: f32,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            p: [
                Vec3 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                Vec3 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                Vec3 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            ],
            c: WHITE,
            avg_z: 0.0,
        }
    }
}


struct Mesh {
    tris: Vec<Triangle>,
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



// Load model test
fn load_mesh(file_name: &str) -> Mesh {
    let file = std::fs::File::open(file_name).expect("Failed to open file");
    let reader = std::io::BufReader::new(file);

    let mut vectors_list: Vec<Vec3> = Vec::new();
    let mut triangles_list: Vec<Triangle> = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "v" => {
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                vectors_list.push(Vec3 { x, y, z, w: 1.0 });
            }
            "f" => {
                let i1: usize = parts[1].split('/').next().unwrap().parse::<usize>().unwrap() - 1;
                let i2: usize = parts[2].split('/').next().unwrap().parse::<usize>().unwrap() - 1;
                let i3: usize = parts[3].split('/').next().unwrap().parse::<usize>().unwrap() - 1;

                triangles_list.push(Triangle {
                    p: [
                        vectors_list[i1],
                        vectors_list[i2],
                        vectors_list[i3],
                    ],
                    c: WHITE,
                    avg_z: 0.0,
                });
            }
            _ => {}
        }
    }

    Mesh { tris: triangles_list }
}

fn draw(frame: &mut [u8], x: usize, y: usize, color: (u8, u8, u8), width: usize, height: usize) {
    if x < width && x >= 0 && y < height && y >= 0 {
        set_pixel(frame, x, y, width, color.0, color.1, color.2);
    }
}

fn draw_line(frame: &mut [u8], x1: i32, y1: i32, x2: i32, y2: i32, color: (u8, u8, u8), width: i32, height: i32) {
    let mut x;
    let mut y;
    let xe;
    let ye;
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
            ye = y2;
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

fn fill_line(frame: &mut [u8], sx: i32, ex: i32, ny: i32, color: (u8, u8, u8), width: i32, height: i32) {
    for i in sx..ex+1 {
        draw(frame, i as usize, ny as usize, color, width as usize, height as usize);
    }
}

// fn fill_triangle(frame: &mut [u8], mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32, mut x3: i32, mut y3: i32, color: (u8, u8, u8), width: i32, height: i32) {
//     let mut changed1 = false;
//     let mut changed2 = false;

//     // Sort vertices
//     if y1 > y2 {
//         (y1, y2) = (y2, y1);
//         (x1, x2) = (x2, x1);
//     }
//     if  y1 > y3 {
//         (y1, y3) = (y3, y1);
//         (x1, x3) = (x3, x1);
//     }
//     if y2 > y3 {
//         (y2, y3) = (y3, y2);
//         (x2, x3) = (x3, x2);
//     }    

//     let mut t1x = x1;
//     let mut t2x = x1;
//     let mut y = y1;  // Starting points
//     let mut signx1;
//     let mut signx2;

//     let mut dx1 = x2 - x1;
//     if dx1 < 0 {
//         dx1 = -dx1;
//         signx1 = -1;
//     } else {
//         signx1 = 1;
//     }
//     let mut dy1 = y2 - y1;

//     let mut dx2 = x3 - x1;
//     if dx2 < 0 {
//         dx2 = -dx2;
//         signx2 = -1;
//     } else {
//         signx2 = 1;
//     }
//     let mut dy2 = y3 - y1;

//     // Swap values
//     if dy1 > dx1 { 
//         (dx1, dy1) = (dy1, dx1);
//         changed1 = true;
//     }

//     // Swap values
//     if dy2 > dx2 {
//         (dy2, dx2) = (dx2, dy2);
//         changed2 = true;
//     }

//     let mut e2 = dx2 >> 1;

//     // Flat top, just process the second half
//     if y1 != y2 {

//         let mut e1 = dx1 >> 1;

        
//         for mut i in 0..dx1 {

//             let mut next1 = false;
//             let mut next2 = false;
//             let mut t1xp = 0;
//             let mut t2xp = 0;
//             let mut minx;
//             let mut maxx;

//             if t1x < t2x {
//                 minx = t1x;
//                 maxx = t2x;
//             } else {
//                 minx = t2x;
//                 maxx = t1x;
//             }

//             // Process first line until y value is about to change
//             while i < dx1 {
//                 i += 1;
//                 e1 += dy1;
//                 while e1 >= dx1 {
//                     e1 -= dx1;
//                     if changed1 {
//                         t1xp = signx1;
//                     } else {
//                         next1 = true;
//                         break;
//                     }
//                 }

//                 if next1 {
//                     break;
//                 }

//                 if changed1 {
//                     break;
//                 } else {
//                     t1x += signx1;
//                 }

//             }

//             // Next1
//             // Process second line until y value is about to change
//             loop {
//                 e2 += dy2;
//                 while e2 >= dx2 {
//                     e2 -= dx2;
//                     if changed2 {
//                         t2xp = signx2;
//                     } else {
//                         next2 = true;
//                         break;
//                     }
//                 }

//                 if next2 {
//                     break;
//                 }
//                 if changed2 {
//                     break;
//                 }
//                 else {
//                     t2x += signx2;
//                 }

//             }

//             // Next2
//             if minx > t1x {
//                 minx = t1x;
//             }
//             if minx > t2x {
//                 minx = t2x;
//             }
//             if maxx < t1x {
//                 maxx = t1x;
//             }
//             if maxx < t2x {
//                 maxx = t2x;
//             }

//             // Draw line from min to max points found on the y
//             fill_line(frame, minx, maxx, y, color, width, height);

//             // Now increase y
//             if !changed1 {
//                 t1x += signx1;
//             }
//             t1x += t1xp;
//             if !changed2 {
//                 t2x += signx2;
//             }
//             t2x += t2xp;
//             y += 1;
//             if y == y2 {
//                 break;
//             }

//         }
//     }

//     // Next
//     // Second half
//     dx1 = x3 - x2;
//     if dx1 < 0 {
//         dx1 = -dx1;
//         signx1 = -1;
//     } else {
//         signx1 = 1;
//     }
//     dy1 = y3 - y2;
//     t1x = x2;

//     // Swap values
//     if dy1 > dx1 {
//         (dy1, dx1) = (dx1, dy1);
//         changed1 = true;

//     } else {
//         changed1 = false;
//     }

//     let mut e1 = dx1 >> 1;

//     for mut  i in 0..dx1+1 {

//         let mut next3 = false;
//         let mut next4 = false;
//         let mut t1xp = 0;
//         let mut t2xp = 0;
//         let mut minx;
//         let mut maxx;

//         if t1x < t2x {
//             minx = t1x;
//             maxx = t2x;
//         } else {
//             minx = t2x;
//             maxx = t1x;
//         }

//         // Process first line until y value is about to change
//         while i < dx1 {
//             e1 += dy1;
//             while e1 >= dx1 {
//                 e1 -= dx1;
//                 if changed1 {
//                     t1xp = signx1; // t1x += signx1;
//                     break;
//                 } else {
//                     next3 = true;
//                     break;
//                 }
//             }

//             if next3 {
//                 break;
//             }

//             if changed1 {
//                 break;
//             }
//             else {
//                 t1x += signx1;
//             }
//             if i < dx1 {
//                 i += 1;
//             }

//         }

//         // Next3
//         // Process second line until y value is about to change
//         while t2x != x3 {
//             e2 += dy2;
//             while e2 >= dx2 {
//                 e2 -= dx2;
//                 if changed2 {
//                     t2xp = signx2;
//                 } else {
//                     next4 = true;
//                     break;
//                 }
//             }

//             if next4 {
//                 break;
//             }

//             if changed2 {
//                 break;
//             } else {
//                 t2x += signx2;
//             }
//         }

//         // Next4
//         // if minx > t1x:    # Visual Glitch with t1x
//         //     minx = t1x
//         if minx > t2x {
//             minx = t2x;
//         }
//         // if maxx < t1x:    # Visual Glitch with t1x
//         //     maxx = t1x
//         if maxx < t2x {
//             maxx = t2x;
//         }

//         fill_line(frame, minx, maxx, y, color, width, height);
//         if !changed1 {
//             t1x += signx1;
//         }
//         t1x += t1xp;
//         if !changed2 {
//             t2x += signx2;
//         }
//         t2x += t2xp;
//         y += 1;
//         if y > y3 {
//             return
//         }
//     }
// }

fn get_color() {

}

fn edge(a: Vec3, b: Vec3, p: Vec3) -> f32 {
    (p.x - a.x) * (b.y - a.y)
        - (p.y - a.y) * (b.x - a.x)
}

fn fill_triangle(
    frame: &mut [u8],
    depth_buffer: &mut [f32],
    tri: &Triangle,
    width: usize,
    height: usize,
) {
    let p0 = tri.p[0];
    let p1 = tri.p[1];
    let p2 = tri.p[2];

    let min_x = p0.x
        .min(p1.x)
        .min(p2.x)
        .floor()
        .max(0.0) as usize;

    let max_x = p0.x
        .max(p1.x)
        .max(p2.x)
        .ceil()
        .min((width - 1) as f32) as usize;

    let min_y = p0.y
        .min(p1.y)
        .min(p2.y)
        .floor()
        .max(0.0) as usize;

    let max_y = p0.y
        .max(p1.y)
        .max(p2.y)
        .ceil()
        .min((height - 1) as f32) as usize;

    let area = edge(p0, p1, p2);

    if area.abs() < 0.000001 {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pixel = Vec3::new(
                x as f32 + 0.5,
                y as f32 + 0.5,
                0.0,
            );

            let w0 = edge(p1, p2, pixel) / area;
            let w1 = edge(p2, p0, pixel) / area;
            let w2 = edge(p0, p1, pixel) / area;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let depth =
                    w0 * p0.z +
                    w1 * p1.z +
                    w2 * p2.z;

                let index = y * width + x;

                if depth < depth_buffer[index] {
                    depth_buffer[index] = depth;

                    set_pixel(
                        frame,
                        x,
                        y,
                        width,
                        tri.c.0,
                        tri.c.1,
                        tri.c.2,
                    );
                }
            }
        }
    }
}

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

    let mesh = load_mesh("test_map.obj");



    // Projection Matrix
    let mat_proj = make_projection(90.0, height as f32 / width as f32, 0.1, 1000.0); 


    let mut theta: f32 = 0.0;



    let mut v_camera = Vec3 {
        x: -34.0,
        y: 1.7,
        z: 0.0,
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
            println!("FPS: {:.1}", fps);

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

                // Clear screen
                reset_screen(frame);
                depth_buffer.fill(f32::INFINITY);

                // theta += 0.01;

                let mat_rot_z = rotation_z(theta * 0.5);
                let mat_rot_x = rotation_x(theta);

                let mat_trans = translation(0.0, 0.0, 16.0);

                let mut mat_world = identity();
                mat_world = multiply_matrix(&mat_world, &mat_rot_x);
                mat_world = multiply_matrix(&mat_world, &mat_rot_z);
                mat_world = multiply_matrix(&mat_world, &mat_trans);

                let v_up = Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                    w: 0.0,
                };

                let v_target = v_camera + v_look_dir;

                let v_up = Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                    w: 0.0,
                };

                let mat_camera =
                    matrix_point_at(v_camera, v_target, v_up);

                let mat_view =
                    matrix_quick_inverse(mat_camera);

                let mat_camera: Mat4x4 = matrix_point_at(v_camera, v_target, v_up);

                // Make view matrix from camera
                let mat_view: Mat4x4 = matrix_quick_inverse(mat_camera);



                // Store triangles for rastering later
                let mut vec_triangles_to_raster: Vec<Triangle> = Vec::new();


                // Draw triangles
                for tri in &mesh.tris {

                    let tri_transformed = Triangle {
                        p: [
                            tri.p[0].matrix_multiply_vector(&mat_world),
                            tri.p[1].matrix_multiply_vector(&mat_world),
                            tri.p[2].matrix_multiply_vector(&mat_world),
                        ],
                        c: WHITE,
                        avg_z: 0.0,
                    };

                    let line1 = tri_transformed.p[1] - tri_transformed.p[0];
                    let line2 = tri_transformed.p[2] - tri_transformed.p[0];

                    let mut normal = line1.cross(line2);

                    normal = normal.normalize();

                    let camera_ray = tri_transformed.p[0] - v_camera;


                    if (normal.dot(camera_ray)) < 0.0 {

                        let mut light_direction = Vec3 {
                            x: 0.0,
                            y: 1.0,
                            z: -1.0,
                            w: 1.0,
                        };
                        light_direction = light_direction.normalize();

                        let direction_to_light = Vec3 {
                            x: 0.4,
                            y: 1.0,
                            z: -0.6,
                            w: 0.0,
                        }.normalize();

                        let diffuse = normal
                            .dot(direction_to_light)
                            .max(0.0);

                        let ambient = 0.15;

                        let intensity =
                            ambient + diffuse * (1.0 - ambient);

                        let grey = (intensity * 255.0) as u8;

                        let color = (grey, grey, grey);

                        
                        // Convert world space --> view space
                        let viewed_p0 = tri_transformed.p[0].matrix_multiply_vector(&mat_view);
                        let viewed_p1 = tri_transformed.p[1].matrix_multiply_vector(&mat_view);
                        let viewed_p2 = tri_transformed.p[2].matrix_multiply_vector(&mat_view);

                        let tri_viewed = Triangle {
                            p: [
                                viewed_p0,
                                viewed_p1,
                                viewed_p2,
                            ],
                            c: color,
                            avg_z: (viewed_p0.z + viewed_p1.z + viewed_p2.z) / 3.0,
                        };

                        // Clip viewed triangle against near plane, this could form two additional triangles
                        let mut clipped = [
                            Triangle::default(),
                            Triangle::default(),
                        ];

                        let (first, rest) = clipped.split_at_mut(1);

                        let n_clipped_triangles = triangle_clip_against_plane(
                            Vec3 { x: 0.0, y: 0.0, z: 0.1, w: 1.0 },
                            Vec3 { x: 0.0, y: 0.0, z: 1.0, w: 1.0 },
                            &tri_viewed,
                            &mut first[0],
                            &mut rest[0],
                        );

                        for n in 0..n_clipped_triangles {


                            // Project triangles from 3D --> 2D
                            let mut tri_projected = Triangle {
                                p: [
                                    clipped[n].p[0].matrix_multiply_vector(&mat_proj),
                                    clipped[n].p[1].matrix_multiply_vector(&mat_proj),
                                    clipped[n].p[2].matrix_multiply_vector(&mat_proj),
                                ],
                                c: clipped[n].c,
                                avg_z: clipped[n].avg_z,
                            };

                            // normalize manually
                            tri_projected.p[0] = tri_projected.p[0] / tri_projected.p[0].w;
                            tri_projected.p[1] = tri_projected.p[1] / tri_projected.p[1].w;
                            tri_projected.p[2] = tri_projected.p[2] / tri_projected.p[2].w;

                            // NDC  has +Y upward.
                            // The framebuffer has +Y downward
                            tri_projected.p[0].y *= -1.0;
                            tri_projected.p[1].y *= -1.0;
                            tri_projected.p[2].y *= -1.0;
                        



                            // Scale triangle into view
                            let offset_view = Vec3 {
                                x: 1.0,
                                y: 1.0,
                                z: 0.0,
                                w: 1.0,
                            };

                            tri_projected.p[0] = tri_projected.p[0] + offset_view;
                            tri_projected.p[1] = tri_projected.p[1] + offset_view;
                            tri_projected.p[2] = tri_projected.p[2] + offset_view;



                            tri_projected.p[0].x *= 0.5 * width as f32;
                            tri_projected.p[1].x *= 0.5 * width as f32;
                            tri_projected.p[2].x *= 0.5 * width as f32;

                            tri_projected.p[0].y *= 0.5 * height as f32;
                            tri_projected.p[1].y *= 0.5 * height as f32;
                            tri_projected.p[2].y *= 0.5 * height as f32;

                            vec_triangles_to_raster.push(tri_projected);

                        }

                    }


                }






                // Loop through all transformed, viewed, projected, and sorted triangles
                // Clip and rasterize triangles
                for tri_to_raster in &vec_triangles_to_raster {
                    let mut clipped: [Triangle; 2] = [tri_to_raster.clone(), tri_to_raster.clone()];
                    let (first, rest) = clipped.split_at_mut(1);
                    let mut list_triangles: std::collections::VecDeque<Triangle> = std::collections::VecDeque::new();
                    list_triangles.push_back(tri_to_raster.clone());
                    let mut n_new_triangles = 1;

                    for p in 0..4 {
                        let mut n_tris_to_add = 0;

                        while n_new_triangles > 0 {
                            let test = list_triangles.pop_front().unwrap();
                            n_new_triangles -= 1;

                            n_tris_to_add = match p {
                                0 => triangle_clip_against_plane(
                                    Vec3::new(0.0, 0.0, 0.0),
                                    Vec3::new(0.0, 1.0, 0.0),
                                    &test,
                                    &mut first[0],
                                    &mut rest[0],
                                ),
                                1 => triangle_clip_against_plane(
                                    Vec3::new(0.0, (height - 1) as f32, 0.0),
                                    Vec3::new(0.0, -1.0, 0.0),
                                    &test,
                                    &mut first[0],
                                    &mut rest[0],
                                ),
                                2 => triangle_clip_against_plane(
                                    Vec3::new(0.0, 0.0, 0.0),
                                    Vec3::new(1.0, 0.0, 0.0),
                                    &test,
                                    &mut first[0],
                                    &mut rest[0],
                                ),
                                3 => triangle_clip_against_plane(
                                    Vec3::new((width - 1) as f32, 0.0, 0.0),
                                    Vec3::new(-1.0, 0.0, 0.0),
                                    &test,
                                    &mut first[0],
                                    &mut rest[0],
                                ),
                                _ => 0,
                            };

                            for w in 0..n_tris_to_add {
                                list_triangles.push_back(if w == 0 { first[w].clone() } else { rest[0].clone() });
                            }
                        }

                        n_new_triangles = list_triangles.len();
                    }

                    // Fill and optionally draw triangle edges
                    for t in &list_triangles {
                        fill_triangle(
                            frame,
                            &mut depth_buffer,
                            t,
                            width as usize,
                            height as usize,
                        );

                        // draw_triangle(
                        //     frame,
                        //     t.p[0].x as i32, t.p[0].y as i32,
                        //     t.p[1].x as i32, t.p[1].y as i32,
                        //     t.p[2].x as i32, t.p[2].y as i32,
                        //     WHITE,
                        //     width as i32,
                        //     height as i32,
                        //  );
                    }
                }

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

                window.request_redraw();
                
            }

            _ => {}
        }
    });
}
