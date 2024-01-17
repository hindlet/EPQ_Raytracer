use super::raytrace_pipeline::raytrace_shader;


pub struct CustomMaterial {
    pub colour: [f32; 3],
    pub emission_colour: [f32; 3],
    pub emission_strength: f32,
    pub smoothness: f32,
    pub fuzz: f32,
    pub specular_probability: f32,
}

impl Into<raytrace_shader::RayTracingMaterial> for CustomMaterial {
    fn into(self) -> raytrace_shader::RayTracingMaterial {
        raytrace_shader::RayTracingMaterial {
            colour: [self.colour[0], self.colour[1], self.colour[2], 0.0],
            emission: [self.emission_colour[0], self.emission_colour[1], self.emission_colour[2], self.emission_strength],
            settings: [self.specular_probability, self.smoothness, self.fuzz, 0.0]
        }
    }
}

impl Default for CustomMaterial {
    fn default() -> Self {
        CustomMaterial {
            colour: [0.5; 3],
            emission_colour: [0.0; 3],
            emission_strength: 0.0,
            smoothness: 0.0,
            fuzz: 0.0,
            specular_probability: 0.0,
        }
    }
}



pub struct LambertianMaterial {
    pub colour: [f32; 3],
}

impl Into<raytrace_shader::RayTracingMaterial> for LambertianMaterial {
    fn into(self) -> raytrace_shader::RayTracingMaterial {
        raytrace_shader::RayTracingMaterial {
            colour: [self.colour[0], self.colour[1], self.colour[2], 0.0],
            emission: [0.0; 4],
            settings: [1.0, 0.0, 0.0, 0.0]
        }
    }
}

pub struct MetalMaterial {
    pub colour: [f32; 3],
    pub smoothness: f32,
    pub fuzz: f32
}

impl Into<raytrace_shader::RayTracingMaterial> for MetalMaterial {
    fn into(self) -> raytrace_shader::RayTracingMaterial {
        raytrace_shader::RayTracingMaterial {
            colour: [self.colour[0], self.colour[1], self.colour[2], 0.0],
            emission: [0.0; 4],
            settings: [1.0, self.smoothness, self.fuzz, 0.0]
        }
    }
}

pub struct LightMaterial {
    pub emission: [f32; 4]
}

impl Into<raytrace_shader::RayTracingMaterial> for LightMaterial {
    fn into(self) -> raytrace_shader::RayTracingMaterial {
        raytrace_shader::RayTracingMaterial {
            colour: [1.0; 4],
            emission: self.emission,
            settings: [1.0, 1.0, 0.0, 0.0]
        }
    }
}

pub struct InvisLightMaterial {
    pub emission: [f32; 4]
}

impl Into<raytrace_shader::RayTracingMaterial> for InvisLightMaterial {
    fn into(self) -> raytrace_shader::RayTracingMaterial {
        raytrace_shader::RayTracingMaterial {
            colour: [1.0; 4],
            emission: self.emission,
            settings: [0.0, 1.0, 0.0, 1.0]
        }
    }
}