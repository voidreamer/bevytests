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
    pub shield: f32,
    pub max_shield: f32,
    pub souls: usize,
    pub last_damage_time: f32,
    pub last_stamina_usage: f32,
    pub last_shield_hit: f32,
}

impl Default for GameUI {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            shield: 80.0,
            max_shield: 100.0,
            souls: 0,
            last_damage_time: 0.0,
            last_stamina_usage: 0.0,
            last_shield_hit: 0.0,
        }
    }
}

// UI components
#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarBg;

#[derive(Component)]
pub struct StaminaBar;

#[derive(Component)]
pub struct ShieldBar;

#[derive(Component)]
pub struct SoulsCounter;

#[derive(Component)]
pub struct StaminaFlash;

#[derive(Component)]
pub struct SoulsText;

// Setup the UI system
pub fn setup_ui(mut commands: Commands) {
    println!("Setting up stacked bar UI system...");
    
    // Root node
    commands.spawn(
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        }
    )
    .with_children(|parent| {
        // Bar container in top-left
        parent.spawn(
            Node {
                width: Val::Px(350.0),
                height: Val::Auto,
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            }
        )
        .with_children(|parent| {
            // Health bar (top and thickest)
            parent.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(34.0),
                    margin: UiRect {
                        bottom: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                }
            )
            .with_children(|parent| {
                // Shadow effect (slight offset black box)
                parent.spawn(
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(3.0),
                        top: Val::Px(3.0),
                        ..default()
                    }
                )
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    ));
                });
                
                // Health bar background
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.0, 0.0, 0.7)),
                    HealthBarBg,
                ))
                .with_children(|parent| {
                    // Health bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(255.0, 0.0, 0.0)),
                        HealthBar,
                    ));
                    
                    // Add segments for visual effect
                    for i in 1..20 {
                        parent.spawn((
                            Node {
                                width: Val::Px(1.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                left: Val::Percent(i as f32 * 5.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                        ));
                    }
                });
            });
            
            // Stamina bar (middle and medium thickness)
            parent.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(20.0),
                    margin: UiRect {
                        bottom: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                }
            )
            .with_children(|parent| {
                // Shadow effect
                parent.spawn(
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(3.0),
                        top: Val::Px(3.0),
                        ..default()
                    }
                )
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    ));
                });
                
                // Stamina bar background
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.3, 0.0, 0.7)),
                ))
                .with_children(|parent| {
                    // Stamina bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                        StaminaBar,
                    ));
                    
                    // Stamina flash effect (initially invisible)
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                        StaminaFlash,
                    ));
                    
                    // Add segments for visual effect
                    for i in 1..20 {
                        parent.spawn((
                            Node {
                                width: Val::Px(1.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                left: Val::Percent(i as f32 * 5.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                        ));
                    }
                });
            });
            
            // Shield bar (bottom and thinnest)
            parent.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(16.0),
                    ..default()
                }
            )
            .with_children(|parent| {
                // Shadow effect
                parent.spawn(
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(3.0),
                        top: Val::Px(3.0),
                        ..default()
                    }
                )
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    ));
                });
                
                // Shield bar background
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.3, 0.7)),
                ))
                .with_children(|parent| {
                    // Shield bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(80.0),  // Default 80%
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.9)),
                        ShieldBar,
                    ));
                    
                    // Add segments for visual effect
                    for i in 1..20 {
                        parent.spawn((
                            Node {
                                width: Val::Px(1.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                left: Val::Percent(i as f32 * 5.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                        ));
                    }
                });
            });
        });
        
        // Souls counter - in top right corner
        parent.spawn((
            Node {
                width: Val::Px(150.0),
                height: Val::Px(60.0),
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                padding: UiRect::all(Val::Px(8.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
            SoulsCounter,
        ))
        .with_children(|parent| {
            // Shadow effect
            parent.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(3.0),
                    top: Val::Px(3.0),
                    ..default()
                }
            )
            .with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ));
            });
            
            // Souls icon (golden circle)
            parent.spawn(
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    margin: UiRect {
                        right: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                }
            )
            .with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.9, 0.8, 0.2, 0.9)),
                ));
            });
            
            // Souls count text - we'll add the SoulsText marker component here
            parent.spawn((
                Text::new("0"),
                TextColor(Color::srgba(0.9, 0.8, 0.3, 1.0)),
                SoulsText, // This marker component will help us target the text
            ));
        });
    });

    // Initialize the UI resource
    commands.insert_resource(GameUI::default());
}

