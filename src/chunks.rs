use crate::fractals::FractalType;
use crate::mandelbrot_material::*;
use crate::JuliaMaterial;
use crate::MandelbrotMaterial;
use crate::PanCam;
use bevy::math::Vec2;
use bevy::math::Vec2Swizzles;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::reflect::TypeUuid;
use std::collections::HashMap;

// Add the uuid crate at the top
use uuid::Uuid;

// Updated Chunk definition
#[derive(Debug, Clone, Component, PartialEq)]
pub struct Chunk {
    pub unique_id: Uuid, // New field for unique ID
    pub offset: Vec2,
    pub zoom: f32,
}

// Update constructors
impl Chunk {
    pub fn new(offset: Vec2, zoom: f32) -> Self {
        Self {
            unique_id: Uuid::new_v4(), // Generate a new UUID
            offset,
            zoom,
        }
    }
}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            unique_id: Uuid::new_v4(),
            offset: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

#[derive(Resource)]
pub struct ChunkManager {
    pub mandelbrot_chunks: HashMap<Uuid, Chunk>, // Using Uuid as key
    pub julia_chunks: HashMap<Uuid, Chunk>,      // Using Uuid as key
    pub active_mandelbrot_chunk_ids: HashSet<Uuid>,
    pub active_julia_chunk_ids: HashSet<Uuid>,
    pub mandelbrot_id_to_handle: HashMap<Uuid, Handle<MandelbrotMaterial>>,
    pub julia_id_to_handle: HashMap<Uuid, Handle<JuliaMaterial>>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            mandelbrot_chunks: HashMap::new(),
            julia_chunks: HashMap::new(),
            active_julia_chunk_ids: HashSet::new(),
            active_mandelbrot_chunk_ids: HashSet::new(),
            mandelbrot_id_to_handle: HashMap::new(),
            julia_id_to_handle: HashMap::new(),
        }
    }

    pub fn create_mandelbrot_chunk(
        &mut self,
        handle: Handle<MandelbrotMaterial>,
        offset: Vec2,
        zoom: f32,
    ) {
        let chunk = Chunk::new(offset, zoom);
        self.mandelbrot_chunks
            .insert(chunk.unique_id, chunk.clone());
        self.mandelbrot_id_to_handle.insert(chunk.unique_id, handle);
    }

    pub fn create_julia_chunk(&mut self, offset: Vec2, zoom: f32) {
        let chunk = Chunk::new(offset, zoom);
        self.julia_chunks.insert(chunk.unique_id, chunk);
    }
}

#[derive(Default)]
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkManager::new());
        //.add_systems(Update, update_chunks);
    }
}

// Assuming you have these imports
use bevy::prelude::*;
use std::collections::HashSet;

use bevy::prelude::Handle;

pub fn get_required_mandelbrot_chunks(
    camera_position: Vec2,
    zoom: f32,
    chunk_manager: &mut ResMut<ChunkManager>,
    materials: &mut ResMut<Assets<MandelbrotMaterial>>,
    colormap_texture_handle: Handle<Image>,
    uniforms: &MandelbrotUniforms,
) -> Vec<Uuid> {
    let mut required_chunk_ids = Vec::new();
    chunk_manager.active_mandelbrot_chunk_ids.clear();

    // Assume a chunk size of 100x100
    let chunk_size: f32 = 100.0;

    // Calculate the zoom adjusted camera view area
    let view_area_width = chunk_size / zoom;
    let view_area_height = chunk_size / zoom;

    let x_min = camera_position.x - view_area_width / 2.0;
    let x_max = camera_position.x + view_area_width / 2.0;
    let y_min = camera_position.y - view_area_height / 2.0;
    let y_max = camera_position.y + view_area_height / 2.0;

    for x in (x_min as i32..x_max as i32).step_by(chunk_size as usize) {
        for y in (y_min as i32..y_max as i32).step_by(chunk_size as usize) {
            let id = get_new_or_existing_chunk(x, y, chunk_size, chunk_manager); // New function
            required_chunk_ids.push(id);
            chunk_manager.active_mandelbrot_chunk_ids.insert(id);
        }
    }
    required_chunk_ids
}

fn update_chunks(
    mut commands: Commands,
    entities: Query<(Entity, &Chunk), Without<JuliaMaterial>>,
    mut chunk_manager: ResMut<ChunkManager>,
    pancam_query: Query<&PanCam>,
    fractal_type: Res<FractalType>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut OrthographicProjection, &mut Transform)>,
) {
    let colormap_texture_handle = asset_server.load("gradient.png");

    let uniforms = &MandelbrotUniforms {
        color_scale: 0.5,
        max_iterations: 5000.0,
    };

    if let Ok(pancam) = pancam_query.get_single() {
        match *fractal_type {
            FractalType::Mandelbrot => {
                for (projection, transform) in &mut query {
                    let required_mandelbrot_chunk_ids = get_required_mandelbrot_chunks(
                        transform.translation.xy(),
                        projection.scale,
                        &mut chunk_manager,
                        &mut materials,
                        colormap_texture_handle.clone(),
                        &uniforms,
                    );
                    // Destroy unneeded chunks
                    for (entity, chunk) in entities.iter() {
                        if !required_mandelbrot_chunk_ids.contains(&chunk.unique_id) {
                            commands.entity(entity).despawn_recursive();
                            chunk_manager.mandelbrot_chunks.remove(&chunk.unique_id);
                            chunk_manager
                                .mandelbrot_id_to_handle
                                .remove(&chunk.unique_id);
                        }
                    }
                }
            }
            FractalType::Julia => {
                // Similar logic for Julia set
                // ...
            }
        }
    }
}

fn get_new_or_existing_chunk(
    x: i32,
    y: i32,
    chunk_size: f32,
    chunk_manager: &mut ResMut<ChunkManager>,
) -> Uuid {
    let scaled_x = (2.0 * x as f32) / chunk_size - 1.0;
    let scaled_y = (2.0 * y as f32) / chunk_size - 1.0;

    let offset = Vec2 {
        x: scaled_x,
        y: scaled_y,
    };

    // Check if this chunk already exists
    if let Some((id, _)) = chunk_manager
        .mandelbrot_chunks
        .iter()
        .find(|(_, chunk)| chunk.offset == offset)
    {
        return *id;
    }

    // If not, create a new one
    let chunk = Chunk::new(offset, 1.0);
    let id = chunk.unique_id;

    chunk_manager.mandelbrot_chunks.insert(id, chunk);

    id
}
