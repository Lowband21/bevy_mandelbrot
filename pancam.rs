#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::vec2,
    math::{Vec2Swizzles, Vec3Swizzles},
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
            (camera_movement, camera_zoom).in_set(PanCamSystemSet),
        )
        .register_type::<PanCam>();

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

fn camera_zoom(
    mut query: Query<(&mut PanCam, &mut OrthographicProjection, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let pixels_per_line = 100.;
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

    let window = primary_window.get_single().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| Vec2::new(p.x, -p.y));

    for (mut cam, mut proj, _) in &mut query {
        if cam.enabled {
            let pre_zoom_mouse_world_position = {
                let proj_size = proj.area.max / cam.zoom;
                cam.translation
                    + mouse_normalized_screen_pos.unwrap_or(Vec2::new(0.5, 0.5)) * proj_size
            };

            cam.zoom = (cam.zoom * (1. + -scroll * 0.001)).max(cam.min_scale);

            let post_zoom_mouse_world_position = {
                let proj_size = proj.area.max / cam.zoom;
                cam.translation + mouse_normalized_screen_pos.unwrap() * proj_size
            };

            cam.translation += pre_zoom_mouse_world_position - post_zoom_mouse_world_position;
        }
    }
}

/*
fn camera_zoom(
    mut query: Query<(&mut PanCam, &mut OrthographicProjection, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let pixels_per_line = 100.; // Maybe make configurable?
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

    for (mut cam, mut proj, mut pos) in &mut query {
        if cam.enabled {
            let old_scale = cam.zoom;
            cam.zoom = (cam.zoom * (1. + -scroll * 0.001)).max(cam.min_scale);

            //// Apply max scale constraint
            //if let Some(max_scale) = cam.max_scale {
            //    cam.zoom = cam.zoom.min(max_scale);
            //}

            //// If there is both a min and max boundary, that limits how far we can zoom. Make sure we don't exceed that
            //let scale_constrained = BVec2::new(
            //    cam.min_x.is_some() && cam.max_x.is_some(),
            //    cam.min_y.is_some() && cam.max_y.is_some(),
            //);

            //if scale_constrained.x || scale_constrained.y {
            //    let bounds_width = if let (Some(min_x), Some(max_x)) = (cam.min_x, cam.max_x) {
            //        max_x - min_x
            //    } else {
            //        f32::INFINITY
            //    };

            //    let bounds_height = if let (Some(min_y), Some(max_y)) = (cam.min_y, cam.max_y) {
            //        max_y - min_y
            //    } else {
            //        f32::INFINITY
            //    };

            //    let bounds_size = vec2(bounds_width, bounds_height);
            //    let max_safe_scale = max_scale_within_bounds(bounds_size, &proj, window_size);

            //    if scale_constrained.x {
            //        cam.zoom = cam.zoom.min(max_safe_scale.x);
            //    }

            //    if scale_constrained.y {
            //        cam.zoom = cam.zoom.min(max_safe_scale.y);
            //    }
            //}

            // Move the camera position to normalize the projection window
            //if let (Some(mouse_normalized_screen_pos), true) =
            //    (mouse_normalized_screen_pos, cam.zoom_to_cursor)
            //{
            //    let proj_size = proj.area.max / old_scale;
            //    let mouse_world_pos =
            //        cam.translation + mouse_normalized_screen_pos * proj_size * old_scale;
            //    cam.translation =
            //        mouse_world_pos - mouse_normalized_screen_pos * proj_size * cam.zoom;

            //    // As we zoom out, we don't want the viewport to move beyond the provided boundary. If the most recent
            //    // change to the camera zoom would move cause parts of the window beyond the boundary to be shown, we
            //    // need to change the camera position to keep the viewport within bounds. The four if statements below
            //    // provide this behavior for the min and max x and y boundaries.
            //    let proj_size = proj.area.size();

            //    let half_of_viewport = proj_size / 2.;

            //    if let Some(min_x_bound) = cam.min_x {
            //        let min_safe_cam_x = min_x_bound + half_of_viewport.x;
            //        cam.translation.x = cam.translation.x.max(min_safe_cam_x);
            //    }
            //    if let Some(max_x_bound) = cam.max_x {
            //        let max_safe_cam_x = max_x_bound - half_of_viewport.x;
            //        cam.translation.x = cam.translation.x.min(max_safe_cam_x);
            //    }
            //    if let Some(min_y_bound) = cam.min_y {
            //        let min_safe_cam_y = min_y_bound + half_of_viewport.y;
            //        cam.translation.y = cam.translation.y.max(min_safe_cam_y);
            //    }
            //    if let Some(max_y_bound) = cam.max_y {
            //        let max_safe_cam_y = max_y_bound - half_of_viewport.y;
            //        cam.translation.y = cam.translation.y.min(max_safe_cam_y);
            //    }
            //}
        }
    }
}
*/

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
    mut query: Query<(&mut PanCam, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = primary_window.get_single().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let current_pos = match window.cursor_position() {
        Some(c) => Vec2::new(c.x, c.y),
        None => return,
    };
    let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

    for (mut cam, _, projection) in &mut query {
        if cam.enabled
            && cam
                .grab_buttons
                .iter()
                .any(|btn| mouse_buttons.pressed(*btn))
        {
            let proj_size = projection.area.size();
            let world_units_per_device_pixel = proj_size / window_size;
            let zoom_factor = cam.zoom;
            let scaling_factor = 0.001 * zoom_factor;
            let delta_world = delta_device_pixels * world_units_per_device_pixel * scaling_factor;
            let mut proposed_cam_transform = cam.translation - delta_world;
            proposed_cam_transform =
                proposed_cam_transform.clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            cam.translation = proposed_cam_transform;
        }
    }
    *last_pos = Some(current_pos);
}
/*
fn camera_movement(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanCam, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = primary_window
        .get_single()
        .expect("No primary window found");
    let window_size = Vec2::new(window.width(), window.height());

    let current_pos = match window.cursor_position() {
        Some(c) => Vec2::new(c.x, c.y), // Removed the negation on y
        None => return,
    };

    let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

    for (mut cam, _, projection) in &mut query {
        if cam.enabled
            && cam
                .grab_buttons
                .iter()
                .any(|btn| mouse_buttons.pressed(*btn))
        {
            let proj_size = projection.area.size();
            let world_units_per_device_pixel = proj_size / window_size;

            // Assume you have a zoom factor; adjust this value as needed.
            let zoom_factor = cam.zoom;

            // Add a scaling factor to slow down movement; adjust this value as needed
            let scaling_factor = 0.001 * zoom_factor;

            let delta_world = delta_device_pixels * world_units_per_device_pixel * scaling_factor;
            let mut proposed_cam_transform = cam.translation - delta_world;

            // Apply boundaries
            proposed_cam_transform =
                proposed_cam_transform.clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));

            cam.translation = proposed_cam_transform;
        }
    }

    *last_pos = Some(current_pos);
}
*/

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
    pub zoom: f32,
    pub translation: Vec2,
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
            zoom: 1.0,
            translation: Vec2::new(0.5, 0.5),
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
