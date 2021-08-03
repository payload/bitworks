use std::f32::consts::FRAC_PI_4;

use bitworks::*;

use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_picking::*;
use bevy_mod_raycast::*;
use smooth_bevy_cameras::controllers::orbit::{OrbitCameraBundle, OrbitCameraController};

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        //.add_plugin(PickingPlugin)
        //.add_plugin(InteractablePickingPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(VoxelPlugin)
        .add_plugin(EguiPlugin)
        //.add_plugin(DebugEventsPickingPlugin)
        //.add_plugin(DebugCursorPickingPlugin)
        //.add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .add_plugin(RaycastPlugin)
        .add_plugin(Setup);
    app.run();
}

struct Setup;

impl Plugin for Setup {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(tool_ui.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_raycast_with_cursor.system())
            .add_system(update_plane_selector_with_raycast_source.system())
            .add_startup_system(spawn_camera.system())
            .add_startup_system(spawn_plane_selector.system())
            .add_startup_system(spawn_plane.system());
    }
}

//

struct RaycastPlugin;

impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PluginState<MyRaycastSet>>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                build_rays::<MyRaycastSet>
                    .system()
                    .label(RaycastSystem::BuildRays),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_raycast::<MyRaycastSet>
                    .system()
                    .label(RaycastSystem::UpdateRaycast)
                    .after(RaycastSystem::BuildRays),
            );
    }
}

//

struct MyRaycastSet;

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RayCastSource<MyRaycastSet>>,
) {
    for mut pick_source in &mut query.iter_mut() {
        // Grab the most recent cursor event if it exists:
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
        }
    }
}

fn update_plane_selector_with_raycast_source(
    source_query: Query<&RayCastSource<MyRaycastSet>>,
    mut selector_query: Query<&mut Transform, With<PlaneSelector>>,
) {
    if let Ok(source) = source_query.single() {
        if let Some((_, intersection)) = source.intersect_top() {
            let pos = intersection.position().round();

            for mut selector in selector_query.iter_mut() {
                let new_pos = vec3(pos.x, selector.translation.y, pos.z);

                if selector.translation != new_pos {
                    selector.translation = new_pos;
                }
            }
        }
    }
}

//

fn spawn_plane(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
            transform: Transform::from_translation(vec3(0.0, -0.1, 0.0)),
            material: materials.add(StandardMaterial::unlit_color(Color::DARK_GRAY)),
            ..Default::default()
        })
        .insert(RayCastMesh::<MyRaycastSet>::default())
        .insert_bundle(PickableBundle::default());
}

struct PlaneSelector;
fn spawn_plane_selector(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(
                shape::Torus {
                    radius: 0.6,
                    ring_radius: 0.05,
                    subdivisions_segments: 4,
                    subdivisions_sides: 2,
                }
                .into(),
            ),
            transform: Transform {
                translation: vec3(0.0, 0.0, 0.0),
                rotation: Quat::from_rotation_y(FRAC_PI_4),
                scale: vec3(1.0, 1.0, 1.0),
            },
            material: materials.add(StandardMaterial::unlit_color(Color::GRAY)),
            ..Default::default()
        })
        .insert(PlaneSelector);
}

fn spawn_camera(mut cmds: Commands) {
    cmds.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController {
            enabled: true,
            mouse_rotate_sensitivity: vec2(0.002, 0.002),
            mouse_translate_sensitivity: vec2(0.4, 0.4),
            mouse_wheel_zoom_sensitivity: 0.1,
            smoothing_weight: 0.9,
        },
        PerspectiveCameraBundle::new_3d(),
        vec3(0.0, 10.0, 0.0),
        Vec3::ZERO,
    ))
    .insert(RayCastSource::<MyRaycastSet>::new_transform_empty())
    .insert_bundle(PickingCameraBundle::default());
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Tool {
    Clear,
    Spring,
    Glassblower,
    Tap,
    Trash,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Clear
    }
}

fn tool_ui(mut local_tool: Local<Tool>, egui_ctx: Res<EguiContext>) {
    let tool = &mut *local_tool;

    egui::Window::new("Tool")
        .scroll(true)
        .default_width(100.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.selectable_value(tool, Tool::Clear, "❌ Clear");
            ui.selectable_value(tool, Tool::Spring, "💧 Spring");
            ui.selectable_value(tool, Tool::Glassblower, "🥃 Glassblower");
            ui.selectable_value(tool, Tool::Tap, "🚰 Tap");
            ui.selectable_value(tool, Tool::Trash, "🗑 Trash");
        });
}