// System to update game state from player data
pub fn update_game_state(
    mut ui_state: ResMut<GameUI>,
    player_query: Query<&Player>,
    time: Res<Time>,
) {
    // Get current time for animations
    let current_time = time.elapsed_secs();
    
    // Update from player stats
    if let Ok(player) = player_query.get_single() {
        // Check if health changed
        if player.health != ui_state.health {
            ui_state.last_damage_time = current_time;
        }
        
        // Check if stamina changed
        if player.stamina != ui_state.stamina && player.stamina < ui_state.stamina {
            ui_state.last_stamina_usage = current_time;
        }
        
        // Sync UI with player stats
        ui_state.health = player.health;
        ui_state.max_health = player.max_health;
        ui_state.stamina = player.stamina;
        ui_state.max_stamina = player.max_stamina;
        
        // Simulate shield damage when player is damaged
        if ui_state.last_damage_time == current_time && ui_state.shield > 10.0 {
            ui_state.shield -= 5.0;
            ui_state.last_shield_hit = current_time;
        }
        
        // Shield regeneration
        if current_time - ui_state.last_shield_hit > 3.0 && ui_state.shield < ui_state.max_shield {
            ui_state.shield = (ui_state.shield + 2.0 * time.delta_secs()).min(ui_state.max_shield);
        }
    }
}

// Update health bar width and effect
pub fn update_health_bar(
    mut q_health_bar: Query<&mut Node, With<HealthBar>>,
    mut q_health_bg: Query<&mut BackgroundColor, With<HealthBarBg>>,
    ui_state: Res<GameUI>,
    time: Res<Time>,
) {
    // Update health bar width
    if let Ok(mut health_node) = q_health_bar.get_single_mut() {
        health_node.width = Val::Percent((ui_state.health / ui_state.max_health) * 100.0);
    }
    
    // Flash effect for health bar background
    if let Ok(mut bg_color) = q_health_bg.get_single_mut() {
        let current_time = time.elapsed_secs();
        let damage_flash_time = 0.5;
        let time_since_damage = current_time - ui_state.last_damage_time;
        
        if time_since_damage < damage_flash_time {
            // Red pulsating effect when damaged
            let flash_intensity = 1.0 - time_since_damage / damage_flash_time;
            bg_color.0 = Color::srgba(0.4 + flash_intensity * 0.3, 0.0, 0.0, 0.7);
        } else {
            // Normal dark red background
            bg_color.0 = Color::srgba(0.3, 0.0, 0.0, 0.7);
        }
    }
}

// Update stamina bar width
pub fn update_stamina_bar(
    mut q_stamina_bar: Query<&mut Node, With<StaminaBar>>,
    ui_state: Res<GameUI>,
) {
    // Update stamina bar width
    if let Ok(mut stamina_node) = q_stamina_bar.get_single_mut() {
        stamina_node.width = Val::Percent((ui_state.stamina / ui_state.max_stamina) * 100.0);
    }
}

// Update stamina flash effect
pub fn update_stamina_flash(
    mut q_stamina_flash: Query<(&mut Node, &mut BackgroundColor), With<StaminaFlash>>,
    ui_state: Res<GameUI>,
    time: Res<Time>,
) {
    // Update stamina flash effect for when stamina is used
    if let Ok((mut flash_node, mut flash_color)) = q_stamina_flash.get_single_mut() {
        let current_time = time.elapsed_secs();
        let usage_flash_time = 0.4;
        let time_since_usage = current_time - ui_state.last_stamina_usage;
        
        if time_since_usage < usage_flash_time {
            // Stamina usage flash effect
            let flash_width = ((time_since_usage / usage_flash_time) * 100.0).min(100.0);
            flash_node.width = Val::Percent(flash_width);
            
            // Fade out
            let alpha = 0.5 * (1.0 - time_since_usage / usage_flash_time);
            flash_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
        } else {
            // Hide effect when not active
            flash_node.width = Val::Percent(0.0);
        }
    }
}

