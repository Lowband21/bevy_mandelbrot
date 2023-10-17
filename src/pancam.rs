#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::{
    input::keyboard::KeyCode,
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::vec2,
    prelude::*,
    render::camera::CameraProjection,
    window::PrimaryWindow,
};

/// Plugin that adds the necessary systems for `PanCamConfig` and `PanCamState` components to work
#[derive(Default)]
pub struct PanCamPlugin;

/// System set to allow ordering of `PanCamPlugin`
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct PanCamSystemSet;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_movement.before(apply_constraints_system),
                camera_zoom
                    .before(apply_constraints_system)
                    .before(zoom_interpolation_system),
                zoom_interpolation_system.before(apply_constraints_system),
                apply_constraints_system,
            )
                .in_set(PanCamSystemSet),
        )
        .register_type::<PanCamConfig>()
        .register_type::<PanCamState>();

        {
            app.init_resource::<EguiWantsFocus>()
                .add_systems(PostUpdate, check_egui_wants_focus)
                .configure_set(
                    Update,
                    PanCamSystemSet.run_if(resource_equals(EguiWantsFocus(false))),
                );
        }
    }
}

#[derive(Resource, Deref, DerefMut, PartialEq, Eq, Default)]
struct EguiWantsFocus(bool);

// todo: make run condition when Bevy supports mutable resources in them
fn check_egui_wants_focus(
    mut contexts: Query<&mut bevy_egui::EguiContext>,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    let ctx = contexts.iter_mut().next();
    let new_wants_focus = if let Some(ctx) = ctx {
        let ctx = ctx.into_inner().get_mut();
        ctx.wants_pointer_input() || ctx.wants_keyboard_input()
    } else {
        false
    };
    wants_focus.set_if_neq(EguiWantsFocus(new_wants_focus));
}

// System that applies constraints on the camera's position and zoom based on defined bounds.
fn apply_constraints_system(
    mut query: Query<(
        &PanCamConfig,
        &mut PanCamState,
        &mut OrthographicProjection,
        &mut Transform,
    )>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    for (cam_conf, cam, mut proj, mut pos) in query.iter_mut() {
        let scale_constrained = BVec2::new(
            cam_conf.min_x.is_some() && cam_conf.max_x.is_some(),
            cam_conf.min_y.is_some() && cam_conf.max_y.is_some(),
        );

        let bounds_size = vec2(
            cam_conf.max_x.unwrap_or(f32::INFINITY) - cam_conf.min_x.unwrap_or(-f32::INFINITY),
            cam_conf.max_y.unwrap_or(f32::INFINITY) - cam_conf.min_y.unwrap_or(-f32::INFINITY),
        );

        let max_safe_scale = max_scale_within_bounds(bounds_size, &proj, window_size);

        // Clamp to minimum scale
        proj.scale = proj.scale.max(cam_conf.min_scale);

        // Apply max scale constraint based on both cam.max_scale and boundary constraints
        let max_scale = cam_conf
            .max_scale
            .unwrap_or(f32::INFINITY)
            .min(if scale_constrained.x {
                max_safe_scale.x
            } else {
                f32::INFINITY
            })
            .min(if scale_constrained.y {
                max_safe_scale.y
            } else {
                f32::INFINITY
            });

        proj.scale = proj.scale.min(max_scale);

        // Apply positional constraints (as previously detailed)
        let proj_size = proj.area.size();
        let half_of_viewport = proj_size / 2.;

        if let Some(min_x_bound) = cam_conf.min_x {
            let min_safe_cam_x = min_x_bound + half_of_viewport.x;
            pos.translation.x = pos.translation.x.max(min_safe_cam_x);
        }
        if let Some(max_x_bound) = cam_conf.max_x {
            let max_safe_cam_x = max_x_bound - half_of_viewport.x;
            pos.translation.x = pos.translation.x.min(max_safe_cam_x);
        }
        if let Some(min_y_bound) = cam_conf.min_y {
            let min_safe_cam_y = min_y_bound + half_of_viewport.y;
            pos.translation.y = pos.translation.y.max(min_safe_cam_y);
        }
        if let Some(max_y_bound) = cam_conf.max_y {
            let max_safe_cam_y = max_y_bound - half_of_viewport.y;
            pos.translation.y = pos.translation.y.min(max_safe_cam_y);
        }
    }
}

