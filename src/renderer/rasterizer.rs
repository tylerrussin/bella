use crate::geometry::Triangle;
use crate::math::vec3::Vec3;

fn set_pixel(frame: &mut [u8], x: usize, y: usize, width: usize, r: u8, g: u8, b: u8) {
    let i = (y * width + x) * 4;
    frame[i] = r;
    frame[i + 1] = g;
    frame[i + 2] = b;
    frame[i + 3] = 255;
}

pub fn reset_screen(frame: &mut [u8]) {
    // Black out the screen
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 0; // R
        pixel[1] = 0; // G
        pixel[2] = 0; // B
        pixel[3] = 255; // A
    }
}

fn edge(a: Vec3, b: Vec3, p: Vec3) -> f32 {
    (p.x - a.x) * (b.y - a.y)
        - (p.y - a.y) * (b.x - a.x)
}

pub fn fill_triangle(
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