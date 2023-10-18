use crate::audio::*;
use crate::JuliaMaterial;
use crate::MandelbrotMaterial;
use crate::PanCamState;
use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts, EguiPlugin};

#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, uniform_update_ui_system);
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
