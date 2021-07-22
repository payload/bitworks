use bevy_inspector_egui::InspectableRegistry;
use bevy_vox::VoxPlugin;

use bitworks::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        //.add_plugin(CameraPlugin)
        .add_plugin(VoxPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LookTransformPlugin)
        .insert_resource(InspectableRegistry::default());
    app.run();
}

struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_stuff.system());
    }
}

fn setup_stuff(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn().with_children(|parent| {
        parent.spawn_scene(asset_server.load("2x2x2.vox"));
    });

    commands
        // light
        .spawn_bundle(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
            ..Default::default()
        });
    commands.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController {
            enabled: true,
            mouse_rotate_sensitivity: vec2(0.002, 0.002),
            mouse_translate_sensitivity: vec2(0.4, 0.4),
            mouse_wheel_zoom_sensitivity: 0.1,
            smoothing_weight: 0.9,
        },
        PerspectiveCameraBundle::new_3d(),
        vec3(0.0, 0.0, 50.0), // be reasonably far away for 2D entities
        Vec3::ZERO,            // look towards the forward facing 2D entities
    ));
}
