use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};

use bevy::{
    ecs::component::Component,
    input::{mouse::MouseButtonInput, ElementState},
    utils::HashMap,
};
use bitworks::*;

use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_picking::*;
use bevy_mod_raycast::*;
use smooth_bevy_cameras::controllers::orbit::{OrbitCameraBundle, OrbitCameraController};

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(VoxelPlugin)
        .add_plugin(EguiPlugin)
        // .add_plugin(DebugEventsPickingPlugin)
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
            .insert_resource(Models::default())
            .insert_resource(Field::default())
            .add_system(tool_ui.system())
            .add_system(build_on_click.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_raycast_with_cursor.system())
            .add_system(update_plane_selector_with_raycast_source.system())
            .add_system(update_build_ghost.system())
            .add_system(animation_system.system())
            .add_system(update_producers_system.system())
            .add_system(build_random_walker.system())
            .add_system(update_random_walker.system())
            .add_startup_system(spawn_camera.system())
            .add_startup_system(spawn_plane_selector.system())
            .add_startup_system(spawn_plane.system())
            .add_startup_system(setup_assets.system());
    }
}

//

struct Build;

#[derive(Default)]
struct RandomWalker {
    angle: f32,
    speed: f32,
    time: f32,
}

enum Product {
    RandomWalker,
}

struct ProducerEntry {
    time: f32,
    product: Product,
}

struct Producer {
    production: Vec<ProducerEntry>,
}

fn update_producers_system(
    mut cmds: Commands,
    mut producers: Query<(&GlobalTransform, &mut Producer)>,
    time: Res<Time>,
) {
    let cmds = &mut cmds;
    let dt = time.delta_seconds();

    for (global, mut producer) in producers.iter_mut() {
        let mut producer: Mut<Producer> = producer;
        for mut entry in producer.production.iter_mut() {
            entry.time = (entry.time - dt).max(0.0);

            if entry.time == 0.0 {
                match entry.product {
                    Product::RandomWalker => build(
                        cmds,
                        &Transform::from_translation(global.translation),
                        RandomWalker::default(),
                    ),
                }
            }
        }

        producer.production.retain(|e| e.time > 0.0);
    }

    fn build<C: Component>(cmds: &mut Commands, t: &Transform, comp: C) {
        cmds.spawn_bundle((Build, t.clone(), comp, GlobalTransform::identity()));
    }
}

fn build_random_walker(
    mut cmds: Commands,
    query: Query<Entity, (With<Build>, With<RandomWalker>)>,
    models: Res<Models>,
) {
    for entity in query.iter() {
        cmds.entity(entity)
            .remove::<Build>()
            .with_children(|parent| {
                parent.spawn_bundle(models.random_walker.bundle());
            });
    }
}

fn update_random_walker(time: Res<Time>, mut walkers: Query<(&mut Transform, &mut RandomWalker)>) {
    let dt = time.delta_seconds();

    for (mut transform, mut walker) in walkers.iter_mut() {
        walker.time += dt;

        let p: Vec2 = vec2(transform.translation.x, transform.translation.z);

        if walker.speed == 0.0 {
            //walker.angle = fastrand::f32() * 2.0 * PI;
            walker.angle = (-p.y).atan2(-p.x) + 0.2 * PI * (0.5 - fastrand::f32());
            walker.speed = 2.0 + 0.5 * fastrand::f32();
        }

        if p.distance(Vec2::ZERO) > 5.0 {
            walker.angle = (-p.y).atan2(-p.x) + 0.2 * PI * (0.5 - fastrand::f32());
        }

        transform.rotation = Quat::from_rotation_y(walker.angle);

        let step = walker.speed * dt;
        transform.translation.x += walker.angle.cos() * step;
        transform.translation.z += walker.angle.sin() * step;
    }
}

//

fn build_on_click(
    mut events: EventReader<MouseButtonInput>,
    plane_selector_query: Query<&Transform, With<PlaneSelector>>,
    tool: Res<Tool>,
    mut cmds: Commands,
    models: Res<Models>,
    mut field: ResMut<Field>,
) {
    let cmds = &mut cmds;
    let field = &mut field;

    for event in events.iter() {
        if let (MouseButton::Left, ElementState::Pressed) = (event.button, event.state) {
            if let Ok(transform) = plane_selector_query.single() {
                let tool = *tool;
                let transform = Transform::from_translation(transform.translation);

                match tool {
                    Tool::Clear => try_clear(cmds, transform, field),
                    Tool::Spring => try_build(cmds, transform, &models.building_spring, field),
                    Tool::Glassblower => {
                        try_build(cmds, transform, &models.building_glassblower, field)
                    }
                    Tool::Tap => try_build(cmds, transform, &models.building_tap, field),
                    Tool::Trash => try_build(cmds, transform, &models.building_trash, field),
                }
            }
        }
    }

    fn try_clear(cmds: &mut Commands, transform: Transform, field: &mut Field) {
        if let Some(entity) = field.clear_building(transform.translation) {
            cmds.entity(entity).despawn_recursive();
        }
    }

    fn try_build(cmds: &mut Commands, transform: Transform, model: &Model, field: &mut Field) {
        if field.has_building(transform.translation) {
            return;
        }

        let entity = cmds
            .spawn_bundle((
                transform,
                GlobalTransform::identity(),
                BuildAnimation::default(),
            ))
            .with_children(|parent| {
                parent
                    .spawn_bundle(model.bundle())
                    .insert_bundle(PickableBundle::default());
                parent.spawn_bundle((
                    GlobalTransform::identity(),
                    Transform::from_translation(vec3(1.0, 0.0, 0.0)),
                    Producer {
                        production: vec![ProducerEntry {
                            time: 1.0,
                            product: Product::RandomWalker,
                        }],
                    },
                ));
            })
            .id();

        field.set_building(transform.translation, entity);
    }
}

