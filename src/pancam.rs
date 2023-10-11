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

/// Plugin that adds the necessary systems for `PanCam` components to work
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
                camera_zoom.before(apply_constraints_system),
                zoom_interpolation_system.before(apply_constraints_system),
                apply_constraints_system,
            )
                .in_set(PanCamSystemSet),
        )
        .register_type::<PanCam>();

        #[cfg(feature = "bevy_egui")]
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
#[cfg(feature = "bevy_egui")]
struct EguiWantsFocus(bool);

// todo: make run condition when Bevy supports mutable resources in them
#[cfg(feature = "bevy_egui")]
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

fn apply_constraints_system(
    mut query: Query<(&PanCam, &mut OrthographicProjection, &mut Transform)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    for (cam, mut proj, mut pos) in query.iter_mut() {
        let scale_constrained = BVec2::new(
            cam.min_x.is_some() && cam.max_x.is_some(),
            cam.min_y.is_some() && cam.max_y.is_some(),
        );

        let bounds_size = vec2(
            cam.max_x.unwrap_or(f32::INFINITY) - cam.min_x.unwrap_or(-f32::INFINITY),
            cam.max_y.unwrap_or(f32::INFINITY) - cam.min_y.unwrap_or(-f32::INFINITY),
        );

        let max_safe_scale = max_scale_within_bounds(bounds_size, &proj, window_size);

        // Clamp to minimum scale
        proj.scale = proj.scale.max(cam.min_scale);

        // Apply max scale constraint based on both cam.max_scale and boundary constraints
        let max_scale = cam
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

        if let Some(min_x_bound) = cam.min_x {
            let min_safe_cam_x = min_x_bound + half_of_viewport.x;
            pos.translation.x = pos.translation.x.max(min_safe_cam_x);
        }
        if let Some(max_x_bound) = cam.max_x {
            let max_safe_cam_x = max_x_bound - half_of_viewport.x;
            pos.translation.x = pos.translation.x.min(max_safe_cam_x);
        }
        if let Some(min_y_bound) = cam.min_y {
            let min_safe_cam_y = min_y_bound + half_of_viewport.y;
            pos.translation.y = pos.translation.y.max(min_safe_cam_y);
        }
        if let Some(max_y_bound) = cam.max_y {
            let max_safe_cam_y = max_y_bound - half_of_viewport.y;
            pos.translation.y = pos.translation.y.min(max_safe_cam_y);
        }
    }
}

fn zoom_interpolation_system(
    mut query: Query<(&mut PanCam, &mut OrthographicProjection, &mut Transform)>,
    time: Res<Time>,
) {
    let interpolation_factor = 5.0 * time.delta_seconds();

    for (mut cam, mut proj, mut transform) in query.iter_mut() {
        if cam.is_zooming {
            if (cam.target_zoom - proj.scale).abs() > 0.01 {
                if let Some(target_translation) = cam.target_translation {
                    // Interpolate zoom
                    proj.scale += (cam.target_zoom - proj.scale) * interpolation_factor;

                    // Interpolate position
                    transform.translation.x +=
                        (target_translation.x - transform.translation.x) * interpolation_factor;
                    transform.translation.y +=
                        (target_translation.y - transform.translation.y) * interpolation_factor;
                }
            } else {
                proj.scale = cam.target_zoom;
                if let Some(target) = cam.target_translation {
                    transform.translation = target;
                }
                cam.is_zooming = false;
            }
        }
    }
}

fn camera_zoom(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut PanCam, &mut OrthographicProjection, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let pixels_per_line = 100.;
    let base_zoom_multiplier = 10.0;

    // Check if left shift is pressed
    let shift_multiplier = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        1.0
    } else {
        3.0
    };

    let scroll = scroll_events
        .iter()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Pixel => ev.y,
            MouseScrollUnit::Line => ev.y * pixels_per_line,
        })
        .sum::<f32>();

    if scroll == 0. {
        return;
    }

    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| Vec2::new(p.x, -p.y));

    for (mut cam, proj, pos) in &mut query {
        if cam.enabled {
            // Compute dynamic zoom factor based on the current scale
            let dynamic_zoom_factor = proj.scale.min(0.5);

            // Adjust the zoom multiplier with the dynamic factor
            let zoom_multiplier = base_zoom_multiplier * dynamic_zoom_factor * shift_multiplier;

            let old_scale = proj.scale;
            cam.target_zoom =
                (old_scale * (1.0 + -scroll * 0.001 * zoom_multiplier)).max(cam.min_scale);

            if let Some(mouse_normalized_screen_pos) = mouse_normalized_screen_pos {
                let proj_size = proj.area.max / old_scale;
                let mouse_world_pos = pos.translation.truncate()
                    + mouse_normalized_screen_pos * proj_size * old_scale;

                cam.target_translation = Some(
                    (mouse_world_pos - mouse_normalized_screen_pos * proj_size * cam.target_zoom)
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

fn camera_movement(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(&PanCam, &mut Transform, &OrthographicProjection)>,
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

    for (cam, mut transform, projection) in &mut query {
        if cam.enabled
            && cam
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

            if let Some(min_x_boundary) = cam.min_x {
                let min_safe_cam_x = min_x_boundary + half_of_viewport.x;
                transform.translation.x = transform.translation.x.max(min_safe_cam_x);
            }
            if let Some(max_x_boundary) = cam.max_x {
                let max_safe_cam_x = max_x_boundary - half_of_viewport.x;
                transform.translation.x = transform.translation.x.min(max_safe_cam_x);
            }
            if let Some(min_y_boundary) = cam.min_y {
                let min_safe_cam_y = min_y_boundary + half_of_viewport.y;
                transform.translation.y = transform.translation.y.max(min_safe_cam_y);
            }
            if let Some(max_y_boundary) = cam.max_y {
                let max_safe_cam_y = max_y_boundary - half_of_viewport.y;
                transform.translation.y = transform.translation.y.min(max_safe_cam_y);
            }
        }
    }
    *last_pos = Some(current_pos);
}

/// A component that adds panning camera controls to an orthographic camera
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCam {
    /// The mouse buttons that will be used to drag and pan the camera
    pub grab_buttons: Vec<MouseButton>,
    /// Whether camera currently responds to user input
    pub enabled: bool,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// The minimum scale for the camera
    ///
    /// The orthographic projection's scale will be clamped at this value when zooming in
    pub min_scale: f32,
    /// The maximum scale for the camera
    ///
    /// If present, the orthographic projection's scale will be clamped at
    /// this value when zooming out.
    pub max_scale: Option<f32>,
    /// The minimum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_x: Option<f32>,
    /// The maximum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_x: Option<f32>,
    /// The minimum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_y: Option<f32>,
    /// The maximum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_y: Option<f32>,
    pub current_zoom: f32,
    pub target_zoom: f32,
    pub is_zooming: bool,
    pub target_translation: Option<Vec3>,
    pub delta_zoom_translation: Option<Vec3>,
}

impl Default for PanCam {
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
            current_zoom: 1.0,
            target_zoom: 1.0,
            is_zooming: false,
            target_translation: None,
            delta_zoom_translation: None,
        }
    }
}

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
