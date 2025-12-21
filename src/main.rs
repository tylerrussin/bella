use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use pixels::{Pixels, SurfaceTexture};

use std::time::{Instant};
use std::io::{stdout, Write};

const MAP_HEIGHT: usize = 16;        
const MAP_WIDTH: usize = 16;

const FOV: f64 = 3.14159 / 4.0;         // Field of View
const DEPTH: f64 = 16.0;          // Maximum rendering distance
const SPEED: f64 = 5.0;           // Walking Speed
const DELTA: f64 = 0.05;

fn set_pixel(frame: &mut [u8], x: usize, y: usize, width: usize, r: u8, g: u8, b: u8) {
    let i = (y * width + x) * 4;
    frame[i] = r;
    frame[i + 1] = g;
    frame[i + 2] = b;
    frame[i + 3] = 255;
}

// set_pixel(frame, x, y, width, r, g, b);

fn raycast(frame: &mut [u8], screen_width: u32, screen_height: u32, player_a: f64, player_x: f64, player_y: f64, map: &Vec<Vec<char>>) {
    
    for x in 0..screen_width {
        // For each column, calculate the projected ray angle into world space
        let ray_angle = (player_a - FOV/2.0) + (x as f64 / screen_width as f64) * FOV;

        // Find distance to wall
        let step_size = 0.01;         // Increment size for ray casting, decrease to increase resolution
        let mut distance_to_wall = 0.0;

        let mut hit_wall = false;       // Set when ray hits wall block
        let mut boundary = false;       // Set when ray hits boundary between two wall blocks

        let eye_x = ray_angle.sin();     // Unit vector for ray in player space
        let eye_y = ray_angle.cos();

        // Incrementally cast ray from player, along ray angle, testing for intersection with a block
        while hit_wall == false && distance_to_wall < DEPTH {

            distance_to_wall += step_size;
            let test_x = player_x  + eye_x * distance_to_wall;
            let test_y = player_y + eye_y * distance_to_wall;

            // Test if ray is out of bounds
            if test_x < 0.0 || test_x >= MAP_WIDTH as f64 || test_y < 0.0 || test_y >= MAP_HEIGHT as f64 {
                
                hit_wall = true;
                distance_to_wall = DEPTH;

            }

            else {

                // Ray is inbounds so test to see if the ray cell is a wall block
                if map[test_y as usize][test_x as usize] == '#' {

                    // Ray has hit wall
                    hit_wall = true;

                    // To highlight tile boundaries, cast a ray from each corner
                    // of the tile, to the player. The more coincident this ray
                    // is to the rendering ray, the closer we are to a tile
                    // boundary, which we'll shade to add details to the walls
                    let mut p: Vec<(f64, f64)> = Vec::new();

                    let tile_x = test_x.floor();
                    let tile_y = test_y.floor();


                    // Test each corner of hit tile, storing the distance from
                    // the player, and the calculated dot product of the two rays
                    for tx in 0..2 {
                        for ty in 0..2 {

                            // Angle of corner to eye
                            let vx = tile_x + tx as f64 - player_x;
                            let vy = tile_y + ty as f64 - player_y;
                            let d = (vx * vx + vy * vy).sqrt();
                            let dot = (eye_x * vx / d) + (eye_y * vy / d);
                            p.push((d, dot));

                        }
                    }

                    // Sort Pairs from closest to farthest
                    p.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                    // First two/three are closest (we will never see all four)
                    let bound = 0.01;
                    if p[0].1.acos() < bound { boundary = true; }
                    if p[1].1.acos() < bound { boundary = true; }
                    // if p[2].1.acos() < bound { boundary = true; }
                }
            }
        }

        // Calculate distance to ceiling and floor
        let ceiling = screen_height as f64 / 2.0 - screen_height as f64 / distance_to_wall;
        let floor = screen_height as f64 - ceiling;

        // Shade walls based on distance
        let mut shade;
        if distance_to_wall <= DEPTH / 4.0 {
            shade = '\u{2588}';  // Very Close
        } else if distance_to_wall < DEPTH / 3.0 {
            shade = '\u{2593}';
        } else if distance_to_wall < DEPTH / 2.0 {
            shade = '\u{2592}';
        } else if distance_to_wall < DEPTH {
            shade = '\u{2591}';
        } else {
            shade = ' '; // Too far away
        }

        if boundary {
            shade = ' '; // Black it out
        }

        for y in 0..screen_height {

            // Each Row
            if (y as f64) < ceiling {
                set_pixel(frame, x as usize, y as usize, screen_width as usize, 0, 0, 0);
            } else if (y as f64) > ceiling && (y as f64)  <= floor {

                let (r1, g1, b1) = match shade {
                    '\u{2588}' => (255, 255, 255), // very close wall
                    '\u{2593}' => (192, 192, 192),
                    '\u{2592}' => (128, 128, 128),
                    '\u{2591}' => (64, 64, 64),
                    ' '        => (0, 0, 0),
                    _          => (0, 0, 0),
                };

                set_pixel(frame, x as usize, y as usize, screen_width as usize, r1, g1, b1);
            } else {
                // Floor

                // Shade floor based on distance
                let b = 1.0 - (y as f64 - screen_height as f64 / 2.0) / (screen_height as f64 / 2.0);
                let shade_2;
                if b < 0.25 {
                    shade_2 = '#';
                } else if b < 0.5 {
                    shade_2 = 'x';
                } else if b < 0.75 {
                    shade_2 = '.';
                } else if b < 0.9 {
                    shade_2 = '-';
                } else {
                    shade_2 = ' ';
                }

                let (r, g, b) = match shade_2 {
                    '#' => (255, 255, 255),
                    'x' => (192, 192, 192),
                    '.' => (128, 128, 128),
                    '-' => (64, 64, 64),
                    _   => (0, 0, 0),
                };

                set_pixel(frame, x as usize, y as usize, screen_width as usize, r, g, b);
            }
        }
    }
}

