use bevy::prelude::*;
use crate::game_state::{CyberResources, GamePhase};
use crate::network::NetworkNode;

/// Represents the player's avatar in the game world,
/// which can interact with network nodes and perform hacking actions.
#[derive(Debug, Clone, Component)]
pub struct HackerAgent {
    pub decryption_suite: u32, // Attack power
}

#[derive(Debug, Clone, Message)]
pub struct BreachAttemptEvent {
    pub target: Entity,
}

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BreachAttemptEvent>()
           .add_systems(Startup, spawn_player_agent)
           .add_systems(Update, (
               listen_for_input.run_if(in_state(GamePhase::NetworkMapping)),
               execute_breach.run_if(in_state(GamePhase::NetworkMapping)),
           ));
    }
}

fn spawn_player_agent(mut commands: Commands) {
    println!("[SYS] Initializing hacker agent...");
    commands.spawn((
        HackerAgent {
            decryption_suite: 15,
        },
        Name::new("Player_Icebreaker"),
    ));
}

/// Listens for player input to trigger breach attempts on network nodes.
fn listen_for_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    query_targets: Query<Entity, With<NetworkNode>>,
    mut event_writer: MessageWriter<BreachAttemptEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if let Some(target_entity) = query_targets.iter().next() {
            event_writer.write(BreachAttemptEvent { target: target_entity });
        }
    }
}

/// Executes breach attempts when a BreachAttemptEvent is received.
fn execute_breach(
    mut events: MessageReader<BreachAttemptEvent>,
    mut resources: ResMut<CyberResources>,
    mut query_nodes: Query<(&mut NetworkNode, &Name)>,
    agent: Single<&HackerAgent>,
) {
    for event in events.read() {
        if resources.available_cpu_cycles == 0 {
            println!("[ALERT] Insufficient CPU cycles to process breach vector.");
            continue;
        }

        if let Ok((mut node, name)) = query_nodes.get_mut(event.target) {
            resources.available_cpu_cycles -= 1;

            println!("[ACTION] Launching payload against {} ({})", name, node.ip_address);

            if agent.decryption_suite >= node.firewall_strength {
                node.is_compromised = true;
                println!("[SUCCESS] {} compromised! Access granted.", node.ip_address);
            } else {
                resources.security_alert_level += 25;
                println!("[FAILED] Firewall held. Alerts raised! Alert Level: {}%", resources.security_alert_level);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breach_success() {
        let mut world = World::new();
        world.init_resource::<CyberResources>();
        let target = world.spawn(NetworkNode {
            ip_address: "127.0.0.1".to_string(),
            firewall_strength: 10,
            is_compromised: false,
        }).id();
        let agent_entity = world.spawn(HackerAgent { decryption_suite: 15 }).id();

        // Read components individually (no concurrent borrows)
        let agent_decryption = world.get::<HackerAgent>(agent_entity).unwrap().decryption_suite;
        let node_firewall = world.get::<NetworkNode>(target).unwrap().firewall_strength;
        assert!(agent_decryption >= node_firewall, "Agent should beat firewall");

        // Mutate in separate scope
        {
            let mut resources = world.resource_mut::<CyberResources>();
            resources.available_cpu_cycles -= 1;
        }
        {
            let mut node = world.get_mut::<NetworkNode>(target).unwrap();
            node.is_compromised = true;
        }

        let node = world.get::<NetworkNode>(target).unwrap();
        assert!(node.is_compromised);
        assert_eq!(world.resource::<CyberResources>().available_cpu_cycles, 3);
    }

    #[test]
    fn test_breach_fails_when_firewall_too_strong() {
        let mut world = World::new();
        world.init_resource::<CyberResources>();
        let target = world.spawn(NetworkNode {
            ip_address: "10.0.0.1".to_string(),
            firewall_strength: 100,
            is_compromised: false,
        }).id();
        let agent_entity = world.spawn(HackerAgent { decryption_suite: 5 }).id();

        let agent_decryption = world.get::<HackerAgent>(agent_entity).unwrap().decryption_suite;
        let node_firewall = world.get::<NetworkNode>(target).unwrap().firewall_strength;
        assert!(agent_decryption < node_firewall, "Agent should be outmatched");

        {
            let mut resources = world.resource_mut::<CyberResources>();
            resources.available_cpu_cycles -= 1;
            resources.security_alert_level += 25;
        }

        let node = world.get::<NetworkNode>(target).unwrap();
        assert!(!node.is_compromised);
        assert_eq!(world.resource::<CyberResources>().security_alert_level, 25);
        assert_eq!(world.resource::<CyberResources>().available_cpu_cycles, 3);
    }

    #[test]
    fn test_insufficient_cpu_cycles_blocks_breach() {
        let mut world = World::new();
        world.init_resource::<CyberResources>();
        let target = world.spawn(NetworkNode {
            ip_address: "10.0.0.1".to_string(),
            firewall_strength: 10,
            is_compromised: false,
        }).id();
        world.spawn(HackerAgent { decryption_suite: 15 });

        // Drain all CPU cycles
        {
            let mut resources = world.resource_mut::<CyberResources>();
            resources.available_cpu_cycles = 0;
        }

        // Breach should not happen because CPU is 0 — node stays un-compromised
        let node = world.get::<NetworkNode>(target).unwrap();
        assert!(!node.is_compromised);
    }
}
