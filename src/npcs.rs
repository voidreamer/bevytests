use bevy::prelude::*;
use avian3d::prelude::{Collider, RigidBody};
use bevy::input::keyboard::KeyCode;
use std::time::Duration;

use crate::player::Player;
use crate::progression::{StatAllocationEvent, PlayerProgress};

pub struct NpcsPlugin;

impl Plugin for NpcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_npcs)
           .add_systems(Update, (
                handle_npc_interaction,
                update_interaction_prompt,
                handle_feedback_text,
                debug_gain_souls, // Add debug system to gain souls with a key
            ));
    }
}

#[derive(Component)]
pub struct LevelUpStation {
    pub interaction_range: f32,
    pub can_interact: bool,
}

#[derive(Component)]
pub struct InteractionPrompt;

fn create_prompt_text() -> Text {
    "Press E to level up".into()
}

fn spawn_npcs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a level-up station at the edge of the map
    let level_station_mesh = meshes.add(Cylinder::new(1.0, 3.0));
    let level_station_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.0, 0.5), // Purple
        emissive: Color::srgb(0.5, 0.0, 0.5).into(), // Convert to LinearRgba
        perceptual_roughness: 0.3,
        metallic: 0.8,
        ..default()
    });

    // Spawn the level-up station
    commands.spawn((
        Name::new("Level Up Station"),
        Mesh3d(level_station_mesh),
        MeshMaterial3d(level_station_material),
        Transform::from_xyz(10.0, 1.5, 10.0),
        RigidBody::Static,
        Collider::cylinder(1.0, 1.5),
        LevelUpStation {
            interaction_range: 3.0,
            can_interact: false,
        },
    ));
    
    // Create interaction prompt text following the pattern from ui.rs
    commands.spawn((
        create_prompt_text(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Percent(50.0),
            display: Display::None, // Hidden by default
            ..default()
        },
        InteractionPrompt,
    ));
}

// System to check if player is in range of the level up station
fn update_interaction_prompt(
    mut level_stations: Query<(&Transform, &mut LevelUpStation)>,
    player_query: Query<&Transform, With<Player>>,
    mut prompt_query: Query<&mut Node, With<InteractionPrompt>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (station_transform, mut station) in &mut level_stations {
            let distance = player_transform.translation.distance(station_transform.translation);
            
            // Check if player is in range
            if distance <= station.interaction_range {
                station.can_interact = true;
                
                // Show interaction prompt
                if let Ok(mut node) = prompt_query.get_single_mut() {
                    node.display = Display::Flex;
                }
            } else {
                station.can_interact = false;
                
                // Hide interaction prompt
                if let Ok(mut node) = prompt_query.get_single_mut() {
                    node.display = Display::None;
                }
            }
        }
    }
}

// Define the cost in souls for leveling up
const LEVEL_UP_COST: u32 = 100;
// Define how much health and stamina is gained per level up
const HEALTH_PER_LEVEL: f32 = 10.0;
const STAMINA_PER_LEVEL: f32 = 5.0;

// Component for the level-up feedback message
#[derive(Component)]
pub struct LevelUpFeedback {
    timer: Timer,
}


