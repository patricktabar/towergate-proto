use bevy::prelude::*;

/// Represents a node in the network architecture
/// that the player can interact with.
#[derive(Debug, Clone, Component)]
pub struct NetworkNode {
    pub ip_address: String,
    pub firewall_strength: u32,
    pub is_compromised: bool,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Startup systems run exactly once when the plugin loads
        app.add_system(Startup, spawn_initial_subnet);
    }
}

pub fn spawn_initial_subnet(mut commands: Commands) {
    println!("[SYS] Generating local virtual subnet architecture...");

    // Spawn Gateway Node
    commands.spawn((
        NetworkNode {
            ip_address: "192.168.1.1".to_string(),
                        firewall_strength: 10,
                        is_compromised: false,
        },
        Name::new("Gateway Router"),
    ));

    // Spawn Workstation Node
    commands.spawn((
        NetworkNode {
            ip_address: "10.0.4.23".to_string(),
            firewall_strength: 45,
            is_compromised: false,
        },
        Name::new("Mainframe Core"),
    ));
}
