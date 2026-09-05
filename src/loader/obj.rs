use crate::geometry::{Triangle, Mesh};
use crate::math::vec3::Vec3;

use std::io::{BufRead, BufReader};

pub fn load_obj(file_name: &str) -> Mesh {
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
                    c: (255, 255, 255),
                    avg_z: 0.0,
                });
            }
            _ => {}
        }
    }

    Mesh { tris: triangles_list }
}