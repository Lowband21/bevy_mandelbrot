use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

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
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath, Asset)]
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
// Utility function to prepare and return a julia material with the given uniforms.
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

// Entity to represent Burning Ship
#[derive(Resource)]
pub struct BurningShipEntity(pub Option<Entity>);
impl Default for BurningShipEntity {
    fn default() -> Self {
        BurningShipEntity(None)
    }
}

// Struct to store uniform parameters for the burning ship.
pub struct BurningShipUniforms {
    pub color_scale: f32,
    pub max_iterations: f32,
}

// Sierpinski Triangle material definition.
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath, Asset)]
#[uuid = "0e17159a-ca90-4cd1-a40e-ab12c9455c11"]
pub struct BurningShipMaterial {
    #[uniform(0)]
    pub color_scale: f32,
    #[uniform(1)]
    pub max_iterations: f32,
    #[texture(4)]
    #[sampler(5)]
    colormap_texture: Handle<Image>,
}

impl Material2d for BurningShipMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/burning_ship_fragment.wgsl".into()
    }
}

// Utility function to prepare and return a burning ship material with the given uniforms.
pub fn prepare_burning_ship_material(
    uniforms: &BurningShipUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<BurningShipMaterial>>,
) -> Handle<BurningShipMaterial> {
    let material = BurningShipMaterial {
        color_scale: uniforms.color_scale,
        max_iterations: uniforms.max_iterations,
        colormap_texture: colormap_texture_handle,
    };
    materials.add(material)
}
