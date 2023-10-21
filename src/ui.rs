use crate::audio::*;
use crate::fractals::AnimationSpeed;
use crate::pancam::PanCam;
use crate::JuliaMaterial;
use crate::MandelbrotMaterial;
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
    mut pancam_query: Query<&mut PanCam>,
    mut animation_speed: ResMut<AnimationSpeed>,
    mut query: Query<(&mut OrthographicProjection, &mut Transform)>,
) {
    let context = ctx.ctx_mut();
    egui::Window::new("Update Uniforms").show(context, |ui| {
        if let Some(mandelbrot_material) = materials.iter_mut().next() {
            ui.horizontal(|ui| {
                ui.label("Animation Speed:");
                ui.add(egui::Slider::new(&mut animation_speed.0, 0.0..=0.1));
            });
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
                    0.0..=10000.0,
                ));
            });

            for (mut projection, mut transform) in &mut query {
                ui.horizontal(|ui| {
                    ui.label("Mandelbrot Zoom:");
                    ui.add(egui::Slider::new(&mut projection.scale, 0.0..=100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Mandelbrot X Position:");
                    ui.add(egui::Slider::new(
                        &mut transform.translation.x,
                        -1000.0..=1000.0,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Mandelbrot Y Position:");
                    ui.add(egui::Slider::new(
                        &mut transform.translation.y,
                        -1000.0..=1000.0,
                    ));
                });
            }
        }
        if let Some(julia_material) = julia_materials.iter_mut().next() {
            ui.horizontal(|ui| {
                ui.label("Animation Speed:");
                ui.add(egui::Slider::new(&mut animation_speed.0, 0.0..=0.1));
            });
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
                    0.0..=10000.0,
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
            for (mut proj, _pos) in &mut query {
                ui.horizontal(|ui| {
                    ui.label("Julia Zoom:");
                    ui.add(egui::Slider::new(&mut proj.scale, 0.0..=8.0));
                });
            }
        }
    });
}
