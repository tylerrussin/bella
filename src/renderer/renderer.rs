use crate::geometry::{Mesh, Triangle};
use crate::math::vec3::Vec3;
use crate::math::mat4::{
    Mat4x4,
    identity,
    rotation_x,
    rotation_z,
    translation,
    multiply_matrix,
    matrix_point_at,
    matrix_quick_inverse,
};

use super::{
    calculate_lighting,
    fill_triangle,
    reset_screen,
    triangle_clip_against_plane,
};

pub fn render_screen(
    frame: &mut [u8],
    depth_buffer: &mut [f32],
    mesh: &Mesh,
    camera: Vec3,
    look_dir: Vec3,
    projection: &Mat4x4,
    theta: f32,
    width: u32,
    height: u32,
) {
    reset_screen(frame);
    depth_buffer.fill(f32::INFINITY);

    let mat_rot_z = rotation_z(theta * 0.5);
    let mat_rot_x = rotation_x(theta);

    let mat_trans = translation(0.0, 0.0, 16.0);

    let mut mat_world = identity();
    mat_world = multiply_matrix(&mat_world, &mat_rot_x);
    mat_world = multiply_matrix(&mat_world, &mat_rot_z);
    mat_world = multiply_matrix(&mat_world, &mat_trans);


    let v_target = camera + look_dir;

    let v_up = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };


    let mat_camera: Mat4x4 = matrix_point_at(camera, v_target, v_up);

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
            uv: [
                tri.uv[0],
                tri.uv[1],
                tri.uv[2],
            ],
            c: (255, 255, 255),
            avg_z: 0.0,
        };

        let line1 = tri_transformed.p[1] - tri_transformed.p[0];
        let line2 = tri_transformed.p[2] - tri_transformed.p[0];

        let mut normal = line1.cross(line2);

        normal = normal.normalize();

        let camera_ray = tri_transformed.p[0] - camera;


        if (normal.dot(camera_ray)) < 0.0 {

            let color = calculate_lighting(normal);



            
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
                 uv: [
                    tri_transformed.uv[0],
                    tri_transformed.uv[1],
                    tri_transformed.uv[2],
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
                        clipped[n].p[0].matrix_multiply_vector(projection),
                        clipped[n].p[1].matrix_multiply_vector(projection),
                        clipped[n].p[2].matrix_multiply_vector(projection),
                    ],
                    uv: [
                        clipped[n].uv[0],
                        clipped[n].uv[1],
                        clipped[n].uv[2],
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
            let mut n_tris_to_add;

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
                depth_buffer,
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

}