// This attribute indicates that if the code is not compiled with debug assertions (i.e., in release mode),
// the application should run with a "windows" subsystem (i.e., without a console window in Windows OS).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Importing required modules and traits from the `bevy` crate.
use bevy::prelude::*;

// Local module import for custom camera controls.
mod pancam;
use crate::pancam::{PanCamConfig, PanCamPlugin, PanCamState};

// Additional imports from the `bevy` crate, useful for rendering and diagnostics.
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::reflect::TypePath;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// The main function to initialize and run the Bevy app.
fn main() {
    // Initializing the Bevy app and adding various plugins.
    let _app = App::new()
        // Uncomment to set a custom clear color for the renderer.
        //.insert_resource(ClearColor(Color::hex("071f3c").unwrap()))
        .init_resource::<FractalType>()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            // Add an inspector that can be toggled using the Escape key.
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(PanCamPlugin::default()) // Custom camera control plugin.
        .add_plugins(LogDiagnosticsPlugin::default()) // For logging diagnostics.
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // Diagnostics for frame time.
        .add_systems(Startup, setup) // Setup function called at startup.
        .add_plugins(Material2dPlugin::<MandelbrotMaterial>::default()) // Plugin for 2D materials.
        .add_plugins(Material2dPlugin::<JuliaMaterial>::default()) // Plugin for 2D materials.
        //.add_systems(Update, mandelbrot_uniform_update_system) // Update system for Mandelbrot material.
        .add_systems(Update, fractal_toggle_system) // Update system for Mandelbrot material.
        .add_systems(Update, fractal_update_system)
        .run();
}


// New component to determine the current fractal type
#[derive(Resource)]
enum FractalType {
    Mandelbrot,
    Julia,
}

impl Default for FractalType {
    fn default() -> Self {
        FractalType::Mandelbrot
    }
}

// Struct to store uniform parameters for the Mandelbrot fractal.
struct MandelbrotUniforms {
    color_scale: f32,
    offset: Vec2,
    zoom: f32,
    max_iterations: f32,
}

// Mandelbrot material definition. It holds parameters and texture for the Mandelbrot fractal.
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
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

// Julia material definition. It holds parameters and texture for the Julia fractal.
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "258ef34b-d54f-4bc3-993b-bc3e203a48f9"]
struct JuliaMaterial {
    #[uniform(0)]
    color_scale: f32,
    #[uniform(0)]
    max_iterations: u32,
    #[uniform(0)]
    offset: Vec2,
    #[uniform(0)]
    zoom: f32,
    #[uniform(0)]
    c: Vec2, // Julia constant
    #[texture(4)]
    #[sampler(5)]
    colormap_texture: Handle<Image>,
}

impl Material2d for MandelbrotMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mandelbrot_fragment.wgsl".into()
    }
}
impl Material2d for JuliaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/julia_fragment.wgsl".into()
    }
}

// System to update the Mandelbrot material's color_scale based on time.
fn mandelbrot_uniform_update_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
) {
    for (_, mut material) in materials.iter_mut() {
        material.color_scale = 0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * 0.5).sin());
    }
}

// System to update the material based on the current fractal type
fn fractal_update_system(
    time: Res<Time>,
    fractal_type: Res<FractalType>,
    mut mandelbrot_materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>,
    mut mandelbrot_query: Query<(&mut MandelbrotMaterial, &mut Visibility), Without<JuliaMaterial>>,
    mut julia_query: Query<(&JuliaMaterial, &mut Visibility), Without<MandelbrotMaterial>>,
) {
    match *fractal_type {
        FractalType::Mandelbrot => {
            println!("Fractal Type is Mandelbrot");
            for (_, mut material) in mandelbrot_materials.iter_mut() {
                material.color_scale = 0.5 * (1.0 + (time.elapsed_seconds() as f32 * 0.5).sin());
            }

            for (mut material, mut visibility) in mandelbrot_query.iter_mut() {
                material.color_scale = 0.5 * (1.0 + (time.elapsed_seconds() as f32 * 0.5).sin());
                *visibility = Visibility::Visible;
            }
            for (_, mut visibility) in julia_query.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        },
        FractalType::Julia => {
            println!("Fractal Type is Julia");
            for (_, mut material) in julia_materials.iter_mut() {
                material.color_scale = 0.5 * (1.0 + (time.elapsed_seconds() as f32 * 0.5).sin());
            }

            for (_, mut visibility) in julia_query.iter_mut() {
                *visibility = Visibility::Visible;
            }
            for (_, mut visibility) in mandelbrot_query.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}




// System to toggle between Mandelbrot and Julia fractals
fn fractal_toggle_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut fractal_type: ResMut<FractalType>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Space was pressed");
        *fractal_type = match *fractal_type {
            FractalType::Mandelbrot => FractalType::Julia,
            FractalType::Julia => FractalType::Mandelbrot,
        };
    }
}


