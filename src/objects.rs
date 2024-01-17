use super::raytrace_pipeline::raytrace_shader;
use super::materials::LambertianMaterial;
use graphics::{Mesh, PositionVertex, Normal};
use graphics::all_vulkano::buffer::BufferContents;

/// Sphere representation
#[derive(Debug, Clone)]
pub struct Sphere {
    pub centre: [f32; 3],
    pub radius: f32,
    pub material: raytrace_shader::RayTracingMaterial
}

impl Into<raytrace_shader::Sphere> for Sphere {
    fn into(self) -> raytrace_shader::Sphere {
        raytrace_shader::Sphere {
            centre: self.centre,
            radius: self.radius,
            material: self.material
        }
    }
}

pub fn get_null_sphere() -> Sphere {
    Sphere {
        centre: [0.0; 3],
        radius: 0.0,
        material: LambertianMaterial{colour: [1.0; 3]}.into()
    }
}


/// Mesh Representation
#[derive(Debug, Clone)]
pub struct RayTracingMesh<T: graphics::Position + BufferContents + Copy + Clone> {
    pub mesh: Mesh<T>,
    pub material: raytrace_shader::RayTracingMaterial
}

pub fn get_null_mesh() -> RayTracingMesh<PositionVertex> {
    let mut mesh = Mesh::new(vec![PositionVertex{position: [0.0; 3]}], vec![0, 0, 0]);
    mesh.set_normals(vec![Normal{normal: [1.0; 3]}]);
    RayTracingMesh {
        mesh: mesh,
        material: LambertianMaterial{colour: [1.0; 3]}.into()
    }
}