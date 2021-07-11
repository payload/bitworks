/*
This was a cody from a bevy_rapier2d example about player movement.
It is enhanced by adding another sprite and a torus mesh to the entity
that rotates with the linear velocity of the parents rigid body.
To set this up an extension trait was used to place to marker traits
into some fluent entity spawn API.
*/

use std::f32::consts::PI;

use bevy::ecs::system::EntityCommands;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::na::Vector2;
use bitworks::shape::Torus;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Player Movement Example".to_string(),
            width: 1000.0,
            height: 1000.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_player.system())
        .add_system(player_movement.system().label("movement"))
        .add_system(linked_rigidbody_rotation.system().after("movement"))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .run();
}

// The float value is the player movement speed in 'pixels/second'.
struct Player(f32);

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Set gravity to 0.0 and spawn camera.
    rapier_config.gravity = Vector2::zeros();
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());

    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    // While we want our sprite to look ~40 px square, we want to keep the physics units smaller
    // to prevent float rounding problems. To do this, we set the scale factor in RapierConfiguration
    // and divide our sprite_size by the scale.
    rapier_config.scale = 20.0;
    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    // Spawn entity with `Player` struct as a component for access in movement query.
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.7, 0.3).into()),
            sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle::default())
        .insert_bundle(ColliderBundle {
            position: [collider_size_x / 2.0, collider_size_y / 2.0].into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(0))
        .insert(Player(300.0))
        .link_rigidbody_linvel_to_children_rotation()
        .with_children(|child| {
            child
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::rgb(0.9, 0.5, 0.3).into()),
                    sprite: Sprite::new(Vec2::new(0.8 * sprite_size_x, 0.8 * sprite_size_y)),
                    transform: Transform::from_translation(vec3(0.0, 0.0, 1.0)),
                    ..Default::default()
                })
                .linked_rigidbody_rotation()
                .with_children(|child| {
                    child.spawn_bundle(SpriteBundle {
                        material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                        mesh: meshes.add(Mesh::from(Torus {
                            radius: 0.3 * sprite_size_x,
                            ring_radius: 0.05 * sprite_size_x,
                            ..Default::default()
                        })),
                        sprite: Sprite::new(Vec2::new(1.0, 1.0)),
                        transform: Transform {
                            translation: vec3(10.0, 0.0, 1.1),
                            rotation: Quat::from_rotation_x(0.5 * PI),
                            scale: Vec3::ONE,
                        },
                        ..Default::default()
                    });
                });
        });
}

trait EntityCommandsExt {
    fn link_rigidbody_linvel_to_children_rotation(&mut self) -> &mut Self;
    fn linked_rigidbody_rotation(&mut self) -> &mut Self;
}

impl<'a, 'b> EntityCommandsExt for EntityCommands<'a, 'b> {
    fn link_rigidbody_linvel_to_children_rotation(&mut self) -> &mut Self {
        self.insert(LinkRbLinvelRot)
    }

    fn linked_rigidbody_rotation(&mut self) -> &mut Self {
        self.insert(LinkedRbRot)
    }
}

struct LinkRbLinvelRot;
struct LinkedRbRot;

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_parameters: Res<RapierConfiguration>,
    mut player_info: Query<(&Player, &mut RigidBodyVelocity)>,
) {
    for (player, mut rb_vels) in player_info.iter_mut() {
        let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
        let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
        let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

        let x_axis = -(left as i8) + right as i8;
        let y_axis = -(down as i8) + up as i8;

        let mut move_delta = Vector2::new(x_axis as f32, y_axis as f32);
        if move_delta != Vector2::zeros() {
            // Note that the RapierConfiguration::Scale factor is also used here to transform
            // the move_delta from: 'pixels/second' to 'physics_units/second'
            move_delta /= move_delta.magnitude() * rapier_parameters.scale;
        }

        // Update the velocity on the rigid_body_component,
        // the bevy_rapier plugin will update the Sprite transform.
        rb_vels.linvel = move_delta * player.0;
    }
}

fn linked_rigidbody_rotation(
    link_parents: Query<(&Children, &LinkRbLinvelRot, &RigidBodyVelocity)>,
    mut linked: Query<(&mut Transform, &LinkedRbRot)>,
) {
    for (children, _, rb_vel) in link_parents.iter() {
        let linvel: Vector2<f32> = rb_vel.linvel;
        if linvel.magnitude_squared() > 0.1 {
            for child in children.iter() {
                if let Ok((mut transform, _)) = linked.get_mut(*child) {
                    let angle = linvel[1].atan2(linvel[0]);
                    transform.rotation = Quat::from_rotation_z(angle);
                }
            }
        }
    }
}
