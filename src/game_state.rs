use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

/// The current phase of the game, which determines
/// what systems are active and how the player
/// can interact with the game world.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, States)]
pub enum GamePhase {
    #[default]
    NetworkMapping, // Scanning the architecture
    NodeInfiltration, // Actively breaching a specific firewall
    GameOver,
}

/// Represents the player's current resources and status in the game.
#[allow(dead_code)]
#[derive(Resource, Debug, Clone)]
pub struct CyberResources {
    pub total_cpu_cycles: u32,
    pub available_cpu_cycles: u32,
    pub security_alert_level: u32, // Reaching 100 = Trace complete (Game Over)
}

impl Default for CyberResources {
    fn default() -> Self {
        CyberResources {
            total_cpu_cycles: 4,
            available_cpu_cycles: 4,
            security_alert_level: 0,
        }
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatesPlugin)
            .init_state::<GamePhase>()
            .insert_resource(CyberResources::default());
    }
}
