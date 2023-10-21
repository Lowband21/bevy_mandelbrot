use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_asset::{AssetServer, Handle};

use crate::audio::MusicUpdateToggle;
use crate::chunks::get_required_mandelbrot_chunks;
use crate::chunks::Chunk;
use crate::chunks::ChunkManager;
use crate::julia_material::{prepare_julia_material, JuliaEntity, JuliaMaterial, JuliaUniforms};
use crate::mandelbrot_material::{
    prepare_mandelbrot_material, MandelbrotEntity, MandelbrotMaterial, MandelbrotUniforms,
};
use crate::pancam::PanCam;
use bevy::math::Vec3Swizzles;
use bevy::window::PrimaryWindow;

#[derive(Resource)]
pub enum FractalType {
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

impl Default for AnimationUpdateToggle {
    fn default() -> Self {
        AnimationUpdateToggle { active: false }
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

// System to toggle between Mandelbrot and Julia fractals
fn fractal_toggle_system(
    keyboard_input: Res<bevy::input::Input<KeyCode>>,
    mut fractal_type: ResMut<FractalType>,
    mut animation_toggle: ResMut<AnimationUpdateToggle>,
    mut music_toggle: ResMut<MusicUpdateToggle>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        *fractal_type = match *fractal_type {
            FractalType::Mandelbrot => FractalType::Julia,
            FractalType::Julia => FractalType::Mandelbrot,
        };
    }
    if keyboard_input.just_pressed(KeyCode::A) {
        // You can choose another key if needed
        animation_toggle.active = !animation_toggle.active;
    }
    if keyboard_input.just_pressed(KeyCode::M) {
        // You can choose another key if needed
        music_toggle.active = !music_toggle.active;
    }
}

// Helper function for Mandelbrot material updates
fn update_mandelbrot_material(
    material: &mut MandelbrotMaterial,
    chunk: &Chunk,
    time: &Time,
    animation_speed: f32,
    pancam: &PanCam,
    screen_resolution: Vec2,         // Add this parameter
    mouse_normalized_position: Vec2, // New parameter
) {
    material.color_scale =
        0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * animation_speed).sin());
    //material.offset = Vec2 {
    //    x: pancam.translation.x / screen_resolution.x,
    //    y: pancam.translation.y / screen_resolution.y,
    //};
    material.chunk_offset = chunk.offset; // 2 by 2 coordinate grid
                                          //material.zoom = pancam.zoom;
    material.chunk_zoom = chunk.zoom; // Same system as regular zoom
    material.mouse_normalized_position = mouse_normalized_position;
}

// Helper function for Julia material updates
fn update_julia_material(material: &mut JuliaMaterial, time: &Time) {
    material.color_scale = 0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * 0.2).sin());
    material.c.y = 0.8 * 0.5 * (1.0 - (time.raw_elapsed_seconds_f64() as f32 * 0.15 + 0.5).cos());
    material.c.x = 0.2 * 0.5 * (1.0 - (time.raw_elapsed_seconds_f64() as f32 * 0.1 - 0.5).cos());
}

