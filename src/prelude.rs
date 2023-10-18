// Importing required modules and traits from the `bevy` crate.
use bevy::prelude::*;

// Additional imports from the `bevy` crate, useful for rendering and diagnostics.
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::reflect::TypePath;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_egui::{egui, EguiContexts, EguiPlugin};
