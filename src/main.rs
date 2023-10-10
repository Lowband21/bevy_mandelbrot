use bevy::prelude::*;
use bevy::render::render_graph::RenderGraph;
use bevy_pancam::{PanCam, PanCamPlugin};

use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, BlendState, BufferAddress, ColorTargetState, ColorWrites, FragmentState,
    IndexFormat, MultisampleState, PipelineDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderRef, ShaderStages, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};

use std::borrow::Cow;

use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin::default())
        .add_systems(Startup, setup)
        .add_plugins(Material2dPlugin::<MandelbrotMaterial>::default())
        .run();
}

struct MandelbrotUniforms {
    offset: Vec2,
    zoom: f32,
    max_iterations: f32,
}

#[derive(Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "148ef27b-c53e-4bc2-982c-bb2b102e38f8"]
struct BasicMaterial {
    #[texture(0)]
    texture: Handle<Image>,
}

impl Material for BasicMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/basic_vertex.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/basic_fragment.wgsl".into()
    }
}

#[derive(Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "148ef22b-c53e-4bc2-982c-bb2b102e38f8"]
struct MandelbrotMaterial {
    #[uniform(0)]
    max_iterations: u32,
    #[uniform(1)]
    color_scale: f32,
    #[uniform(2)]
    offset: Vec2,
    #[uniform(3)]
    zoom: f32,
    #[texture(4)]
    #[sampler(5)]
    colormap_texture: Handle<Image>,
}
impl Material2d for MandelbrotMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/mandelbrot_vertex.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/mandelbrot_fragment.wgsl".into()
    }
}

//fn setup(
//    mut commands: Commands,
//    asset_server: Res<AssetServer>,
//    mut meshes: ResMut<Assets<Mesh>>,
//    mut materials: ResMut<Assets<BasicMaterial>>,
//) {
//    let texture_handle = asset_server.load("viridis.png"); // This will be our test texture
//
//    commands
//        .spawn(Camera2dBundle {
//            transform: Transform::from_xyz(0.0, 0.0, 10.0), // Move the camera back
//            ..Default::default()
//        })
//        .insert(PanCam {
//            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
//            enabled: true,
//            zoom_to_cursor: true,
//            min_scale: 1.,
//            max_scale: Some(40.),
//            ..Default::default()
//        });
//
//    let material = BasicMaterial {
//        texture: texture_handle,
//    };
//    let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Quad {
//        size: Vec2::new(2.0, 2.0),
//        flip: false,
//    }));
//    let material_handle: Handle<BasicMaterial> = materials.add(material);
//
//    commands.spawn(MaterialMeshBundle {
//        mesh: mesh,
//        material: material_handle,
//        transform: Transform::from_xyz(0.0, 0.0, 0.0),
//        ..Default::default()
//    });
//}
//
fn mandelbrot_uniform_update_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
) {
    for (id, material) in materials.iter_mut() {
        // Here, you can update your uniforms as you want, for example:
        material.offset += Vec2::new(time.delta_seconds(), 0.0);
        // ... and any other updates you'd like to do.
    }
}

fn prepare_mandelbrot_material(
    uniforms: MandelbrotUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<MandelbrotMaterial>>,
) -> Handle<MandelbrotMaterial> {
    let material = MandelbrotMaterial {
        max_iterations: uniforms.max_iterations as u32,
        color_scale: 1.0, // You can adjust this as needed.
        offset: uniforms.offset,
        zoom: uniforms.zoom,
        colormap_texture: colormap_texture_handle,
    };
    materials.add(material)
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
) {
    let colormap_texture_handle = asset_server.load("viridis.png");

    // Add the camera with PanCam component for panning and zooming
    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
        enabled: true,
        zoom_to_cursor: true,
        min_scale: 1.,
        max_scale: Some(40.),
        ..Default::default()
    });

    let uniforms = MandelbrotUniforms {
        offset: Vec2::new(0.0, 0.0),
        zoom: 1.0,
        max_iterations: 1000.0,
    };
    let mandelbrot_material_handle =
        prepare_mandelbrot_material(uniforms, colormap_texture_handle, &mut materials);
    let mandelbrot_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(2.0, 2.0),
        flip: false,
    })));

    commands.spawn(MaterialMesh2dBundle {
        mesh: mandelbrot_mesh,
        material: mandelbrot_material_handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}
