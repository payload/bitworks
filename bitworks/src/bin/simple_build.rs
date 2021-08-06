use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{
    ecs::system::SystemParam,
    input::{mouse::MouseButtonInput, ElementState},
};
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
        app //
            .insert_resource(Tool::Spring)
            .add_system(tool_ui.system())
            .add_system(build_on_click.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_raycast_with_cursor.system())
            .add_system(update_plane_selector_with_raycast_source.system())
            .add_system(update_build_ghost.system())
            .add_system(animation_system.system())
            .add_startup_system(spawn_camera.system())
            .add_startup_system(spawn_plane_selector.system())
            .add_startup_system(spawn_plane.system())
            .add_startup_system(setup_assets.system());
    }
}

//

fn build_on_click(
    mut events: EventReader<MouseButtonInput>,
    plane_selector_query: Query<&Transform, With<PlaneSelector>>,
    tool: Res<Tool>,
    mut cmds: Commands,
    mut build_params: BuildParams,
) {
    let cmds = &mut cmds;
    let p = &mut build_params;

    for event in events.iter() {
        if let (MouseButton::Left, ElementState::Pressed) = (event.button, event.state) {
            if let Ok(transform) = plane_selector_query.single() {
                let tool = *tool;
                let transform = Transform::from_translation(transform.translation);

                match tool {
                    Tool::Clear => {}
                    Tool::Spring => build_spring_at(cmds, p, transform),
                    Tool::Glassblower => build_glassblower_at(cmds, p, transform),
                    Tool::Tap => build_tap_at(cmds, p, transform),
                    Tool::Trash => build_trash_at(cmds, p, transform),
                }
            }
        }
    }
}

fn build_spring_at(cmds: &mut Commands, p: &mut BuildParams, transform: Transform) {
    cmds.spawn_bundle((transform, GlobalTransform::identity()))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: p.meshes.get_handle("building-spring"),
                    material: p.materials.get_handle("black"),
                    ..Default::default()
                })
                .insert(BuildAnimation { value: 0.0 });
        });
}

fn build_glassblower_at(cmds: &mut Commands, p: &mut BuildParams, transform: Transform) {
    cmds.spawn_bundle(PbrBundle {
        transform,
        mesh: p.meshes.get_handle("building-glassblower"),
        material: p.materials.get_handle("black"),
        ..Default::default()
    });
}

fn build_tap_at(cmds: &mut Commands, p: &mut BuildParams, transform: Transform) {
    cmds.spawn_bundle(PbrBundle {
        transform,
        mesh: p.meshes.get_handle("building-tap"),
        material: p.materials.get_handle("black"),
        ..Default::default()
    });
}

fn build_trash_at(cmds: &mut Commands, p: &mut BuildParams, transform: Transform) {
    cmds.spawn_bundle(PbrBundle {
        transform,
        mesh: p.meshes.get_handle("building-trash"),
        material: p.materials.get_handle("black"),
        ..Default::default()
    });
}

//

struct BuildAnimation {
    value: f32,
}

fn animation_system(
    mut cmds: Commands,
    time: Res<Time>,
    mut anim_query: Query<(Entity, &mut Transform, &mut BuildAnimation)>,
) {
    let dt = time.delta_seconds();

    for (entity, mut transform, mut animation) in anim_query.iter_mut() {
        let x = animation.value.min(1.0);

        if x >= 1.0 {
            cmds.entity(entity).remove::<BuildAnimation>();
        }

        // v is from 0 to 1
        // hull is from 1 to 0
        // bounce is is 3 times from 1 to 0
        let hull = -(x * x) + 1.0;
        let bounce = 0.5 * (5.0 * x * PI).cos() + 1.0;

        transform.translation.y = hull * bounce;

        animation.value += dt;
    }
}

//

fn setup_assets(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let _ = meshes.set("building-spring", shape::Cube { size: 0.6 }.into());
    let _ = meshes.set("building-glassblower", shape::Cube { size: 0.8 }.into());
    let _ = meshes.set(
        "building-tap",
        shape::Icosphere {
            radius: 0.3,
            subdivisions: 1,
        }
        .into(),
    );
    let _ = meshes.set(
        "building-trash",
        shape::Icosphere {
            radius: 0.4,
            subdivisions: 3,
        }
        .into(),
    );

    let _ = materials.set("black", StandardMaterial::unlit_color(Color::BLACK));
}

//

#[derive(SystemParam)]
pub struct BuildParams<'a> {
    meshes: Res<'a, Assets<Mesh>>,
    materials: Res<'a, Assets<StandardMaterial>>,
}

struct BuildGhost;
fn update_build_ghost(
    res_tool: Res<Tool>,
    meshes: Res<Assets<Mesh>>,
    mut ghost_query: Query<(&mut Tool, &mut Handle<Mesh>, &mut Visible), With<BuildGhost>>,
) {
    if let Ok((mut ghost_tool, mut handle_mesh, mut visible)) = ghost_query.single_mut() {
        let tool = *res_tool;
        if *ghost_tool != tool {
            *ghost_tool = tool;

            if tool == Tool::Clear {
                visible.is_visible = false;
            } else {
                visible.is_visible = true;

                let handle = handle_mesh.clone();
                *handle_mesh = match tool {
                    Tool::Clear => handle,
                    Tool::Spring => meshes.get_handle("building-spring"),
                    Tool::Glassblower => meshes.get_handle("building-glassblower"),
                    Tool::Tap => meshes.get_handle("building-tap"),
                    Tool::Trash => meshes.get_handle("building-trash"),
                };
            }
        }
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
    cmds.spawn_bundle((
        PlaneSelector,
        GlobalTransform::identity(),
        Transform::identity(),
    ))
    .with_children(|parent| {
        parent.spawn().insert_bundle(PbrBundle {
            mesh: meshes.add(
                shape::Torus {
                    radius: 0.6,
                    ring_radius: 0.05,
                    subdivisions_segments: 4,
                    subdivisions_sides: 2,
                }
                .into(),
            ),
            transform: Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_4)),
            material: materials.add(StandardMaterial::unlit_color(Color::GRAY)),
            ..Default::default()
        });

        parent
            .spawn()
            .insert(BuildGhost)
            .insert(Tool::Clear)
            .insert_bundle(PbrBundle {
                material: materials.add(StandardMaterial::unlit_color(Color::LIME_GREEN)),
                ..Default::default()
            });
    });
}

fn spawn_camera(mut cmds: Commands) {
    cmds.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController {
            enabled: true,
            mouse_rotate_sensitivity: vec2(0.002, 0.002),
            mouse_translate_sensitivity: vec2(0.04, 0.04),
            mouse_wheel_zoom_sensitivity: 0.01,
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

fn tool_ui(mut res_tool: ResMut<Tool>, egui_ctx: Res<EguiContext>) {
    let tool = &mut *res_tool;

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
