use std::time::Duration;
use bevy::{
    prelude::*,
    input::{
        mouse::MouseButton,
        keyboard::KeyCode,
    },
};
use crate::player::Player;

const CHARACTER_PATH: &str = "models/character.glb";
// Better organized animation resource
#[derive(Resource)]
pub struct PlayerAnimations {
    pub idle: AnimationNodeIndex,
    pub tpose: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub roll: AnimationNodeIndex,  
    pub walk: AnimationNodeIndex,  
    pub graph: Handle<AnimationGraph>,
}

// Setup animations with the new structure
pub fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    println!("Setting up character animations...");

    // Load all animations like before
    let anim_handles = [
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(3).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(4).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(5).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(6).from_asset(CHARACTER_PATH)),
    ];

    // Create animation graph with all clips
    let (graph, node_indices) = AnimationGraph::from_clips(anim_handles);
    let graph_handle = graphs.add(graph);
    
    // Store with better naming
    commands.insert_resource(PlayerAnimations {
        tpose: node_indices[0],
        idle: node_indices[1],
        jump: node_indices[2],
        roll: node_indices[3],
        run: node_indices[4],
        attack: node_indices[5],
        walk: node_indices[6],
        graph: graph_handle,
    });
}

// Update scene once loaded - same as before but with new structure
pub fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<PlayerAnimations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        // Start with idle animation
        transitions
            .play(&mut player, animations.idle, Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}

