#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;

mod pancam;
use crate::pancam::{PanCam, PanCamPlugin};
use bevy::reflect::TypePath;

use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::reflect::TypeUuid;

fn main() {
    let app = App::new()
        //.insert_resource(ClearColor(Color::hex("071f3c").unwrap()))
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_plugins(Material2dPlugin::<MandelbrotMaterial>::default())
        .add_systems(Update, mandelbrot_uniform_update_system)
        .run();
}

struct MandelbrotUniforms {
    color_scale: f32,
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
    color_scale: f32,
    #[uniform(0)]
    max_iterations: u32,
    #[uniform(0)]
    offset: Vec2,
    #[uniform(0)]
    zoom: f32,
    #[texture(4)]
    #[sampler(5)]
    colormap_texture: Handle<Image>,
}
impl Material2d for MandelbrotMaterial {
    //fn vertex_shader() -> ShaderRef {
    //    "shaders/mandelbrot_vertex.wgsl".into()
    //}

    fn fragment_shader() -> ShaderRef {
        "shaders/mandelbrot_fragment.wgsl".into()
    }
}
fn mandelbrot_uniform_update_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
) {
    for (_, mut material) in materials.iter_mut() {
        // Oscillate color_scale between 0 and 1 using a sinusoidal function
        material.color_scale = 0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * 0.5).sin());

        //println!(
        //    "{:?}, {:?}, {:?}, {:?}, {:?}",
        //    material.offset,
        //    material.zoom,
        //    material.color_scale,
        //    material.max_iterations,
        //    material.colormap_texture,
        //);
    }
}

fn prepare_mandelbrot_material(
    uniforms: MandelbrotUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<MandelbrotMaterial>>,
) -> Handle<MandelbrotMaterial> {
    let material = MandelbrotMaterial {
        max_iterations: uniforms.max_iterations as u32,
        color_scale: uniforms.color_scale, // You can adjust this as needed.
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
    let colormap_texture_handle = asset_server.load("gradient.png");

    let uniforms = MandelbrotUniforms {
        color_scale: 0.5,
        offset: Vec2::new(5.0, 0.0),
        zoom: 1.00,
        max_iterations: 1000.0,
    };
    let mandelbrot_material_handle =
        prepare_mandelbrot_material(uniforms, colormap_texture_handle, &mut materials);
    let mesh = Mesh::from(shape::Quad {
        size: Vec2::new(10000.0, 10000.0),
        flip: false,
    });

    let mandelbrot_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));

    commands.spawn(MaterialMesh2dBundle {
        mesh: mandelbrot_mesh,
        material: mandelbrot_material_handle,
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // Add the camera with PanCam component for panning and zooming
    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
        enabled: true,
        zoom_to_cursor: true,
        min_scale: 0.0012,
        max_scale: Some(7.5),
        min_x: Some(-5000.0),
        min_y: Some(-5000.0),
        max_x: Some(5000.0),
        max_y: Some(5000.0),
        current_zoom: 1.0,
        target_zoom: 7.0,
        is_zooming: true,
        target_translation: None,
        delta_zoom_translation: None,
        ..default()
    });
    //commands.spawn(Camera2dBundle::default());
}