//

#[derive(Default)]
struct Field {
    cells: HashMap<IVec3, FieldCell>,
}

impl Field {
    fn clear_building(&mut self, at: Vec3) -> Option<Entity> {
        if let Some(cell) = self.cells.get_mut(&at.as_i32()) {
            cell.building.take()
        } else {
            None
        }
    }

    fn set_building(&mut self, at: Vec3, entity: Entity) {
        self.cells.entry(at.as_i32()).or_default().building = Some(entity);
    }

    fn has_building(&self, at: Vec3) -> bool {
        self.cells
            .get(&at.as_i32())
            .and_then(|cell| cell.building)
            .is_some()
    }
}

#[derive(Default)]
struct FieldCell {
    building: Option<Entity>,
}

//

#[derive(Default)]
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
        let bounce = 0.5 * ((5.0 * x * PI).cos() + 1.0);

        transform.translation.y = hull * bounce;

        animation.value += dt;
    }
}

//

#[derive(Default)]
pub struct Model {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
}

impl Model {
    fn bundle(&self) -> PbrBundle {
        PbrBundle {
            transform: self.transform,
            mesh: self.mesh.clone(),
            material: self.material.clone(),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Models {
    building_spring: Model,
    building_tap: Model,
    building_glassblower: Model,
    building_trash: Model,
    build_ghost_buildable: Handle<StandardMaterial>,
    build_ghost_not_buildable: Handle<StandardMaterial>,
    random_walker: Model,
}

fn setup_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut models: ResMut<Models>,
) {
    let black = materials.set("black", StandardMaterial::unlit_color(Color::BLACK));
    let green = materials.set("green", StandardMaterial::unlit_color(Color::LIME_GREEN));
    let red = materials.set("red", StandardMaterial::unlit_color(Color::ORANGE_RED));

    models.building_spring = Model {
        transform: Transform::from_translation(vec3(0.0, 0.3, 0.0)),
        material: black.clone(),
        mesh: meshes.add(shape::Cube { size: 0.6 }.into()),
    };

    models.building_glassblower = Model {
        transform: Transform::from_translation(vec3(0.0, 0.4, 0.0)),
        material: black.clone(),
        mesh: meshes.add(shape::Cube { size: 0.8 }.into()),
    };

    models.building_tap = Model {
        transform: Transform::from_translation(vec3(0.0, 0.3, 0.0)),
        material: black.clone(),
        mesh: meshes.add(
            shape::Icosphere {
                radius: 0.3,
                subdivisions: 1,
            }
            .into(),
        ),
    };

    models.building_trash = Model {
        transform: Transform::from_translation(vec3(0.0, 0.4, 0.0)),
        material: black.clone(),
        mesh: meshes.add(
            shape::Icosphere {
                radius: 0.4,
                subdivisions: 3,
            }
            .into(),
        ),
    };

    models.build_ghost_buildable = green;
    models.build_ghost_not_buildable = red;

    models.random_walker = Model {
        transform: Transform::from_translation(vec3(0.0, 0.4, 0.0)),
        material: black.clone(),
        mesh: meshes.add(
            shape::Torus {
                radius: 0.4,
                ring_radius: 0.1,
                subdivisions_segments: 3,
                subdivisions_sides: 3,
            }
            .into(),
        ),
    };
}

//

struct BuildGhost;
fn update_build_ghost(
    res_tool: Res<Tool>,
    models: Res<Models>,
    field: Res<Field>,
    mut ghost_query: Query<
        (
            &mut Tool,
            &mut Handle<Mesh>,
            &mut Visible,
            &mut Handle<StandardMaterial>,
            &GlobalTransform,
        ),
        With<BuildGhost>,
    >,
) {
    if let Ok((mut ghost_tool, mut handle_mesh, mut visible, mut material, transform)) =
        ghost_query.single_mut()
    {
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
                    Tool::Spring => models.building_spring.mesh.clone(),
                    Tool::Glassblower => models.building_glassblower.mesh.clone(),
                    Tool::Tap => models.building_tap.mesh.clone(),
                    Tool::Trash => models.building_trash.mesh.clone(),
                };
            }
        }

        match tool {
            Tool::Clear => {}
            Tool::Spring | Tool::Glassblower | Tool::Tap | Tool::Trash => {
                let new_material = if field.has_building(transform.translation) {
                    &models.build_ghost_not_buildable
                } else {
                    &models.build_ghost_buildable
                };

                if &*material != new_material {
                    *material = new_material.clone();
                }
            }
        };
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
            transform: Transform::from_translation(vec3(0.0, 0.0, 0.0)),
            material: materials.add(StandardMaterial::unlit_color(Color::DARK_GRAY)),
            ..Default::default()
        })
        .insert(RayCastMesh::<MyRaycastSet>::default());
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
            transform: Transform {
                translation: vec3(0.0, 0.1, 0.0),
                rotation: Quat::from_rotation_y(FRAC_PI_4),
                scale: Vec3::ONE,
            },
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