pub fn keyboard_movement_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<PlayerAnimations>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
    mut is_moving: Local<bool>,
    mut current_animation: Local<usize>,
    mut is_jumping: Local<bool>,
    mut is_attacking: Local<bool>,
    mut is_rolling: Local<bool>,
    mut attack_timer: Local<Option<Timer>>,
    mut jump_timer: Local<Option<Timer>>,
    mut roll_timer: Local<Option<Timer>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    // Initialize timers if needed
    if attack_timer.is_none() {
        *attack_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }
    
    if jump_timer.is_none() {
        *jump_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }
    
    if roll_timer.is_none() {
        *roll_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }
    
    // Get camera for movement direction
    let camera_transform = if let Ok(camera) = camera_query.get_single() {
        camera
    } else {
        return;
    };
    
    // Get player transform
    if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
        // Update timers
        if let Some(timer) = attack_timer.as_mut() {
            timer.tick(time.delta());
            
            // Reset attack state when timer finishes
            if timer.finished() && *is_attacking {
                *is_attacking = false;
            }
        }
        
        if let Some(timer) = jump_timer.as_mut() {
            timer.tick(time.delta());
            
            // Reset jump state when timer finishes
            if timer.finished() && *is_jumping {
                *is_jumping = false;
            }
        }
        
        if let Some(timer) = roll_timer.as_mut() {
            timer.tick(time.delta());
            
            // Reset roll state when timer finishes
            if timer.finished() && *is_rolling {
                *is_rolling = false;
            }
        }
        
        // Calculate camera directions for movement
        let forward = camera_transform.forward();
        let camera_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
        let camera_right = camera_forward.cross(Vec3::Y).normalize();
        
        // Initialize movement direction
        let mut direction = Vec3::ZERO;
        let mut any_movement = false;
        
        // Check each movement key and add its contribution
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += camera_forward;
            any_movement = true;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= camera_forward;
            any_movement = true;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= camera_right;
            any_movement = true;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += camera_right;
            any_movement = true;
        }
        
        // Update player's moving state
        player.is_moving = any_movement;
        *is_moving = any_movement;
        
        // Apply movement if we have input and not in certain animations
        if any_movement && !*is_rolling {
            direction = direction.normalize();
            
            // Calculate movement speed based on player state
            let base_speed = 5.0; // Base movement speed
            let speed_multiplier = if player.exhausted {
                0.5 // Slower when exhausted
            } else if keyboard_input.pressed(KeyCode::ShiftLeft) && player.stamina > 10.0 {
                2.0 // Running speed
            } else {
                1.0 // Walking speed
            };
            
            // Apply movement
            let movement = direction * base_speed * speed_multiplier * time.delta_secs();
            transform.translation += movement;
            
            // Rotate player to face movement direction with smoothing
            if direction.length_squared() > 0.01 {
                let target_rotation = Quat::from_rotation_y(
                    f32::atan2(direction.x, direction.z)
                );
                
                // Smoothly rotate towards movement direction
                transform.rotation = transform.rotation.slerp(
                    target_rotation, 
                    10.0 * time.delta_secs()
                );
            }
        }
        
        // Handle animations
        for (mut anim_player, mut transitions) in &mut animation_players {
            // Priority order for animations:
            // 1. Attacking
            // 2. Rolling
            // 3. Jumping
            // 4. Running/Walking/Idle
            
            // Handle attack
            if *is_attacking {
                if let Some(timer) = attack_timer.as_mut() {
                    if timer.just_finished() {
                        *is_attacking = false;
                        // Animation will be handled in movement code below
                    } else {
                        continue; // Skip other animations while attacking
                    }
                }
            }
            
            // Check for new attack input
            if mouse_button_input.just_pressed(MouseButton::Left) && 
               !*is_attacking && !*is_jumping && !*is_rolling && 
               player.stamina >= 15.0 && !player.exhausted {
                
                *is_attacking = true;
                
                // Use stamina for attack
                player.stamina = (player.stamina - 15.0).max(0.0);
                
                // Play attack animation
                *current_animation = 5; // attack
                transitions
                    .play(&mut anim_player, animations.attack, Duration::from_secs_f32(0.15));
                
                if let Some(timer) = attack_timer.as_mut() {
                    timer.set_duration(Duration::from_secs_f32(1.5));
                    timer.reset();
                }
                
                continue;
            }
            
            // Handle roll
            if *is_rolling {
                if let Some(timer) = roll_timer.as_mut() {
                    if timer.just_finished() {
                        *is_rolling = false;
                        // Animation will be handled in movement code below
                    } else {
                        continue; // Skip other animations while rolling
                    }
                }
            }
            
            // Check for new roll input (Space key)
            if keyboard_input.just_pressed(KeyCode::Space) && 
               !*is_attacking && !*is_jumping && !*is_rolling && 
               player.stamina >= 25.0 && !player.exhausted {
                
                *is_rolling = true;
                
                // Use stamina for roll
                player.stamina = (player.stamina - 25.0).max(0.0);
                
                // Play roll animation
                *current_animation = 3; // roll
                transitions
                    .play(&mut anim_player, animations.roll, Duration::from_secs_f32(0.1))
                    .repeat();
                
                if let Some(timer) = roll_timer.as_mut() {
                    timer.set_duration(Duration::from_secs_f32(0.8));
                    timer.reset();
                }
                
                // Apply a quick movement in the direction the player is facing
                if any_movement {
                    // Roll in movement direction
                    transform.translation += direction * 2.0; // Quick boost
                } else {
                    // Roll forward if not moving
                    let forward_dir = transform.forward();
                    transform.translation += forward_dir * 2.0;
                }
                
                continue;
            }
            
            // Handle jump
            if *is_jumping {
                if let Some(timer) = jump_timer.as_mut() {
                    if timer.just_finished() {
                        *is_jumping = false;
                        // Animation will be handled in movement code below
                    } else {
                        continue; // Skip other animations while jumping
                    }
                }
            }
            
            // Check for new jump input (Ctrl key)
            if keyboard_input.just_pressed(KeyCode::ControlLeft) && 
               !*is_attacking && !*is_jumping && !*is_rolling && 
               player.stamina >= 20.0 && !player.exhausted {
                
                *is_jumping = true;
                
                // Use stamina for jump
                player.stamina = (player.stamina - 20.0).max(0.0);
                
                // Play jump animation
                *current_animation = 2; // jump
                transitions
                    .play(&mut anim_player, animations.jump, Duration::from_secs_f32(0.15))
                    .repeat();
                
                if let Some(timer) = jump_timer.as_mut() {
                    timer.set_duration(Duration::from_secs_f32(1.0));
                    timer.reset();
                }
                
                continue;
            }
            
            // If no special animation is active, handle movement animations
            if !*is_attacking && !*is_jumping && !*is_rolling {
                if any_movement {
                    let is_running = keyboard_input.pressed(KeyCode::ShiftLeft) && 
                                     player.stamina > 10.0 && 
                                     !player.exhausted;
                    
                    if is_running {
                        // Drain stamina while running
                        player.stamina = (player.stamina - 10.0 * time.delta_secs()).max(0.0);
                        
                        // Check if we need to become exhausted
                        if player.stamina <= 10.0 {
                            player.exhausted = true;
                        }
                        
                        // Play run animation if not already running
                        if *current_animation != 4 { // Run animation
                            *current_animation = 4;
                            transitions
                                .play(&mut anim_player, animations.run, Duration::from_secs_f32(0.25))
                                .repeat();
                            
                            //anim_player.set_playback_rate(1.0);
                        }
                    } else if player.exhausted {
                        // Play walk animation when exhausted
                        if *current_animation != 0 { // Walk animation
                            *current_animation = 0;
                            transitions
                                .play(&mut anim_player, animations.walk, Duration::from_secs_f32(0.25))
                                .repeat();
                            
                            // Slower animation when exhausted
                            //anim_player.set_playback_rate(0.6);
                        }
                    } else {
                        // Regular walking
                        if *current_animation != 0 { // Walk animation
                            *current_animation = 0;
                            transitions
                                .play(&mut anim_player, animations.walk, Duration::from_secs_f32(0.25))
                                .repeat();
                            
                            // Normal animation speed
                            //anim_player.set_playback_rate(1.0);
                        }
                    }
                } else {
                    // Idle when not moving
                    if *current_animation != 1 { // Idle animation
                        *current_animation = 1;
                        transitions
                            .play(&mut anim_player, animations.idle, Duration::from_secs_f32(0.25))
                            .repeat();
                    }
                }
            }
        }
        
        // Regenerate stamina when not running
        if !keyboard_input.pressed(KeyCode::ShiftLeft) || !player.is_moving {
            player.stamina = (player.stamina + 15.0 * time.delta_secs()).min(100.0);
            
            // Clear exhaustion if stamina recovered enough
            if player.stamina > 30.0 {
                player.exhausted = false;
            }
        }
    }
}

// Plugin for player animation 
pub struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_animations)
            .add_systems(Update, setup_scene_once_loaded)
            .add_systems(Update, keyboard_movement_control);
    }
}