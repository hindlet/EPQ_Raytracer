use graphics::{Mesh, ColouredVertex};
use maths::{Vector3, lerp};

const PHI: f32 = 1.618033989; // (1 + sqrt(5)) / 2
const SCALE: f32 = 1.902113033; // sqrt(1 + phi^2)

pub fn gen_icosphere(
    radius: f32,
    centre: impl Into<Vector3>,
    colour: [f32; 4],
    num_subdivisions: usize,
) -> Mesh<ColouredVertex> {
    let centre: Vector3 = centre.into();

    // set up initial icosohedron
    let dist1 = PHI * radius / SCALE;
    let dist2 = radius / SCALE;
    let mut vertices = vec![
        Vector3::new(-dist2, 0.0, dist1),
        Vector3::new(dist2, 0.0, dist1),
        Vector3::new(-dist2, 0.0, -dist1),
        Vector3::new(dist2, 0.0, -dist1),

        Vector3::new(0.0, dist1, dist2),
        Vector3::new(0.0, dist1, -dist2),
        Vector3::new(0.0, -dist1, dist2),
        Vector3::new(0.0,- dist1, -dist2),

        Vector3::new(dist1, dist2, 0.0),
        Vector3::new(-dist1, dist2, 0.0),
        Vector3::new(dist1, -dist2, 0.0),
        Vector3::new(-dist1, -dist2, 0.0),
    ];

    let mut indices: Vec<u32> = vec![
        0, 4, 1,
        0, 9, 4,
        9, 5, 4,
        4, 5, 8,
        4, 8, 1,

        8, 10, 1,
        8, 3, 10,
        5, 3, 8,
        5, 2, 3,
        2, 7, 3,

        7, 10, 3,
        7, 6, 10,
        7, 11, 6,
        11, 0, 6,
        0, 1, 6,

        6, 1, 10,
        9, 0, 11,
        9, 11, 2,
        9, 2, 5,
        7, 2, 11
    ];
    let mut next_indices: Vec<u32> = Vec::new();

    for _ in 0..num_subdivisions {
        next_indices.clear();
        // loop through all triangles
        for i in (0..indices.len()).step_by(3) {
            let v1 = vertices[indices[i + 0] as usize];
            let v2 = vertices[indices[i + 1] as usize];
            let v3 = vertices[indices[i + 2] as usize];

            let new_index = vertices.len() as u32;

            let new_vert1 = lerp(v1, v2, 0.5);
            vertices.push(new_vert1.normalised() * radius);
            let new_vert2 = lerp(v2, v3, 0.5);
            vertices.push(new_vert2.normalised() * radius);
            let new_vert3 = lerp(v1, v3, 0.5);
            vertices.push(new_vert3.normalised() * radius);

            next_indices.push(indices[i + 0]);
            next_indices.push(new_index + 0);
            next_indices.push(new_index + 2);

            next_indices.push(indices[i + 1]);
            next_indices.push(new_index + 1);
            next_indices.push(new_index + 0);

            next_indices.push(indices[i + 2]);
            next_indices.push(new_index + 2);
            next_indices.push(new_index + 1);

            next_indices.push(new_index + 0);
            next_indices.push(new_index + 1);
            next_indices.push(new_index + 2);
        }
        indices = next_indices.clone();
    }


    let mut mesh_vertices = Vec::new();
    for vertex in vertices {
        mesh_vertices.push(ColouredVertex{position: (vertex + centre).into(), colour: colour});
    }
    let mut out_mesh = Mesh::new(mesh_vertices, indices);
    out_mesh.smooth_shade();
    out_mesh.recalculate_normals();
    out_mesh
}