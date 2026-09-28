use crate::geometry::Triangle;
use crate::math::vec3::Vec3;

pub fn triangle_clip_against_plane(plane_p: Vec3, mut plane_n: Vec3, in_tri: &Triangle, out_tri1: &mut Triangle, out_tri2: &mut Triangle) -> usize {
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