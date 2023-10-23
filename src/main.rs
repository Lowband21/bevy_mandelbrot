// This attribute indicates that if the code is not compiled with debug assertions (i.e., in release mode),
// the application should run with a "windows" subsystem (i.e., without a console window in Windows OS).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::sprite::Material2dPlugin;
use bevy_egui::EguiPlugin;

mod audio;
use crate::audio::AudioVizPlugin;

mod fractals;
use crate::fractals::FractalControlPlugin;

mod pancam;
use crate::pancam::{PanCamConfig, PanCamPlugin, PanCamState};

mod julia_material;
use crate::julia_material::{JuliaEntity, JuliaMaterial};

mod mandelbrot_material;
use crate::mandelbrot_material::{MandelbrotEntity, MandelbrotMaterial};

mod burning_ship_material;
use crate::burning_ship_material::{BurningShipEntity, BurningShipMaterial};

mod prelude;

mod ui;
use crate::ui::UIPlugin;

// The main function to initialize and run the Bevy app.
fn main() {
    // Initializing the Bevy app and adding various plugins.
    let _app = App::new()
        // Uncomment to set a custom clear color for the renderer.
        .insert_resource(ClearColor(Color::hex("071f3c").unwrap()))
        .insert_resource(Msaa::Sample8)
        .init_resource::<MandelbrotEntity>()
        .init_resource::<JuliaEntity>()
        .init_resource::<BurningShipEntity>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(LogDiagnosticsPlugin::default()) // For logging diagnostics.
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // Diagnostics for frame time.
        .add_plugins(PanCamPlugin::default()) // Custom camera control plugin.
        .add_plugins(UIPlugin)
        .add_plugins(AudioVizPlugin)
        .add_plugins(FractalControlPlugin)
        .add_systems(Startup, setup) // Setup function called at startup.
        .add_plugins(Material2dPlugin::<MandelbrotMaterial>::default()) // Plugin for 2D materials.
        .add_plugins(Material2dPlugin::<JuliaMaterial>::default()) // Plugin for 2D materials.
        .add_plugins(Material2dPlugin::<BurningShipMaterial>::default()) // Plugin for 2D materials.
        .run();
}

// The setup function initializes entities in the Bevy app, such as the Mandelbrot mesh and camera.
fn setup(
    mut commands: Commands,
    //_asset_server: Res<AssetServer>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<MandelbrotMaterial>>,
    //mut julia_materials: ResMut<Assets<JuliaMaterial>>,
    //mut mandelbrot_entity: ResMut<MandelbrotEntity>,
    //mut julia_entity: ResMut<JuliaEntity>,
) {
    // Add a camera with custom pan and zoom capabilities.
    commands.spawn((
        Camera2dBundle::default(),
        PanCamConfig {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.00012,
            max_scale: Some(20.0),
            min_x: Some(-5000.0),
            min_y: Some(-5000.0),
            max_x: Some(5000.0),
            max_y: Some(5000.0),
            pixels_per_line: 10.0,
            base_zoom_multiplier: 10.0,
            shift_multiplier_normal: 10.0,
            shift_multiplier_shifted: 100.0,
            animation_scale: 3.0,
            ..default()
        },
        PanCamState {
            current_zoom: 0.5,
            target_zoom: 7.0,
            is_zooming: true,
            target_translation: None,
            delta_zoom_translation: None,
            ..default()
        },
    ));
}
