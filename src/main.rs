mod game_state;
mod network;
mod agent;

use bevy::prelude::*;
use game_state::StatePlugin;
use network::NetworkPlugin;
use agent::AgentPlugin;

fn main() {
    App::new()
        // Minimal plugins avoid launching a heavy 3D window environment for now
        .add_plugins(MinimalPlugins)
        // Add our game mechanics domains
        .add_plugins(StatePlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(AgentPlugin)
        .run();
}
