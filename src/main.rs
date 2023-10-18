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

use bevy_egui::{egui, EguiContexts, EguiPlugin};

use audioviz::io::{Device, Input, InputController};
use bevy::audio::*;

use audioviz::spectrum::{
    config::{Interpolation, ProcessorConfig, StreamConfig},
    stream::Stream,
    Frequency,
};

#[derive(Component)]
struct MyMusic;
// The main function to initialize and run the Bevy app.
fn main() {
    // Initializing the Bevy app and adding various plugins.
    let _app = App::new()
        // Uncomment to set a custom clear color for the renderer.
        //.insert_resource(ClearColor(Color::hex("071f3c").unwrap()))
        .init_resource::<FractalType>()
        .init_resource::<MandelbrotEntity>()
        .init_resource::<JuliaEntity>()
        .init_resource::<AnimationUpdateToggle>()
        .init_resource::<MusicUpdateToggle>()
        .init_resource::<LastRms>()
        .add_plugins(DefaultPlugins)
        //.add_plugins(
        //    // Add an inspector that can be toggled using the Escape key.
        //    WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        //)
        .add_plugins(EguiPlugin)
        .add_plugins(PanCamPlugin::default()) // Custom camera control plugin.
        .add_plugins(LogDiagnosticsPlugin::default()) // For logging diagnostics.
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // Diagnostics for frame time.
        .add_systems(Startup, setup) // Setup function called at startup.
        .add_plugins(Material2dPlugin::<MandelbrotMaterial>::default()) // Plugin for 2D materials.
        .add_plugins(Material2dPlugin::<JuliaMaterial>::default()) // Plugin for 2D materials.
        .add_systems(
            Update,
            (mandelbrot_uniform_update_system, mandelbrot_toggle_system),
        ) // Update system for Mandelbrot material.
        .add_systems(Update, fractal_toggle_system) // Update system for Mandelbrot material.
        .add_systems(Update, fractal_update_system)
        .add_systems(Update, uniform_update_ui_system)
        //.add_systems(Startup, (setup_audio))
        .add_systems(Update, (update_system_sound_level, update_from_music))
        .run();
}

#[derive(Component)]
struct SystemSoundLevel {
    value: f32,
}

#[derive(Resource, Default)]
struct LastRms(f32);

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music_handle = asset_server.load("audio/1-06 Solitude Is Bliss.flac");

    commands.spawn((
        AudioBundle {
            source: music_handle,
            ..Default::default()
        },
        MyMusic,
        SystemSoundLevel { value: 0.0 }, // New component
    ));
}
fn update_system_sound_level(
    mut sound_level_query: Query<&mut SystemSoundLevel, With<MyMusic>>,
    last_rms: ResMut<LastRms>,
    toggle: Res<MusicUpdateToggle>,
) {
    if !toggle.active {
        return;
    }
    // Initialize audio input and spectrum visualizer stream just once.
    // You might consider moving this to a startup system or a resource.
    let mut audio_input = Input::new();
    let (_channel_count, _sampling_rate, input_controller) =
        audio_input.init(&Device::DefaultInput, None).unwrap();
    let mut stream: Stream = Stream::new(StreamConfig::default());
    let mut sound_level = 0.0;

    loop {
        if let Some(data) = input_controller.pull_data() {
            sound_level = compute_sound_level_from_frequencies(&data, last_rms);
            stream.push_data(data);
            stream.update();
        } else {
            println!("Failed to pull data");
        }

        //stream.update();
        // Retrieve frequencies
        //let frequencies = stream.get_frequencies();
        //println!("{}", frequencies[0].len());
        // Compute sound level from frequencies (this could be RMS or some other measure)
        //let flattened_frequencies: Vec<Frequency> = frequencies.into_iter().flatten().collect();

        // Update SystemSoundLevel component
        for mut system_sound_level in sound_level_query.iter_mut() {
            system_sound_level.value = sound_level;
        }
        break;
    }
}

fn compute_sound_level_from_frequencies(
    frequencies: &Vec<f32>,
    mut last_rms: ResMut<LastRms>,
) -> f32 {
    // Define bass frequency range indices
    let bass_range = 0..50;

    // Filter out only the bass frequencies
    let bass_frequencies: Vec<f32> = frequencies[bass_range.clone()].to_vec();

    if bass_frequencies.is_empty() {
        println!("Bass frequencies array is empty");
        return 0.0;
    }

    let mut sum_of_squares = 0.0;

    for &frequency in bass_frequencies.iter() {
        if frequency.is_nan() || frequency.is_infinite() {
            println!("Invalid frequency value: {}", frequency);
            continue;
        }
        sum_of_squares += frequency.powi(2);
    }

    if sum_of_squares <= 0.0 {
        println!("Sum of squares is non-positive: {}", sum_of_squares);
        return 0.0;
    }

    let rms = (sum_of_squares / bass_frequencies.len() as f32).sqrt();

    if rms.is_nan() || rms.is_infinite() {
        println!("Invalid RMS: {}", rms);
        return 0.0;
    }

    // EMA smoothing
    // Alpha is the weight of the new sample, should be between 0 and 1.
    // Higher alpha discounts older observations faster.
    let alpha = 0.1;
    last_rms.0 = alpha * rms + (1.0 - alpha) * last_rms.0;

    // Debugging
    println!("Smoothed RMS: {}", last_rms.0);

    // Return the smoothed RMS
    last_rms.0 * 1000.0 + 0.5
}

