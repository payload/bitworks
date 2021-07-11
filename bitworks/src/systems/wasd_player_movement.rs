use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::na::Vector2;

pub struct WasdPlayerMovementPlugin;

impl Plugin for WasdPlayerMovementPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(player_movement.system());
    }
}

pub trait EntityCommandsExt {
    fn wasd_player_movement_insert_default_rb_collider(&mut self, sprite_size: Vec2, rapier_config: &RapierConfiguration) -> &mut Self;
}

impl<'a, 'b> EntityCommandsExt for EntityCommands<'a, 'b> {
    fn wasd_player_movement_insert_default_rb_collider(&mut self, sprite_size: Vec2, rapier_config: &RapierConfiguration) -> &mut Self {
        let collider_size_x = sprite_size.x / rapier_config.scale;
        let collider_size_y = sprite_size.y / rapier_config.scale;

        self.insert_bundle(RigidBodyBundle::default())
            .insert_bundle(ColliderBundle {
                position: [collider_size_x / 2.0, collider_size_y / 2.0].into(),
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete)
    }
}

pub struct WasdPlayerMovment {
    pub velocity: f32,
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_parameters: Res<RapierConfiguration>,
    mut player_info: Query<(&WasdPlayerMovment, &mut RigidBodyVelocity)>,
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

        rb_vels.linvel = move_delta * player.velocity;
    }
}
