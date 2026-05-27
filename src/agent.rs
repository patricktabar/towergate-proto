use bevy::prelude::*;
use crate::game_state::{CyberResources, GamePhase};
use crate::network::NetworkNode;

/// Represents the player's avatar in the game world,
/// which can interact with network nodes and perform hacking actions.
#[derive(Debug, Clone, Component)]
pub struct HackerAgent {
    pub decryption_suite: u32, // Attack power
}

#[derive(Debug, Clone, Event)]
pub struct BreachAttemptEvent {
    pub target: Entity,
}

pub struct AgentPlugin;

/// Spawns the player's hacker agent into the game world at startup.
/// build do the following:
/// - add an event type for breach attempts, so we can trigger breaches from input systems
/// - at startup lauch the spawn_player_agent system to create the player's avatar
/// - at update phase, add the listen_for_input system to check for player
/// commands and trigger breach attempts
/// this run if the game state is NetworkMapping, so the player
/// can only attempt breaches during the initial scanning phase
impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
            app.add_event::<BreachAttemptEvent>()
               .add_systems(Startup, spawn_player_agent)
               .add_systems(
                   Update,
                   (
                       listen_for_input,
                       execute_breach
                   ).run_if(in_state(GamePhase::NetworkMapping))
               );
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
/// Takes in the following params:
/// - keyboard input resource to check for key presses,
/// - query to find target network nodes,
/// - event writer to send breach attempt events when the player initiates a breach.
fn listen_for_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    query_targets: Query<Entity, With<NetworkNode>>,
    mut event_writer: EventWriter<BreachAttemptEvent>,
) {
    if(keyboard.just_pressed(KeyCode::Space)) {
        if let Some(target_entity) = query_targets.iter().next() {
            event_writer.send(BreachAttemptEvent { target: target_entity });
        }
    }
}

/// Executes breach attempts when a BreachAttemptEvent is received.
/// takes in the following params:
/// - event reader to listen for breach attempt events,
/// - mutable resource for cyber resources to update CPU cycles and alert levels,
/// - query to access and modify target network nodes,
/// - query to access the hacker agent's stats for breach calculations.
/// and return the following:
/// - if the breach is successful, mark the node as compromised and print success message
/// - if the breach fails, increase the security alert level and print failure message
/// also checks if there are enough CPU cycles to attempt the breach, and if not, prints an alert and skips the attempt.
/// assumes there is exactly one hacker agent in the game world for simplicity.
/// this system runs during the NetworkMapping phase, so the player can only attempt breaches during the initial scanning phase of the game.
fn execute_breach(
    mut events: EventReader<BreachAttemptEvent>,
    mut resources: ResMut<CyberResource>,
    mut query_nodes: Query<(&mut NetworkNode, &Name)>,
    agent_query: Query<&HackerAgent>,
) {
    let agent = agent_query.single(); // Assumes exactly one agent exists

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
