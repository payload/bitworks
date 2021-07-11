use bevy::{
    input::{keyboard::KeyboardInput, system::exit_on_esc_system, ElementState},
    prelude::*,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    GameRunning,
    GamePaused,
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(exit_on_esc_system.system())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                game_pause_running_switch_system.system(),
            );
    }
}

fn game_pause_running_switch_system(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut app_state: ResMut<State<AppState>>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Released && key_code == KeyCode::Return {
                let new_state = match app_state.current() {
                    AppState::GamePaused => AppState::GameRunning,
                    AppState::GameRunning => AppState::GamePaused,
                };
                info!("{:?} => {:?}", app_state.current(), new_state);
                app_state
                    .set(new_state)
                    .expect("state change pause running");
            }
        }
    }
}