fn update_from_music(
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    music_controller: Query<(&AudioSink, &SystemSoundLevel), With<MyMusic>>,
    toggle: Res<MusicUpdateToggle>,
) {
    if !toggle.active {
        return;
    }
    for (_audio_sink, sound_level) in music_controller.iter() {
        if let Some(mandelbrot_material) = materials.iter_mut().next() {
            mandelbrot_material.1.color_scale = sound_level.value;
        }
    }
}
fn uniform_update_ui_system(
    mut ctx: EguiContexts,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>,
    mut pancam_query: Query<&mut PanCamState>,
) {
    let context = ctx.ctx_mut();
    egui::Window::new("Update Uniforms").show(context, |ui| {
        if let Some(mandelbrot_material) = materials.iter_mut().next() {
            ui.horizontal(|ui| {
                ui.label("Mandelbrot Color Scale:");
                ui.add(egui::Slider::new(
                    &mut mandelbrot_material.1.color_scale,
                    0.0..=1.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Mandelbrot Iterations:");
                ui.add(egui::Slider::new(
                    &mut mandelbrot_material.1.max_iterations,
                    0.0..=100000.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Mandelbrot Zoom:");
                ui.add(egui::Slider::new(
                    &mut pancam_query.get_single_mut().unwrap().target_zoom,
                    0.0..=100.0,
                ));
            });
        }
        if let Some(julia_material) = julia_materials.iter_mut().next() {
            ui.horizontal(|ui| {
                ui.label("Julia Color Scale:");
                ui.add(egui::Slider::new(
                    &mut julia_material.1.color_scale,
                    0.0..=1.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Julia Iterations:");
                ui.add(egui::Slider::new(
                    &mut julia_material.1.max_iterations,
                    0.0..=100000.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Julia c.x:");
                ui.add(egui::Slider::new(&mut julia_material.1.c.x, -2.0..=2.0));
            });
            ui.horizontal(|ui| {
                ui.label("Julia c.y:");
                ui.add(egui::Slider::new(&mut julia_material.1.c.y, -2.0..=2.0));
            });
        }
    });
}

// These handles will store the references to our spawned entities.
#[derive(Resource)]
struct MandelbrotEntity(Option<Entity>);
impl Default for MandelbrotEntity {
    fn default() -> Self {
        MandelbrotEntity(None)
    }
}
#[derive(Resource)]
struct JuliaEntity(Option<Entity>);
impl Default for JuliaEntity {
    fn default() -> Self {
        JuliaEntity(None)
    }
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

#[derive(Resource)]
struct AnimationUpdateToggle {
    active: bool,
}
#[derive(Resource)]
struct MusicUpdateToggle {
    active: bool,
}

impl Default for AnimationUpdateToggle {
    fn default() -> Self {
        AnimationUpdateToggle { active: true }
    }
}
impl Default for MusicUpdateToggle {
    fn default() -> Self {
        MusicUpdateToggle { active: false }
    }
}

// Struct to store uniform parameters for the Mandelbrot fractal.
struct MandelbrotUniforms {
    color_scale: f32,
    max_iterations: f32,
}

// Mandelbrot material definition. It holds parameters and texture for the Mandelbrot fractal.
#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "148ef22b-c53e-4bc2-982c-bb2b102e38f8"]
struct MandelbrotMaterial {
    #[uniform(0)]
    color_scale: f32,
    #[uniform(1)]
    max_iterations: f32,
    #[uniform(2)]
    zoom: f32,
    #[uniform(3)]
    offset: Vec2,
    #[uniform(6)]
    global_offset: Vec2,
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
    #[uniform(1)]
    max_iterations: f32,
    #[uniform(2)]
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

fn mandelbrot_toggle_system(
    keyboard_input: Res<bevy::input::Input<KeyCode>>,
    mut animation_toggle: ResMut<AnimationUpdateToggle>,
    mut music_toggle: ResMut<MusicUpdateToggle>,
) {
    if keyboard_input.just_pressed(KeyCode::A) {
        // You can choose another key if needed
        animation_toggle.active = !animation_toggle.active;
    }
    if keyboard_input.just_pressed(KeyCode::M) {
        // You can choose another key if needed
        music_toggle.active = !music_toggle.active;
    }
}

// System to update the Mandelbrot material's color_scale based on time.
fn mandelbrot_uniform_update_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>, // For Julia material
    toggle: Res<AnimationUpdateToggle>,
    pancam_query: Query<&PanCamState>,
) {
    if !toggle.active {
        return;
    }
    for (_, mut material) in materials.iter_mut() {
        material.color_scale = (0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * 0.1).sin()));
        let pancam = pancam_query.get_single().unwrap();
        material.zoom = pancam.current_zoom;

        let offset = Vec2::new(
            pancam
                .target_translation
                .unwrap_or(Vec3::new(0.0, 0.0, 0.0))
                .x,
            pancam
                .target_translation
                .unwrap_or(Vec3::new(0.0, 0.0, 0.0))
                .y,
        );
        material.offset = offset;
        material.global_offset = offset / pancam.current_zoom;
    }
    for (_, mut material) in julia_materials.iter_mut() {
        // Different frequencies and phase shifts for x and y components
        material.color_scale = (0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * 0.01).sin()))
            .min(0.8)
            .max(0.2);
        material.c.y =
            0.8 * 0.5 * (1.0 - (time.raw_elapsed_seconds_f64() as f32 * 0.15 + 0.5).cos());
        material.c.x =
            0.2 * 0.5 * (1.0 - (time.raw_elapsed_seconds_f64() as f32 * 0.1 - 0.5).cos());

        //println!("X: {}, Y: {}", material.c.x, material.c.y);
    }
}

use bevy::ecs::entity::Entities;
// System to update the material based on the current fractal type
// System to update the material based on the current fractal type
fn fractal_update_system(
    entities: &Entities,
    mut commands: Commands,
    asset_server: Res<AssetServer>, // For loading assets
    mut materials: ResMut<Assets<MandelbrotMaterial>>, // For Mandelbrot material
    mut julia_materials: ResMut<Assets<JuliaMaterial>>, // For Julia material
    mut meshes: ResMut<Assets<Mesh>>, // For meshes
    fractal_type: Res<FractalType>,
    mut mandelbrot_entity: ResMut<MandelbrotEntity>,
    mut julia_entity: ResMut<JuliaEntity>,
) {
    if fractal_type.is_changed() {
        println!("Fractal Type Changed");
        let colormap_texture_handle = asset_server.load("gradient.png");
        // Define uniform values for the Mandelbrot material.
        let uniforms = MandelbrotUniforms {
            color_scale: 0.5,
            max_iterations: 5000.0,
        };
        let mesh = Mesh::from(shape::Quad {
            size: Vec2::new(100000.0, 100000.0),
            flip: false,
        });
        let mandelbrot_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));

        match *fractal_type {
            FractalType::Mandelbrot => {
                if let Some(entity) = julia_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }

                // Spawn Mandelbrot entity
                let mandelbrot_material_handle = prepare_mandelbrot_material(
                    &uniforms,
                    colormap_texture_handle.clone(),
                    &mut materials,
                );
                mandelbrot_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: mandelbrot_mesh.clone(),
                            material: mandelbrot_material_handle,
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
            FractalType::Julia => {
                if let Some(entity) = mandelbrot_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }

                // Spawn Julia entity
                let julia_material_handle = prepare_julia_material(
                    &uniforms,
                    colormap_texture_handle,
                    &mut julia_materials,
                );
                julia_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: mandelbrot_mesh.clone(),
                            material: julia_material_handle,
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
        }
    }
}

// System to toggle between Mandelbrot and Julia fractals
fn fractal_toggle_system(
    keyboard_input: Res<bevy::input::Input<KeyCode>>,
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
        max_iterations: uniforms.max_iterations,
        color_scale: uniforms.color_scale,
        zoom: 4.5,
        offset: Vec2 { x: 0.0, y: 0.0 },
        global_offset: Vec2 { x: 0.0, y: 0.0 } / 4.5,
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
        color_scale: uniforms.color_scale,
        max_iterations: uniforms.max_iterations,
        c: Vec2 { x: 0.3, y: 0.8 },
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
    mut mandelbrot_entity: ResMut<MandelbrotEntity>,
    mut julia_entity: ResMut<JuliaEntity>,
) {
    // Add a camera with custom pan and zoom capabilities.
    commands.spawn((
        Camera2dBundle::default(),
        PanCamConfig {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.00012,
            max_scale: Some(100.0),
            min_x: Some(-50000.0),
            min_y: Some(-50000.0),
            max_x: Some(50000.0),
            max_y: Some(50000.0),
            pixels_per_line: 10.0,
            base_zoom_multiplier: 10.0,
            shift_multiplier_normal: 10.0,
            shift_multiplier_shifted: 100.0,
            animation_scale: 3.0,
            ..default()
        },
        PanCamState {
            current_zoom: 1.0,
            target_zoom: 75.0,
            is_zooming: true,
            target_translation: None,
            delta_zoom_translation: None,
            ..default()
        },
    ));
}
