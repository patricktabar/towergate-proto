use bevy::prelude::*;

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

// Precise marker components to target our live UI elements
#[derive(Component)]
pub struct CpuDisplayMarker;

#[derive(Component)]
pub struct AlertDisplayMarker;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePhase>()
            .insert_resource(CyberResources::default())
            .add_systems(Startup, spawn_terminal_hud)
            .add_systems(Update, refresh_terminal_hud);
    }
}

fn spawn_terminal_hud(mut commands: Commands) {
    // Parent Flexbox Layout - covers entire engine view window canvas
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(20.0)),
        ..default()
    })
    .with_children(|terminal| {
        // Top HUD Row Configuration Banner
        terminal.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::linear_rgb(0.0, 0.05, 0.0)),
        ))
        .with_children(|header| {
            // Identity Logo Text Element
            header.spawn((
                Text::new("CORE_UPLINK // TOWERGATE_OS"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::linear_rgb(0.0, 0.9, 0.0)),
            ));

            // Real-time CPU Cycle Tracking Node
            header.spawn((
                Text::new("CPU: 4/4"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::linear_rgb(0.0, 1.0, 0.0)),
                CpuDisplayMarker,
            ));

            // Threat Trace Percentage Node
            header.spawn((
                Text::new("ALERT DETECT: 0%"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::linear_rgb(1.0, 0.1, 0.1)),
                AlertDisplayMarker,
            ));
        });

        // Main Matrix Workspace panel viewport
        terminal.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(90.0),
            margin: UiRect::top(Val::Px(15.0)),
            ..default()
        });
    });
}

/// Decoupled update system utilizing native 0.18 Option<Single> parameters.
/// This prevents any possibility of runtime query panics or layout conflicts.
fn refresh_terminal_hud(
    resources: Res<CyberResources>,
    mut params: ParamSet<(
        Option<Single<&mut Text, With<CpuDisplayMarker>>>,
        Option<Single<&mut Text, With<AlertDisplayMarker>>>,
    )>,
) {
    // Only run mutability hooks when resources alter
    if resources.is_changed() {
        if let Some(mut text) = params.p0() {
            text.0 = format!("CPU: {}/{}", resources.available_cpu_cycles, resources.total_cpu_cycles);
        }
        if let Some(mut text) = params.p1() {
            text.0 = format!("ALERT DETECT: {}%", resources.security_alert_level);
        }
    }
}
