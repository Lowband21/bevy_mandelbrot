use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_asset::{AssetServer, Handle};

use crate::audio::MusicUpdateToggle;
use crate::julia_material::{prepare_julia_material, JuliaEntity, JuliaMaterial, JuliaUniforms};
use crate::mandelbrot_material::{
    prepare_mandelbrot_material, MandelbrotEntity, MandelbrotMaterial, MandelbrotUniforms,
};
use crate::PanCamState;

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

impl Default for AnimationUpdateToggle {
    fn default() -> Self {
        AnimationUpdateToggle { active: true }
    }
}

#[derive(Default)]
pub struct FractalControlPlugin;

impl Plugin for FractalControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FractalType>();
        app.init_resource::<AnimationUpdateToggle>();
        app.add_systems(Update, uniform_update_system); // Update system for Mandelbrot material.
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

// System to update the Mandelbrot material's color_scale based on time.
fn uniform_update_system(
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
        material.color_scale = 0.5 * (1.0 + (time.raw_elapsed_seconds_f64() as f32 * 0.1).sin());
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
        let mandelbrot_uniforms = MandelbrotUniforms {
            color_scale: 0.5,
            max_iterations: 5000.0,
        };
        let julia_uniforms = JuliaUniforms {
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
                    &mandelbrot_uniforms,
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
                println!("Spawned Mandelbrot");
            }
            FractalType::Julia => {
                if let Some(entity) = mandelbrot_entity.0 {
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
        }
    }
}
