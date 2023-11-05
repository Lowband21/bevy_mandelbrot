
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d};

// Entity to represent Barnsley Fern
#[derive(Resource)]
pub struct BurningShipEntity(pub Option<Entity>);
impl Default for BurningShipEntity {
    fn default() -> Self {
        BurningShipEntity(None)
    }
}

// Struct to store uniform parameters for the Sierpinski Triangle.
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

// Utility function to prepare and return a Sierpinski Triangle material with the given uniforms.
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
