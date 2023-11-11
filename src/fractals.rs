use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_asset::AssetServer;

use crate::materials::{
    prepare_burning_ship_material, BurningShipEntity, BurningShipMaterial, BurningShipUniforms,
};
use crate::materials::{prepare_julia_material, JuliaEntity, JuliaMaterial, JuliaUniforms};
use crate::materials::{
    prepare_mandelbrot_material, MandelbrotEntity, MandelbrotMaterial, MandelbrotUniforms,
};

use crate::PanCamState;

#[derive(Resource)]
enum FractalType {
    Mandelbrot,
    Julia,
    BurningShip,
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

impl Default for AnimationUpdateToggle {
    fn default() -> Self {
        AnimationUpdateToggle { active: true }
    }
}

#[derive(Resource)]
pub struct AnimationSpeed(pub f32);

impl Default for AnimationSpeed {
    fn default() -> Self {
        AnimationSpeed(0.001)
    }
}

#[derive(Default)]
pub struct FractalControlPlugin;

impl Plugin for FractalControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FractalType>();
        app.init_resource::<AnimationUpdateToggle>();
        app.init_resource::<AnimationSpeed>();
        app.add_systems(FixedUpdate, uniform_update_system); // Update system for Mandelbrot material.
        app.add_systems(Update, fractal_toggle_system); // Update system for Mandelbrot material.
        app.add_systems(Update, fractal_update_system);
    }
}

fn fractal_toggle_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut fractal_type: ResMut<FractalType>,
    mut animation_toggle: ResMut<AnimationUpdateToggle>,
    //mut music_toggle: ResMut<MusicUpdateToggle>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        *fractal_type = match *fractal_type {
            FractalType::Mandelbrot => FractalType::Julia,
            FractalType::Julia => FractalType::BurningShip,
            FractalType::BurningShip => FractalType::Mandelbrot,
        };
    }
    if keyboard_input.just_pressed(KeyCode::A) {
        animation_toggle.active = !animation_toggle.active;
    }
    //if keyboard_input.just_pressed(KeyCode::M) {
    //    music_toggle.active = !music_toggle.active;
    //}
}

// System to update the Mandelbrot material's color_scale based on time.
fn uniform_update_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>, // For Julia material
    mut burning_ship_materials: ResMut<Assets<BurningShipMaterial>>, // For Julia material
    toggle: Res<AnimationUpdateToggle>,
    animation_speed: ResMut<AnimationSpeed>,
    pancam_query: Query<&PanCamState>,
    fractal_type: Res<FractalType>,
) {
    if !toggle.active {
        return;
    }
    match *fractal_type {
        FractalType::Mandelbrot => {
            for (_, material) in materials.iter_mut() {
                let min_val = 0.05;
                let max_val = 0.95;
                let oscillation = (time.elapsed_seconds_f64() as f32 * animation_speed.0).sin();

                let range = max_val - min_val;
                material.color_scale = min_val + (range / 2.0) * (oscillation + 1.0);

                let pancam = pancam_query.get_single().unwrap();
                material.zoom = pancam.current_zoom;

                let offset = pancam.uv_offset;
                material.offset = offset;
                material.global_offset = offset / pancam.current_zoom;
            }
        }
        FractalType::Julia => {
            for (_, material) in julia_materials.iter_mut() {
                // Different frequencies and phase shifts for x and y components
                let min_val = 0.05;
                let max_val = 0.95;
                let oscillation = (time.elapsed_seconds_f64() as f32 * animation_speed.0).sin();

                let range = max_val - min_val;
                material.color_scale = min_val + (range / 2.0) * (oscillation + 1.0);

                // Restrict the range for c values
                let max_c = 0.8;
                let min_c = -0.8;

                let c_range = max_c - min_c;
                let cx_oscillation =
                    0.5 * (1.0 - (time.elapsed_seconds_f64() as f32 * 0.1 - 0.5).cos());
                let cy_oscillation =
                    0.5 * (1.0 - (time.elapsed_seconds_f64() as f32 * 0.15 + 0.5).cos());

                material.c.x = min_c + c_range * cx_oscillation;
                material.c.y = min_c + c_range * cy_oscillation;
            }
        }
        FractalType::BurningShip => {
            for (_, material) in burning_ship_materials.iter_mut() {
                // Different frequencies and phase shifts for x and y components
                let min_val = 0.00;
                let max_val = 0.70;
                let oscillation = (time.elapsed_seconds_f64() as f32 * animation_speed.0).sin();

                let range = max_val - min_val;
                material.color_scale = min_val + (range / 2.0) * (oscillation + 1.0);
            }
        }
    }
}

use bevy::ecs::entity::Entities;
// System to update the material based on the current fractal type
// System to update the material based on the current fractal type
fn fractal_update_system(
    entities: &Entities,
    mut commands: Commands,
    asset_server: Res<AssetServer>, // For loading assets
    mut mandelbrot_materials: ResMut<Assets<MandelbrotMaterial>>, // For Mandelbrot material
    mut julia_materials: ResMut<Assets<JuliaMaterial>>, // For Julia material
    mut burning_ship_materials: ResMut<Assets<BurningShipMaterial>>, // For Julia material
    mut meshes: ResMut<Assets<Mesh>>, // For meshes
    fractal_type: Res<FractalType>,
    mut mandelbrot_entity: ResMut<MandelbrotEntity>,
    mut julia_entity: ResMut<JuliaEntity>,
    mut burning_ship_entity: ResMut<BurningShipEntity>,
    pancam_query: Query<&PanCamState>,
) {
    if fractal_type.is_changed() {
        println!("Fractal Type Changed");
        let colormap_texture_handle = asset_server.load("gradient.png");
        // Define uniform values for the Mandelbrot material.
        let mandelbrot_uniforms = MandelbrotUniforms {
            color_scale: 0.5,
            max_iterations: 5000.0,
        };
        let julia_uniforms = JuliaUniforms {
            color_scale: 0.5,
            max_iterations: 1000.0,
        };
        let burning_ship_uniforms = BurningShipUniforms {
            color_scale: 0.5,
            max_iterations: 1000.0,
        };
        let mesh = Mesh::from(shape::Quad {
            size: Vec2::new(10000.0, 10000.0),
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
                if let Some(entity) = burning_ship_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }

                // Spawn Mandelbrot entity
                let mandelbrot_material_handle = prepare_mandelbrot_material(
                    &mandelbrot_uniforms,
                    colormap_texture_handle.clone(),
                    &mut mandelbrot_materials,
                    pancam_query.get_single().unwrap().current_zoom,
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
                println!("Spawned Mandelbrot");
            }
            FractalType::Julia => {
                if let Some(entity) = mandelbrot_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }
                if let Some(entity) = burning_ship_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }

                // Spawn Julia entity
                let julia_material_handle = prepare_julia_material(
                    &julia_uniforms,
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
                println!("Spawned Julia");
            }
            FractalType::BurningShip => {
                if let Some(entity) = mandelbrot_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }
                if let Some(entity) = julia_entity.0 {
                    if entities.contains(entity) {
                        commands.entity(entity).despawn();
                    }
                }

                // Spawn Sierpinski Triangle entity
                let burning_ship_material_handle = prepare_burning_ship_material(
                    &burning_ship_uniforms,
                    colormap_texture_handle,
                    &mut burning_ship_materials,
                );
                burning_ship_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: mandelbrot_mesh.clone(),
                            material: burning_ship_material_handle,
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
                println!("Spawned Sierpinski Triangle");
            }
        }
    }
}
