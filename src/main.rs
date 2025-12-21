use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, size},
    style::{SetBackgroundColor, SetForegroundColor, Color},
    ExecutableCommand,
};

use std::io::{stdout, Write, Result};
use std::time::{Duration, Instant};
use std::collections::HashSet;


const MAP_HEIGHT: usize = 16;        
const MAP_WIDTH: usize = 16;

const FOV: f64 = 3.14159 / 4.0;         // Field of View
const DEPTH: f64 = 16.0;          // Maximum rendering distance
const SPEED: f64 = 5.0;           // Walking Speed
const DELTA: f64 = 0.01;

fn set(screen: &mut [char], x: usize, y: usize, ch: char, width: usize) {
    screen[y * width + x] = ch;
}


// RAII cleanup struct to restore terminal on exit or panic
struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = disable_raw_mode();
        let _ = stdout.execute(Show);
        let _ = stdout.execute(LeaveAlternateScreen);
    }
}


fn main() -> Result<()> {
    let mut stdout = stdout();
    

    // Setup terminal
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    stdout.execute(SetBackgroundColor(Color::Black))?;
    stdout.execute(SetForegroundColor(Color::White))?;


    // RAII guard ensures cleanup
    let _cleanup = TerminalCleanup;

    let (cols, rows) = size()?;

    // Player position
    let mut player_a: f64 = 0.0;         // Player Start Rotation
    let mut player_x: f64 = 13.0;        // Player Start Position
    let mut player_y: f64 = 5.0;

    // Create Screen Buffer
    let screen_width = cols as usize;     // Console Screen Size X (columns)
    let screen_height = rows as usize;     // Console Screen Size Y (rows)
    let mut screen: Vec<char> = vec![' '; screen_width * screen_height];

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

    let mut tp1 = Instant::now();

    loop {
        let tp2 = Instant::now();
        let elapsed_time = tp2.duration_since(tp1).as_secs_f64();
        tp1 = tp2;


        let mut pressed_keys: HashSet<KeyCode> = HashSet::new();

        // Poll all key events without blocking
        while poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = read()? {
                match key_event.code {
                    KeyCode::Char(c) => {
                        pressed_keys.insert(KeyCode::Char(c.to_ascii_lowercase()));
                    }
                    KeyCode::Esc => {
                        pressed_keys.insert(KeyCode::Esc);
                    },
                    _ => {}
                }
            }
        }

        if pressed_keys.contains(&KeyCode::Esc) {
            break;
        }
        // Handle CCW Rotation
        if pressed_keys.contains(&KeyCode::Char('a')) {
            player_a -= (SPEED * 0.75) * DELTA;
        }
        // Handle CW Rotation
        if pressed_keys.contains(&KeyCode::Char('d')) {
            player_a += (SPEED * 0.75) * DELTA;
        }
        // Handle Forwards movement & collision
        if pressed_keys.contains(&KeyCode::Char('w')) {
            player_x += player_a.sin() * SPEED * DELTA;
            player_y += player_a.cos() * SPEED * DELTA;
            if map[player_y as usize][player_x as usize] == '#' {
                player_x -= player_a.sin() * SPEED * DELTA;
                player_y -= player_a.cos() * SPEED * DELTA;
            }
        }
        // Handle backwards movement & collision
        if pressed_keys.contains(&KeyCode::Char('s')) {
            player_x -= player_a.sin() * SPEED * DELTA;
            player_y -= player_a.cos() * SPEED * DELTA;
            if map[player_y as usize][player_x as usize] == '#' {
                player_x += player_a.sin() * SPEED * DELTA;
                player_y += player_a.cos() * SPEED * DELTA;
            }
        }
        // Handle rightwards movement & collision
        if pressed_keys.contains(&KeyCode::Char('q')) {
            player_x -= (player_a + 1.5).sin() * SPEED * DELTA;
            player_y -= (player_a + 1.5).cos() * SPEED * DELTA;
            if map[player_y as usize][player_x as usize] == '#' {
                player_x += (player_a + 1.5).sin() * SPEED * DELTA;
                player_y += (player_a + 1.5).cos() * SPEED * DELTA;
            }
        }
        // Handle rightwards movement & collision    
        if pressed_keys.contains(&KeyCode::Char('e')) {
            player_x -= (player_a - 1.5).sin() * SPEED * DELTA;
            player_y -= (player_a - 1.5).cos() * SPEED * DELTA;
            if map[player_y as usize][player_x as usize] == '#' {
                player_x += (player_a - 1.5).sin() * SPEED * DELTA;
                player_y += (player_a - 1.5).cos() * SPEED * DELTA;
            }
        }


        for x in 0..screen_width {
            // For each column, calculate the projected ray angle into world space
            let ray_angle = (player_a - FOV/2.0) + (x as f64 / screen_width as f64) * FOV;

            // Find distance to wall
            let step_size = 0.1;         // Increment size for ray casting, decrease to increase resolution
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
                        if p[2].1.acos() < bound { boundary = true; }
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
                    set(&mut screen, x, y, ' ', screen_width);
                } else if (y as f64) > ceiling && (y as f64)  <= floor {
                    set(&mut screen, x, y, shade, screen_width);
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

                    set(&mut screen, x, y, shade_2, screen_width);
                }
            }
        }


        // Display Map
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                set(&mut screen, x, y + 1, map[y][x], screen_width);
            }
        }

        set(&mut screen, player_x as usize, player_y as usize, 'P', screen_width);

        // Display Stats
        let mut stats = String::new();
        stats.push_str(&format!("A={:.2} X={:.2} Y={:.2} FPS={:.2}", player_a, player_x, player_y, (1.0 / elapsed_time)));
        let mut count = 0;
        for ch in stats.chars() {
            set(&mut screen, count, 0, ch, screen_width);
            count += 1;
        }


        // Build output string
        let mut output = String::new();

        for y in 0..screen_height {
            for x in 0..screen_width {
                output.push(screen[y * screen_width + x]);
            }
        }

        // Display Frame
        // Move curser to column 0, row 0 (x, y)
        stdout.execute(MoveTo(0, 0))?;

        // Write everything at once
        stdout.write_all(output.as_bytes())?;

        // Force it to appear
        stdout.flush()?;
    }
    
    Ok(())
}