// Interpolation system for smooth camera zoom transitions.
fn zoom_interpolation_system(
    mut query: Query<(
        &PanCamConfig,
        &mut PanCamState,
        &mut OrthographicProjection,
        &mut Transform,
    )>,
    time: Res<Time>,
) {
    for (cam_conf, mut cam, mut proj, mut transform) in query.iter_mut() {
        let interpolation_factor = cam_conf.animation_scale * time.delta_seconds();
        if cam.is_zooming {
            let zoom_difference = cam.target_zoom - proj.scale;

            // Reset zooming flag if close to target values
            if zoom_difference.abs() <= 0.1
                && (cam.target_translation.is_none()
                    || (cam.target_translation.unwrap() - transform.translation).length() <= 0.01)
            {
                cam.is_zooming = false;
            }
            if zoom_difference.abs() <= 0.1 {
                cam.first_zoom = false;
            }

            // Scaling interpolation
            let zoom_step = zoom_difference * interpolation_factor;
            if zoom_difference.signum() == (cam.target_zoom - (proj.scale + zoom_step)).signum() {
                proj.scale += zoom_step;
            } else {
                proj.scale = cam.target_zoom;
                cam.first_zoom = false;
            }

            // Positional interpolation
            if let Some(target_translation) = cam.target_translation {
                let translation_diff = target_translation - transform.translation;
                let translation_step = translation_diff * interpolation_factor;

                if translation_diff.x.signum()
                    == (target_translation.x - (transform.translation.x + translation_step.x))
                        .signum()
                {
                    transform.translation.x += translation_step.x;
                } else {
                    transform.translation.x = target_translation.x;
                }

                if translation_diff.y.signum()
                    == (target_translation.y - (transform.translation.y + translation_step.y))
                        .signum()
                {
                    transform.translation.y += translation_step.y;
                } else {
                    transform.translation.y = target_translation.y;
                }
            }
        }
    }
}

// Handle camera zooming based on mouse wheel events and target zoom levels.
fn camera_zoom(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &PanCamConfig,
        &mut PanCamState,
        &mut OrthographicProjection,
        &mut Transform,
    )>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| Vec2::new(p.x, -p.y));

    for (cam_conf, mut cam, mut proj, pos) in &mut query {
        let pixels_per_line = cam_conf.pixels_per_line;
        let base_zoom_multiplier = cam_conf.base_zoom_multiplier;

        // Check if left shift is pressed
        let shift_multiplier = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            cam_conf.shift_multiplier_shifted
        } else {
            cam_conf.shift_multiplier_normal
        };

        let mut scroll = scroll_events
            .iter()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Pixel => ev.y,
                MouseScrollUnit::Line => ev.y * pixels_per_line,
            })
            .sum::<f32>();

        //println!("Current scale: {:?}", proj.scale);
        //println!("Target scale: {:?}", cam.target_zoom);
        if scroll == 0.0
            && cam.first_zoom
            && (proj.scale.signum() < cam.target_zoom.signum()
                || proj.scale.signum() > cam.target_zoom.signum())
        {
            if proj.scale.signum() != cam.target_zoom.signum() {
                scroll = 10.0; // adjust this value to your desired initial zoom factor
            } else {
                cam.first_zoom = false;
            }
        }
        if !cam.initialized && !cam.first_zoom {
            proj.scale = cam.current_zoom;
            scroll = 10.0; // adjust this value to your desired initial zoom factor
            cam.first_zoom = true;
            cam.is_zooming = true; // Ensure the camera starts zooming on initialization
            cam.initialized = true;
        }

        if scroll != 0.0 || cam.is_zooming {
            if cam_conf.enabled {
                // Compute dynamic zoom factor based on the current scale

                // Adjust the zoom multiplier with the dynamic factor
                let zoom_multiplier = base_zoom_multiplier * shift_multiplier;

                let old_scale = proj.scale;
                if !cam.first_zoom {
                    cam.target_zoom = old_scale * (1.0 + (-scroll * 0.001 * zoom_multiplier));
                }

                if let Some(mouse_normalized_screen_pos) = mouse_normalized_screen_pos {
                    let proj_size = proj.area.max / old_scale;
                    let mouse_world_pos = pos.translation.truncate()
                        + mouse_normalized_screen_pos * proj_size * old_scale;

                    cam.target_translation = Some(
                        (mouse_world_pos
                            - mouse_normalized_screen_pos * proj_size * cam.target_zoom)
                            .extend(pos.translation.z),
                    );
                }

                if let Some(mouse_normalized_screen_pos) = mouse_normalized_screen_pos {
                    let proj_size = proj.area.max / old_scale;
                    let mouse_world_pos_before = pos.translation.truncate()
                        + mouse_normalized_screen_pos * proj_size * old_scale;
                    let mouse_world_pos_after = pos.translation.truncate()
                        + mouse_normalized_screen_pos * proj_size * cam.target_zoom;
                    cam.delta_zoom_translation =
                        Some((mouse_world_pos_before - mouse_world_pos_after).extend(0.0));
                } else {
                    cam.delta_zoom_translation = Some(Vec3::ZERO);
                }

                // set the zooming flag
                cam.is_zooming = true;
            }
        }
    }
}
/// max_scale_within_bounds is used to find the maximum safe zoom out/projection
/// scale when we have been provided with minimum and maximum x boundaries for
/// the camera.
fn max_scale_within_bounds(
    bounds_size: Vec2,
    proj: &OrthographicProjection,
    window_size: Vec2, //viewport?
) -> Vec2 {
    let mut p = proj.clone();
    p.scale = 1.;
    p.update(window_size.x, window_size.y);
    let base_world_size = p.area.size();
    bounds_size / base_world_size
}

