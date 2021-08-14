use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};

use bevy::{
    ecs::component::Component,
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec3Swizzles,
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
        // .add_plugin(RaycastPlugin)
        // and now the not-plugins
        .add_system(spawn_robodog.system())
        .add_system(update_robodog_seek.system())
        .add_system(update_robodog_movement.system())
        .add_startup_system(setup_sketch.system());
    app.run();
}

struct Robodog {
    avoid_radius: f32,
    avoid_force: Vec2,
    seek_entity: Entity,
    seek_force: Vec2,
    near_entity: Option<Entity>,
}

fn setup_sketch(mut cmds: Commands) {
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
    ));

    let e1 = cmds
        .spawn_bundle((
            Transform::from_translation(vec3(5.0, 0.0, -5.0)),
            GlobalTransform::identity(),
        ))
        .id();

    cmds.spawn_bundle((
        Robodog {
            avoid_radius: 0.0,
            avoid_force: Vec2::ZERO,
            seek_entity: e1,
            seek_force: Vec2::ZERO,
            near_entity: None,
        },
        Transform::from_translation(Vec3::ZERO),
        GlobalTransform::identity(),
    ));
}

fn spawn_robodog(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Robodog), Added<Robodog>>,
) {
    for (entity, robodog) in query.iter() {
        cmds.entity(entity).insert_bundle(PbrBundle {
            mesh: meshes.add(shape::Cube { size: 0.6 }.into()),
            ..Default::default()
        });
    }
}

// fn update_robodog_avoid(
//     mut query: Query<(Entity, &Transform, &mut Robodog)>,
//     avoid_q: Query<(Entity, &Transform, &AvoidLikeFire)>,
// ) {
//     let query_distance =
//         QueryByDistance::new(avoid_q.iter().map(|(e, t, _)| (e, t.translation.xz())));

//     for (entity, trans, mut robodog) in query.iter_mut() {
//         let pos = trans.translation.xz();

//         for avoid in query_distance.within_unsorted(pos, robodog.avoid_radius) {}

//         robodog.avoid_force = Vec2::ZERO;
//     }
// }

fn update_robodog_seek(
    mut query: Query<(Entity, &Transform, &mut Robodog)>,
    seek_q: Query<&Transform>,
) {
    for (entity, trans, mut robodog) in query.iter_mut() {
        if let Ok(seek_transform) = seek_q.get(robodog.seek_entity) {
            let pos = trans.translation.xz();
            let seek_pos = seek_transform.translation.xz();
            let seek_diff = seek_pos - pos;
            let seek_distance = seek_diff.length();
            let seek_dir = seek_diff / seek_distance;

            robodog.seek_force = if seek_distance > 1.0 {
                robodog.near_entity = None;
                seek_dir * 10.0
            } else {
                robodog.near_entity = Some(robodog.seek_entity);
                Vec2::ZERO
            };
        } else if robodog.seek_force != Vec2::ZERO {
            robodog.near_entity = None;
            robodog.seek_force = Vec2::ZERO;
        }
    }
}

fn update_robodog_movement(time: Res<Time>, mut query: Query<(Entity, &mut Transform, &Robodog)>) {
    let dt = time.delta_seconds();

    for (entity, mut trans, robodog) in query.iter_mut() {
        if robodog.seek_force != Vec2::ZERO {
            trans.translation += (robodog.seek_force * dt).extend(0.0).xzy();
        }
    }
}

fn update_robodog_do_things(mut query: Query<(Entity, &Transform, &mut Robodog)>) {
    for (entity, trans, mut robodog) in query.iter_mut() {
        if let Some(near) = robodog.near_entity {
            // do something
        }
    }
}

#[derive(Default)]
struct QueryByDistance {
    entities: Vec<(Entity, Vec2)>,
}

struct DistantEntity {
    entity: Entity,
    pos: Vec2,
    dir: Vec2,
    distance_squared: f32,
}

impl QueryByDistance {
    fn new<T: Iterator<Item = (Entity, Vec2)> + Sized>(mut entities: T) -> Self {
        Self {
            entities: entities.collect(),
        }
    }

    fn within_unsorted(&self, center: Vec2, radius: f32) -> Vec<DistantEntity> {
        let radius_squared = radius * radius;

        self.entities
            .iter()
            .filter_map(|(entity, pos)| {
                let diff: Vec2 = *pos - center;
                let distance_squared = diff.length_squared();
                let dir = diff / distance_squared / distance_squared;
                if distance_squared <= radius_squared {
                    Some(DistantEntity {
                        distance_squared,
                        entity: *entity,
                        pos: *pos,
                        dir,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

trait VecToTuple {
    fn tuple(&self) -> (f32, f32, f32);
}

impl VecToTuple for Vec3 {
    fn tuple(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}
