use crate::math::vec3::Vec3;
use rand::Rng;


pub fn calculate_lighting(normal: Vec3) -> (u8, u8, u8) {
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

    (grey, grey, grey)
}