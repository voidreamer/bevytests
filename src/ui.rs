use bevy::{
    prelude::*,
    input::keyboard::KeyCode,
};
use crate::player::Player;

// UI Resource to track game state
#[derive(Resource)]
pub struct GameUI {
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub souls: usize,
}

impl Default for GameUI {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            souls: 0,
        }
    }
}

// UI components
#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct StaminaBar;

#[derive(Component)]
pub struct SoulsCounter;

// Setup the UI system
pub fn setup_ui(mut commands: Commands) {
    println!("Setting up UI system...");
    
    // Root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Health bar container (top center)
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(30.0),
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        left: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::rgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|parent| {
                    // Health bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::rgb(0.8, 0.0, 0.0)),
                        HealthBar,
                    ));
                });
            
            // Stamina bar container
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(15.0),
                        position_type: PositionType::Absolute,
                        top: Val::Px(60.0),
                        left: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::rgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|parent| {
                    // Stamina bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::rgb(0.0, 0.8, 0.0)),
                        StaminaBar,
                    ));
                });
                
            // Souls counter
            parent.spawn((
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(40.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    right: Val::Px(20.0),
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::rgba(0.1, 0.1, 0.1, 0.8)),
                SoulsCounter,
                Text::new("Azurite: 0"),
                TextColor(Color::rgb(0.9, 0.8, 0.4)),
            ));
        });

    // Initialize the UI resource
    commands.insert_resource(GameUI::default());
}

// System to update UI based on player stats
pub fn update_ui(
    mut q_health_bar: Query<&mut Node, With<HealthBar>>,
    mut q_stamina_bar: Query<&mut Node, (With<StaminaBar>, Without<HealthBar>)>,
    mut q_souls_counter: Query<&mut Text, With<SoulsCounter>>,
    mut ui_state: ResMut<GameUI>,
    player_query: Query<&Player>,
) {
    // Update player health and stamina based on player component
    if let Ok(player) = player_query.get_single() {
        // Sync UI with player stats
        ui_state.health = player.health;
        ui_state.max_health = player.max_health;
        ui_state.stamina = player.stamina;
        ui_state.max_stamina = player.max_stamina;
    }

    // Update health bar width
    if let Ok(mut health_node) = q_health_bar.get_single_mut() {
        health_node.width = Val::Percent((ui_state.health / ui_state.max_health) * 100.0);
    }

    // Update stamina bar width
    if let Ok(mut stamina_node) = q_stamina_bar.get_single_mut() {
        stamina_node.width = Val::Percent((ui_state.stamina / ui_state.max_stamina) * 100.0);
    }
    
    // Update souls counter text
    if let Ok(mut text) = q_souls_counter.get_single_mut() {
        text.0 = format!("Souls: {}", ui_state.souls);
    }
}

// Debug system to test UI updates with keyboard
pub fn debug_ui_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>,
    mut ui_state: ResMut<GameUI>,
    time: Res<Time>,
) {
    // Directly modify player stats (health & stamina) for debugging
    if let Ok(mut player) = player_query.get_single_mut() {
        let change_amount = 20.0 * time.delta_secs();
        
        // Health controls
        if keyboard.pressed(KeyCode::KeyH) {
            player.health = (player.health - change_amount).max(0.0);
        }
        if keyboard.pressed(KeyCode::KeyJ) {
            player.health = (player.health + change_amount).min(player.max_health);
        }
        
        // Add souls with S key
        if keyboard.just_pressed(KeyCode::KeyS) {
            ui_state.souls += 100;
            println!("Souls gained! Total: {}", ui_state.souls);
        }
    }
}

// UI plugin
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUI>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, (update_ui, debug_ui_control));
    }
}