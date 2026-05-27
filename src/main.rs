mod game_state;
mod network;
mod agent;

use bevy::prelude::*;

use game_state::StatePlugin;
use network::NetworkPlugin;
use agent::AgentPlugin;

fn main() {
    App::new()
        // Inject DefaultPlugins configured with specific terminal window rules
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TOWERGATE // Network Intrusion Terminal".to_string(),
                resolution: (1024, 768).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Seed our baseline dark background theme
        .insert_resource(ClearColor(Color::BLACK))

        // Load our decoupled strategy plugins
        .add_plugins(StatePlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(AgentPlugin)

        // Spawn our modern graphics view camera
        .add_systems(Startup, setup_terminal_camera)
        .run();
}

fn setup_terminal_camera(mut commands: Commands) {
    // In modern Bevy, Camera2d implicitly configures your Required Components (Transform, Visibility, etc.)
    commands.spawn(Camera2d);
}