// Utility function to prepare and return a Mandelbrot material with the given uniforms.
fn prepare_mandelbrot_material(
    uniforms: &MandelbrotUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<MandelbrotMaterial>>,
) -> Handle<MandelbrotMaterial> {
    let material = MandelbrotMaterial {
        max_iterations: uniforms.max_iterations as u32,
        color_scale: uniforms.color_scale,
        offset: uniforms.offset,
        zoom: uniforms.zoom,
        colormap_texture: colormap_texture_handle,
    };
    materials.add(material)
}
// Utility function to prepare and return a Mandelbrot material with the given uniforms.
fn prepare_julia_material(
    uniforms: &MandelbrotUniforms,
    colormap_texture_handle: Handle<Image>,
    materials: &mut ResMut<Assets<JuliaMaterial>>,
) -> Handle<JuliaMaterial> {
    let material = JuliaMaterial {
        c: Vec2 { x: 1.0, y: 1.0 },
        max_iterations: uniforms.max_iterations as u32,
        color_scale: uniforms.color_scale,
        offset: uniforms.offset,
        zoom: uniforms.zoom,
        colormap_texture: colormap_texture_handle,
    };
    materials.add(material)
}

// The setup function initializes entities in the Bevy app, such as the Mandelbrot mesh and camera.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>,
) {
    // Load colormap texture for the Mandelbrot material.
    let colormap_texture_handle = asset_server.load("gradient.png");

    // Define uniform values for the Mandelbrot material.
    let uniforms = MandelbrotUniforms {
        color_scale: 0.5,
        offset: Vec2::new(5.0, 0.0),
        zoom: 1.00,
        max_iterations: 1000.0,
    };

    // Create and store Mandelbrot material.
    let mandelbrot_material_handle =
        prepare_mandelbrot_material(&uniforms, colormap_texture_handle.clone(), &mut materials);

        // Create and store Julia material.
    let julia_material_handle =
        prepare_julia_material(&uniforms, colormap_texture_handle, &mut julia_materials);

    // Create a large quad mesh.
    let mesh = Mesh::from(shape::Quad {
        size: Vec2::new(10000.0, 10000.0),
        flip: false,
    });
    let mandelbrot_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));

    // Spawn the Mandelbrot mesh with its material in the world.
    commands.spawn(MaterialMesh2dBundle {
        mesh: mandelbrot_mesh.clone(),
        material: mandelbrot_material_handle,
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        visibility: Visibility::Visible,
        ..Default::default()
    });

        // Spawn the Julia mesh with its material in the world (you can start it hidden or at a different position).
    commands.spawn(MaterialMesh2dBundle {
        mesh: mandelbrot_mesh/* ... appropriate mesh, possibly the same as Mandelbrot ... */,
        material: julia_material_handle,
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        visibility: Visibility::Hidden,
        ..Default::default()
    });

    // Add a camera with custom pan and zoom capabilities.
    commands.spawn((
        Camera2dBundle::default(),
        PanCamConfig {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.00012,
            max_scale: Some(7.5),
            min_x: Some(-5000.0),
            min_y: Some(-5000.0),
            max_x: Some(5000.0),
            max_y: Some(5000.0),
            pixels_per_line: 100.0,
            base_zoom_multiplier: 10.0,
            shift_multiplier_normal: 10.0,
            shift_multiplier_shifted: 100.0,
            animation_scale: 3.0,
            ..default()
        },
        PanCamState {
            current_zoom: 1.0,
            target_zoom: 4.5,
            is_zooming: true,
            target_translation: None,
            delta_zoom_translation: None,
            ..default()
        },
    ));
}
