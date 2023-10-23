
use bevy::prelude::*;

use bevy::reflect::TypePath;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d};

// These handles will store the references to our spawned entities.
#[derive(Resource)]
pub struct JuliaEntity(pub Option<Entity>);
impl Default for JuliaEntity {
    fn default() -> Self {
        JuliaEntity(None)
    }
}

// Struct to store uniform parameters for the Mandelbrot fractal.
pub struct JuliaUniforms {
    pub color_scale: f32,
    pub max_iterations: f32,
}

// Julia material definition. It holds parameters and texture for the Julia fractal.
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "258ef34b-d54f-4bc3-993b-bc3e203a48f9"]
pub struct JuliaMaterial {
    #[uniform(0)]
    pub color_scale: f32,
    #[uniform(1)]
    pub max_iterations: f32,
    #[uniform(2)]
    pub c: Vec2, // Julia constant
    #[texture(4)]
    #[sampler(5)]
    colormap_texture: Handle<Image>,
}

impl Material2d for JuliaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/julia_fragment.wgsl".into()
    }
}
// Utility function to prepare and return a Mandelbrot material with the given uniforms.
pub fn prepare_julia_material(
    uniforms: &JuliaUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<JuliaMaterial>>,
) -> Handle<JuliaMaterial> {
    let material = JuliaMaterial {
        color_scale: uniforms.color_scale,
        max_iterations: uniforms.max_iterations,
        c: Vec2 { x: 0.3, y: 0.8 },
        colormap_texture: colormap_texture_handle,
    };
    materials.add(material)
}
