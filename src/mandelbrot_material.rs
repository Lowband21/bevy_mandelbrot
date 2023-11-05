
use bevy::prelude::*;

use bevy::reflect::TypePath;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d};

#[derive(Resource)]
pub struct MandelbrotEntity(pub Option<Entity>);
impl Default for MandelbrotEntity {
    fn default() -> Self {
        MandelbrotEntity(None)
    }
}

// Struct to store uniform parameters for the Mandelbrot fractal.
pub struct MandelbrotUniforms {
    pub color_scale: f32,
    pub max_iterations: f32,
}

// Mandelbrot material definition. It holds parameters and texture for the Mandelbrot fractal.
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath, Asset)]
#[uuid = "148ef22b-c53e-4bc2-982c-bb2b102e38f8"]
pub struct MandelbrotMaterial {
    #[uniform(0)]
    pub color_scale: f32,
    #[uniform(1)]
    pub max_iterations: f32,
    #[uniform(2)]
    pub zoom: f32,
    #[uniform(3)]
    pub offset: Vec2,
    #[uniform(6)]
    pub global_offset: Vec2,
    #[texture(4)]
    #[sampler(5)]
    colormap_texture: Handle<Image>,
}

impl Material2d for MandelbrotMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mandelbrot_fragment.wgsl".into()
    }
}

// Utility function to prepare and return a Mandelbrot material with the given uniforms.
pub fn prepare_mandelbrot_material(
    uniforms: &MandelbrotUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<MandelbrotMaterial>>,
) -> Handle<MandelbrotMaterial> {
    let material = MandelbrotMaterial {
        max_iterations: uniforms.max_iterations,
        color_scale: uniforms.color_scale,
        zoom: 4.5,
        offset: Vec2 { x: 0.0, y: 0.0 },
        global_offset: Vec2 { x: 0.0, y: 0.0 } / 4.5,
        colormap_texture: colormap_texture_handle,
    };
    materials.add(material)
}