// Update shield bar width
pub fn update_shield_bar(
    mut q_shield_bar: Query<&mut Node, With<ShieldBar>>,
    ui_state: Res<GameUI>,
) {
    // Update shield bar width
    if let Ok(mut shield_node) = q_shield_bar.get_single_mut() {
        shield_node.width = Val::Percent((ui_state.shield / ui_state.max_shield) * 100.0);
    }
}

// Update souls counter - now with the correct component marker
pub fn update_souls_counter(
    mut q_souls_text: Query<&mut Text, With<SoulsText>>,
    ui_state: Res<GameUI>,
) {
    // Update souls counter
    if let Ok(mut souls_text) = q_souls_text.get_single_mut() {
        souls_text.0 = format!("{}", ui_state.souls);
    }
}

// Animate health bar color based on health level
pub fn animate_health_bar(
    mut q_health_bar: Query<&mut BackgroundColor, With<HealthBar>>,
    ui_state: Res<GameUI>,
    time: Res<Time>,
) {
    if let Ok(mut color) = q_health_bar.get_single_mut() {
        let health_percent = ui_state.health / ui_state.max_health;
        let t = time.elapsed_secs();
        
        if health_percent < 0.3 {
            // Critical health - pulsating red
            let pulse = (t * 3.0).sin() * 0.5 + 0.5;
            color.0 = Color::srgb(0.9, 0.1 + pulse * 0.1, 0.1 + pulse * 0.1);
        } else {
            // Normal health - gradient from red to yellow-red based on health
            color.0 = Color::srgb(0.9, 0.2 , 0.2);
        }
    }
}

// Animate shield bar color based on shield level
pub fn animate_shield_bar(
    mut q_shield_bar: Query<&mut BackgroundColor, With<ShieldBar>>,
    ui_state: Res<GameUI>,
    time: Res<Time>,
) {
    if let Ok(mut color) = q_shield_bar.get_single_mut() {
        let shield_percent = ui_state.shield / ui_state.max_shield;
        let t = time.elapsed_secs();
        
        // Subtle pulse effect
        let pulse = (t * 1.5).sin() * 0.1 + 0.9;
        
        if shield_percent < 0.3 {
            // Low shield - purple-ish
            color.0 = Color::srgb(0.5 * pulse, 0.2 * pulse, 0.8 * pulse);
        } else {
            // Normal shield - blue
            color.0 = Color::srgb(0.2 * pulse, 0.3 * pulse, 0.9 * pulse);
        }
    }
}

// Debug controls for testing UI
pub fn debug_ui_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>,
    mut ui_state: ResMut<GameUI>,
    time: Res<Time>,
) {
    // Get change amount based on time
    let change_amount = 20.0 * time.delta_secs();
    
    // Modify player stats directly if available
    if let Ok(mut player) = player_query.get_single_mut() {
        // Health controls
        if keyboard.pressed(KeyCode::KeyH) {
            player.health = (player.health - change_amount).max(0.0);
        }
        if keyboard.pressed(KeyCode::KeyJ) {
            player.health = (player.health + change_amount).min(player.max_health);
        }
    }
    
    // Controls that modify UI state directly
    
    // Add souls with S key
    if keyboard.just_pressed(KeyCode::KeyS) {
        ui_state.souls += 100;
        println!("Souls gained! Total: {}", ui_state.souls);
    }
    
    // Damage shield with K key
    if keyboard.just_pressed(KeyCode::KeyK) {
        ui_state.shield = (ui_state.shield - 10.0).max(0.0);
        ui_state.last_shield_hit = time.elapsed_secs();
        println!("Shield hit! Remaining: {}", ui_state.shield);
    }
    
    // Recover shield with L key
    if keyboard.just_pressed(KeyCode::KeyL) {
        ui_state.shield = ui_state.max_shield;
        println!("Shield fully recovered!");
    }
}

// UI plugin
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUI>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, (
               update_game_state,
               update_health_bar,
               update_stamina_bar,
               update_stamina_flash,
               update_shield_bar,
               update_souls_counter,
               animate_health_bar,
               animate_shield_bar,
               debug_ui_control,
           ));
    }
}