// System to handle interaction with the level up station
fn handle_npc_interaction(
    level_stations: Query<&LevelUpStation>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_progress: ResMut<PlayerProgress>,
    mut player_query: Query<&mut Player>,
    mut stat_events: EventWriter<StatAllocationEvent>,
    mut commands: Commands,
    mut game_ui: ResMut<crate::ui::GameUI>,
    _time: Res<Time>,
) {
    // Check if any level station is in range
    let can_interact = level_stations.iter().any(|station| station.can_interact);
    
    if can_interact && keyboard.just_pressed(KeyCode::KeyE) {
        // Check if player has enough souls to level up
        if game_ui.souls >= LEVEL_UP_COST as usize {
            // Consume souls for leveling up
            game_ui.souls -= LEVEL_UP_COST as usize;
            
            // Spawn a temporary feedback text for souls spent
            commands.spawn((
                Text::new(format!("- {} souls", LEVEL_UP_COST)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(200.0),
                    left: Val::Percent(50.0),
                    ..default()
                },
                LevelUpFeedback {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
            ));
            
            // Grant experience directly to progress toward level
            player_progress.experience += LEVEL_UP_COST;
            
            info!("Spent {} souls. Total XP: {}/{}", 
                LEVEL_UP_COST, 
                player_progress.experience, 
                player_progress.experience_to_next_level
            );
            
            // Allocate any available stat points automatically
            if player_progress.available_stat_points > 0 {
                // Allocate to vigor (health) for testing
                stat_events.send(StatAllocationEvent {
                    stat_name: "vigor".to_string(),
                    amount: 1,
                });
                
                // Spawn a temporary feedback text for stat allocation
                commands.spawn((
                    Text::new(format!("Vigor increased to {}", player_progress.vigor + 1)),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(230.0),
                        left: Val::Percent(50.0),
                        ..default()
                    },
                    LevelUpFeedback {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
                
                info!("Allocated 1 point to vigor. Remaining points: {}", 
                    player_progress.available_stat_points - 1
                );
                
                // Directly increase player health and stamina
                if let Ok(mut player) = player_query.get_single_mut() {
                    // Store old maximums for reference
                    let old_max_health = player.max_health;
                    let old_max_stamina = player.max_stamina;
                    
                    // Increase max health and stamina
                    player.max_health += HEALTH_PER_LEVEL;
                    player.max_stamina += STAMINA_PER_LEVEL;
                    
                    // Also heal the player to full on level up
                    player.health = player.max_health;
                    player.stamina = player.max_stamina;
                    
                    // Update the stat growth tracker for visualization
                    // Create visual growth indicators to show bars expanding
                    for i in 0..5 {
                        // Health growth indicator
                        commands.spawn((
                            Text::new("+❤️"),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(30.0 + (i as f32 * 3.0)),
                                left: Val::Px(20.0 + (i as f32 * 10.0)), 
                                ..default()
                            },
                            LevelUpFeedback {
                                timer: Timer::from_seconds(2.0, TimerMode::Once),
                            },
                        ));
                        
                        // Stamina growth indicator 
                        commands.spawn((
                            Text::new("+💪"),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(60.0 + (i as f32 * 3.0)),
                                left: Val::Px(20.0 + (i as f32 * 10.0)),
                                ..default()
                            },
                            LevelUpFeedback {
                                timer: Timer::from_seconds(2.0, TimerMode::Once),
                            },
                        ));
                    }
                    
                    // Calculate the percentage increase for feedback
                    let health_percent_increase = HEALTH_PER_LEVEL / old_max_health * 100.0;
                    let stamina_percent_increase = STAMINA_PER_LEVEL / old_max_stamina * 100.0;
                    
                    // Spawn feedback text for increased stats with percentage
                    commands.spawn((
                        Text::new(format!("Max Health +{} ({:.1}%), Max Stamina +{} ({:.1}%)", 
                            HEALTH_PER_LEVEL, health_percent_increase,
                            STAMINA_PER_LEVEL, stamina_percent_increase)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(260.0),
                            left: Val::Percent(50.0),
                            ..default()
                        },
                        LevelUpFeedback {
                            timer: Timer::from_seconds(2.5, TimerMode::Once), // Give a bit more time to read
                        },
                    ));
                    
                    // Display a "LEVEL UP!" text with larger font for emphasis
                    commands.spawn((
                        Text::new("LEVEL UP!"),
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(170.0), // Position above the other messages
                            left: Val::Percent(50.0),
                            ..default()
                        },
                        LevelUpFeedback {
                            timer: Timer::from_seconds(3.0, TimerMode::Once),
                        },
                    ));
                    
                    info!("Player stats increased! Health: {} (+{}), Stamina: {} (+{})", 
                        player.max_health, HEALTH_PER_LEVEL,
                        player.max_stamina, STAMINA_PER_LEVEL);
                }
            }
        } else {
            // Not enough souls - show error message
            commands.spawn((
                Text::new(format!("Not enough souls! Need {}", LEVEL_UP_COST)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(200.0),
                    left: Val::Percent(50.0),
                    ..default()
                },
                LevelUpFeedback {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
            ));
            
            info!("Not enough souls to level up. Have: {}, Need: {}", 
                game_ui.souls, LEVEL_UP_COST);
        }
    }
}

// System to handle the feedback text lifetime
fn handle_feedback_text(
    mut commands: Commands,
    time: Res<Time>,
    mut feedback_query: Query<(Entity, &mut LevelUpFeedback)>,
) {
    for (entity, mut feedback) in &mut feedback_query {
        feedback.timer.tick(Duration::from_secs_f32(time.delta_secs()));
        
        if feedback.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}


// DEBUG: System to gain souls with G key for testing
fn debug_gain_souls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_ui: ResMut<crate::ui::GameUI>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        // Add 100 souls for testing
        game_ui.souls += 100;
        
        commands.spawn((
            Text::new("+ 100 souls (debug)"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(300.0),
                left: Val::Percent(50.0),
                ..default()
            },
            LevelUpFeedback {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
        
        info!("DEBUG: Added 100 souls. Total: {}", game_ui.souls);
    }
}