// Handle camera movement based on mouse drag events.
fn camera_movement(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(
        &PanCamConfig,
        &mut PanCamState,
        &mut Transform,
        &OrthographicProjection,
    )>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    // Use position instead of MouseMotion, otherwise we don't get acceleration movement
    let current_pos = match window.cursor_position() {
        Some(c) => Vec2::new(c.x, -c.y),
        None => return,
    };
    let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

    for (cam_conf, cam, mut transform, projection) in &mut query {
        if cam_conf.enabled
            && cam_conf
                .grab_buttons
                .iter()
                .any(|btn| mouse_buttons.pressed(*btn))
        {
            let proj_size = projection.area.size();
            let world_units_per_device_pixel = proj_size / window_size;
            let delta_world = delta_device_pixels * world_units_per_device_pixel;

            if !cam.is_zooming {
                // Handle panning
                transform.translation -= delta_world.extend(0.0);
            }

            // Apply boundary constraints
            let half_of_viewport = proj_size / 2.;

            if let Some(min_x_boundary) = cam_conf.min_x {
                let min_safe_cam_x = min_x_boundary + half_of_viewport.x;
                transform.translation.x = transform.translation.x.max(min_safe_cam_x);
            }
            if let Some(max_x_boundary) = cam_conf.max_x {
                let max_safe_cam_x = max_x_boundary - half_of_viewport.x;
                transform.translation.x = transform.translation.x.min(max_safe_cam_x);
            }
            if let Some(min_y_boundary) = cam_conf.min_y {
                let min_safe_cam_y = min_y_boundary + half_of_viewport.y;
                transform.translation.y = transform.translation.y.max(min_safe_cam_y);
            }
            if let Some(max_y_boundary) = cam_conf.max_y {
                let max_safe_cam_y = max_y_boundary - half_of_viewport.y;
                transform.translation.y = transform.translation.y.min(max_safe_cam_y);
            }
        }
    }
    *last_pos = Some(current_pos);
}

/// A component for user-facing configurations of panning camera controls.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCamConfig {
    pub grab_buttons: Vec<MouseButton>,
    pub enabled: bool,
    pub zoom_to_cursor: bool,
    pub min_scale: f32,
    pub max_scale: Option<f32>,
    pub min_x: Option<f32>,
    pub max_x: Option<f32>,
    pub min_y: Option<f32>,
    pub max_y: Option<f32>,
    pub pixels_per_line: f32,
    pub base_zoom_multiplier: f32,
    pub shift_multiplier_normal: f32,
    pub shift_multiplier_shifted: f32,
    pub animation_scale: f32,
}

impl Default for PanCamConfig {
    fn default() -> Self {
        Self {
            grab_buttons: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.00001,
            max_scale: None,
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
            pixels_per_line: 100.0,
            base_zoom_multiplier: 10.0,
            shift_multiplier_normal: 10.0,
            shift_multiplier_shifted: 30.0,
            animation_scale: 3.0,
        }
    }
}

/// A component for internal state variables of panning camera controls.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCamState {
    pub current_zoom: f32,
    pub target_zoom: f32,
    pub is_zooming: bool,
    pub target_translation: Option<Vec3>,
    pub delta_zoom_translation: Option<Vec3>,
    pub first_zoom: bool,
    pub initialized: bool,
}

impl Default for PanCamState {
    fn default() -> Self {
        Self {
            current_zoom: 1.0,
            target_zoom: 1.0,
            is_zooming: false,
            target_translation: None,
            delta_zoom_translation: None,
            first_zoom: false,
            initialized: false,
        }
    }
}

// Unit tests to verify the behavior of functions within this module.
#[cfg(test)]
mod tests {
    use std::f32::INFINITY;

    use bevy::prelude::OrthographicProjection;

    use super::*;

    /// Simple mock function to construct a square projection from a window size
    fn mock_proj(window_size: Vec2) -> OrthographicProjection {
        let mut proj = Camera2dBundle::default().projection;
        proj.update(window_size.x, window_size.y);
        proj
    }

    #[test]
    fn bounds_matching_window_width_have_max_scale_1() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(100., INFINITY), &proj, window_size).x,
            1.
        );
    }

    // boundaries are 1/2 the size of the projection window
    #[test]
    fn bounds_half_of_window_width_have_half_max_scale() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(50., INFINITY), &proj, window_size).x,
            0.5
        );
    }

    // boundaries are 2x the size of the projection window
    #[test]
    fn bounds_twice_of_window_width_have_max_scale_2() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(200., INFINITY), &proj, window_size).x,
            2.
        );
    }

    #[test]
    fn bounds_matching_window_height_have_max_scale_1() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(INFINITY, 100.), &proj, window_size).y,
            1.
        );
    }

    // boundaries are 1/2 the size of the projection window
    #[test]
    fn bounds_half_of_window_height_have_half_max_scale() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(INFINITY, 50.), &proj, window_size).y,
            0.5
        );
    }

    // boundaries are 2x the size of the projection window
    #[test]
    fn bounds_twice_of_window_height_have_max_scale_2() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(INFINITY, 200.), &proj, window_size).y,
            2.
        );
    }
}