fn main() {
    let width: u32 = 1600;
    let height: u32 = 1000;
    
    // Player position
    let mut player_a: f64 = 0.0;         // Player Start Rotation
    let mut player_x: f64 = 13.0;        // Player Start Position
    let mut player_y: f64 = 5.0;

    // Create Map of world space # = wall block, . = space
    let mut map: Vec<Vec<char>> = Vec::new();

    map.push("#########.......".chars().collect());
    map.push("#...............".chars().collect());
    map.push("#.......########".chars().collect());
    map.push("#..............#".chars().collect());
    map.push("#......##......#".chars().collect());
    map.push("#......##......#".chars().collect());
    map.push("#..............#".chars().collect());
    map.push("###............#".chars().collect());
    map.push("##.............#".chars().collect());
    map.push("#......####..###".chars().collect());
    map.push("#......#.......#".chars().collect());
    map.push("#......#.......#".chars().collect());
    map.push("#..............#".chars().collect());
    map.push("#......#########".chars().collect());
    map.push("#..............#".chars().collect());
    map.push("################".chars().collect());

    let mut tp1: Instant = Instant::now();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Red Window")
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
                                if map[player_y as usize][player_x as usize] == '#' {
                                    player_x -= player_a.sin() * SPEED * DELTA;
                                    player_y -= player_a.cos() * SPEED * DELTA;
                                }
                            }

                            (VirtualKeyCode::S, ElementState::Pressed) => {
                                player_x -= player_a.sin() * SPEED * DELTA;
                                player_y -= player_a.cos() * SPEED * DELTA;
                                if map[player_y as usize][player_x as usize] == '#' {
                                    player_x += player_a.sin() * SPEED * DELTA;
                                    player_y += player_a.cos() * SPEED * DELTA;
                                }
                            }

                            (VirtualKeyCode::Q, ElementState::Pressed) => {
                                player_x -= (player_a + 1.5).sin() * SPEED * DELTA;
                                player_y -= (player_a + 1.5).cos() * SPEED * DELTA;
                                if map[player_y as usize][player_x as usize] == '#' {
                                    player_x += (player_a + 1.5).sin() * SPEED * DELTA;
                                    player_y += (player_a + 1.5).cos() * SPEED * DELTA;
                                }
                            }
                            
                            (VirtualKeyCode::E, ElementState::Pressed) => {
                                player_x -= (player_a - 1.5).sin() * SPEED * DELTA;
                                player_y -= (player_a - 1.5).cos() * SPEED * DELTA;
                                if map[player_y as usize][player_x as usize] == '#' {
                                    player_x += (player_a - 1.5).sin() * SPEED * DELTA;
                                    player_y += (player_a - 1.5).cos() * SPEED * DELTA;
                                }
                            }

                            _ => {}
                        }
                    }
                }

                _ => {} // <-- catch all other WindowEvent variants
            },


            Event::RedrawRequested(_) => {
                let frame: &mut [u8] = pixels.frame_mut();

                raycast(
                    frame,
                    width,
                    height,
                    player_a,
                    player_x,
                    player_y,
                    &map,
                );

                // Print stats
                print!("\rA={:.2} X={:.2} Y={:.2} FPS={:.2}     ", player_a, player_x, player_y, 1.0 / elapsed_time);
                stdout().flush().unwrap();

                pixels.render().unwrap();
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        }
    });
}