use bevy::input::mouse::MouseMotion;
fn uniform_update_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>,
    toggle: Res<AnimationUpdateToggle>,
    animation_speed: Res<AnimationSpeed>,
    fractal_type: Res<FractalType>,
    chunk_manager: Res<ChunkManager>,
    pancam_query: Query<&PanCam>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_motion_events: EventReader<MouseMotion>,
) {
    if !toggle.active {
        return;
    }

    let pancam = pancam_query.get_single().unwrap();

    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    let mouse_normalized_position = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| Vec2::new(p.x, -p.y));

    match *fractal_type {
        FractalType::Mandelbrot => {
            for chunk in chunk_manager.mandelbrot_chunks.values() {
                if let Some(handle) = chunk_manager.mandelbrot_id_to_handle.get(&chunk.unique_id) {
                    if let Some(mut material) = materials.get_mut(handle) {
                        update_mandelbrot_material(
                            &mut material,
                            &chunk,
                            &time,
                            animation_speed.0,
                            pancam,
                            window_size,
                            mouse_normalized_position.unwrap_or(Vec2::new(0.0, 0.0)),
                        );
                    }
                }
            }
        }
        FractalType::Julia => {
            for chunk in chunk_manager.julia_chunks.values() {
                if let Some(handle) = chunk_manager.julia_id_to_handle.get(&chunk.unique_id) {
                    if let Some(mut material) = julia_materials.get_mut(handle) {
                        update_julia_material(&mut material, &time);
                    }
                }
            }
        }
    }
}
/*

use bevy::ecs::entity::Entities;

fn fractal_update_system(
    entities: &Entities,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    fractal_type: Res<FractalType>,
    mut mandelbrot_entity: ResMut<MandelbrotEntity>,
    mut julia_entity: ResMut<JuliaEntity>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let colormap_texture_handle = asset_server.load("gradient.png");

    match *fractal_type {
        FractalType::Mandelbrot => {
            let center_position = Vec2::new(0.0, 0.0); // Assuming center of screen is (0,0)
            let chunk_size = Vec2::new(100.0, 100.0);

            // Define two hardcoded chunks
            let chunks = [
                Chunk {
                    offset: center_position - chunk_size / 2.0,
                    ..Default::default()
                },
                Chunk {
                    offset: center_position + chunk_size / 2.0,
                    ..Default::default()
                },
            ];

            for chunk in &chunks {
                let mandelbrot_material_handle = prepare_mandelbrot_material(
                    &MandelbrotUniforms {
                        color_scale: 0.5,
                        max_iterations: 5000.0,
                    },
                    colormap_texture_handle.clone(),
                    &mut materials,
                    &chunk,
                );

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
                            size: chunk_size,
                            flip: false,
                        }))),
                        material: mandelbrot_material_handle.clone(),
                        transform: Transform::from_xyz(chunk.offset.x, chunk.offset.y, 0.0),
                        ..Default::default()
                    },
                    chunk.clone(),
                ));
                println!("Spawned Chunk at {:?}", chunk.offset);
            }
        }
        FractalType::Julia => {
            // Handle Julia type if needed.
            todo!();
        }
    }
}

*/

use bevy::ecs::entity::Entities;
// System to update the material based on the current fractal type
// System to update the material based on the current fractal type
fn fractal_update_system(
    entities: &Entities,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut julia_materials: ResMut<Assets<JuliaMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    fractal_type: Res<FractalType>,
    mut chunk_manager: ResMut<ChunkManager>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut OrthographicProjection, &mut Transform)>,
    mut chunks_query: Query<(Entity, &Chunk)>, // Add this line here
) {
    let colormap_texture_handle = asset_server.load("gradient.png");

    // Get window dimensions.
    let window = primary_window.get_single().unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    // Check the current fractal type to update.
    match *fractal_type {
        FractalType::Mandelbrot => {
            let uniforms = MandelbrotUniforms {
                color_scale: 0.5,
                max_iterations: 5000.0,
            };

            // Update required chunks and collect their UUIDs.
            let required_chunk_ids = {
                // Assuming there is a single camera, you would get its information here.
                let (projection, transform) = query.get_single().unwrap();
                get_required_mandelbrot_chunks(
                    transform.translation.xy(),
                    projection.scale,
                    &mut chunk_manager,
                    &mut materials,
                    colormap_texture_handle.clone(),
                    &uniforms,
                )
            };

            // Destroy unneeded chunks.
            for (entity, chunk) in chunks_query.iter() {
                if !required_chunk_ids.contains(&chunk.unique_id) {
                    commands.entity(entity).despawn_recursive();
                    chunk_manager.mandelbrot_chunks.remove(&chunk.unique_id);
                    chunk_manager
                        .mandelbrot_id_to_handle
                        .remove(&chunk.unique_id);
                }
            }

            // Spawn new chunks or update existing ones.
            for chunk_id in required_chunk_ids {
                let chunk = chunk_manager.mandelbrot_chunks[&chunk_id].clone();
                let mandelbrot_material_handle = prepare_mandelbrot_material(
                    &uniforms,
                    colormap_texture_handle.clone(),
                    &mut materials,
                    &chunk,
                );

                // Update ChunkManager to map chunk IDs to material handles.
                chunk_manager
                    .mandelbrot_id_to_handle
                    .insert(chunk_id, mandelbrot_material_handle.clone());

                // Create entities for each chunk.
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
                            size: Vec2::new(100.0, 100.0), // Adjust based on chunk size
                            flip: false,
                        }))),
                        material: mandelbrot_material_handle,
                        transform: Transform::from_xyz(
                            chunk.offset.x * window_size.x,
                            chunk.offset.y * window_size.y,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    chunk,
                ));
            }
        }
        FractalType::Julia => {
            // TODO: Implement Julia logic.
        }
    }
}
