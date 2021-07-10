use bevy::{
    math::{vec2, vec3, Vec3},
    prelude::{Commands, OrthographicCameraBundle, PerspectiveCameraBundle, Plugin},
};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_plugin(OrbitCameraPlugin)
            .add_plugin(LookTransformPlugin);
    }
}

/// Usage first seen in https://github.com/bonsairobo/feldspar-editor
pub fn spawn_3d_orbit_camera(cmds: &mut Commands) {
    cmds.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController {
            enabled: true,
            mouse_rotate_sensitivity: vec2(0.002, 0.002),
            mouse_translate_sensitivity: vec2(0.4, 0.4),
            mouse_wheel_zoom_sensitivity: 0.1,
            smoothing_weight: 0.9,
        },
        PerspectiveCameraBundle::new_3d(),
        vec3(0.0, 0.0, 500.0), // be reasonably far away for 2D entities
        Vec3::ZERO,            // look towards the forward facing 2D entities
    ));
}

pub fn spawn_2d_ortho_camera(cmds: &mut Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.translation.y = 48.0;
    camera.orthographic_projection.scale = 0.5;

    cmds.spawn_bundle(camera);
}
