use bevy::{
    prelude::*,
    input::{
        keyboard::KeyCode,
    },
};
use bevy_tnua::{
    builtins::{TnuaBuiltinDash, TnuaBuiltinJumpState},
    prelude::*, TnuaAnimatingState, TnuaAnimatingStateDirective, TnuaUserControlsSystemSet};
use std::time::Duration;

use crate::player::{Player, PlayerGltfHandle};


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackDirection {
    Forward,
    Left,
    Right,
    Backward
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayerAnimationState {
    Tpose,
    Idling,
    Jumping,
    Running,
    // Holds combo stage (0-2) and attack direction
    Attacking(u8, AttackDirection), 
    Rolling,
    Walking,
    Falling
}

// Animation state machine to handle complex transitions and interrupts
#[derive(Component)]
pub struct AnimationStateMachine {
    // Current state of animations
    pub current_state: PlayerAnimationState,
    pub previous_state: Option<PlayerAnimationState>,
    
    // Transition config
    pub interruptible: bool,
    pub transition_progress: f32,
    
    // Combo system
    pub combo_count: u8,
    pub combo_window_active: bool,
    pub combo_window_timer: f32,
    pub max_combo_chain: u8,
}

impl AnimationStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: PlayerAnimationState::Idling,
            previous_state: None,
            interruptible: true,
            transition_progress: 0.0,
            combo_count: 0,
            combo_window_active: false,
            combo_window_timer: 0.0,
            max_combo_chain: 3, // Support for 3-hit combo
        }
    }
    
    // Try to transition to new state, respecting interruption rules
    pub fn try_transition(&mut self, new_state: PlayerAnimationState, cancellation: Option<&AnimationCancellation>) -> bool {
        // Always allow transition if current state is interruptible
        if self.interruptible {
            self.previous_state = Some(self.current_state);
            self.current_state = new_state;
            self.transition_progress = 0.0;
            return true;
        }
        
        // Check if animation can be canceled via the cancellation system
        if let Some(cancel_info) = cancellation {
            if cancel_info.cancelable && cancel_info.current_time >= cancel_info.cancelable_after_time {
                // Check if current state can be canceled into the requested state
                if cancel_info.can_cancel_into.contains(&new_state) {
                    self.previous_state = Some(self.current_state);
                    self.current_state = new_state;
                    self.transition_progress = 0.0;
                    return true;
                }
            }
        }
        
        // Special case for attack combos
        if let PlayerAnimationState::Attacking(combo_stage, _) = self.current_state {
            if self.combo_window_active {
                if let PlayerAnimationState::Attacking(_, direction) = new_state {
                    // Advance combo if in window
                    let next_combo = (combo_stage + 1).min(self.max_combo_chain - 1);
                    self.previous_state = Some(self.current_state);
                    // Use the direction from the new attack input
                    self.current_state = PlayerAnimationState::Attacking(next_combo, direction);
                    self.combo_count = next_combo;
                    self.combo_window_active = false;
                    return true;
                }
            }
        }
        
        // Could not transition
        false
    }
    
    // Start a combo window - time during which next attack can be chained
    pub fn start_combo_window(&mut self, window_duration: f32) {
        self.combo_window_active = true;
        self.combo_window_timer = window_duration;
    }
    
    // Update combo window timer
    pub fn update(&mut self, delta_time: f32) {
        if self.combo_window_active {
            self.combo_window_timer -= delta_time;
            if self.combo_window_timer <= 0.0 {
                self.combo_window_active = false;
                
                // Reset combo if window expires
                if let PlayerAnimationState::Attacking(_, _) = self.current_state {
                    // Only reset if we're still in attack state and window closes
                    if !self.interruptible {
                        // Wait for animation to finish if not interruptible
                    } else {
                        self.combo_count = 0;
                    }
                } else {
                    self.combo_count = 0;
                }
            }
        }
        
        // Update transition progress
        self.transition_progress += delta_time;
    }
    
    // Set whether current animation can be interrupted
    pub fn set_interruptible(&mut self, interruptible: bool) {
        self.interruptible = interruptible;
    }
    
    // Reset combo counter
    pub fn reset_combo(&mut self) {
        self.combo_count = 0;
        self.combo_window_active = false;
    }
}

#[derive(Resource)]
pub struct PlayerAnimationNodes {
    pub idle: AnimationNodeIndex,
    pub tpose: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,      // First attack in combo (slash)
    pub attack2: AnimationNodeIndex,     // Second attack in combo (could be different move)
    pub attack3: AnimationNodeIndex,     // Third attack in combo (could be stronger finisher)
    pub roll: AnimationNodeIndex,  
    pub walk: AnimationNodeIndex,  
    pub fall: AnimationNodeIndex,  
}

// Marker component for animations that use root motion
#[derive(Component)]
pub struct RootMotionAnimation {
    pub enabled: bool,
    pub previous_root_transform: Option<Transform>,
    pub motion_strength: f32,
}

// Component to track which animations can be canceled and into what states
#[derive(Component, Default)]
pub struct AnimationCancellation {
    pub cancelable: bool,
    pub cancelable_after_time: f32,  // Time after which animation can be canceled (seconds)
    pub current_time: f32,           // Current time in animation
    pub priority: u8,                // Priority of current animation (higher can cancel lower)
    pub can_cancel_into: Vec<PlayerAnimationState>,  // States this animation can cancel into
}

// Setup animations with the new structure
pub fn setup_animations(
    handle: Option<Res<PlayerGltfHandle>>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
    animation_player_query: Query<Entity, With<AnimationPlayer>>,
    mut animation_graphs_assets: ResMut<Assets<AnimationGraph>>,
    mut players: Query<(Entity, &AnimationPlayer), Added<AnimationPlayer>>,
) {
    // Initialize players with animations if they're new
    for (entity, _player) in &mut players {
        let transitions = AnimationTransitions::new();
        
        // We'll set up initial animations later when the PlayerAnimationNodes are ready
        // This is just to initialize the transitions component
        commands.entity(entity).insert(transitions);
    };

    let Some(handle) = handle else { return };
    let Some(gltf) = gltf_assets.get(&handle.0) else {
        return;
    };
    let Ok(animation_player_entity) = animation_player_query.get_single() else {
        return;
    };

    let mut graph = AnimationGraph::new();
    let root_node = graph.root;

    // For simplicity, we'll reuse the slash animation for each combo stage
    // In a real game, you would have separate animations for each stage
    let slash_anim = gltf.named_animations["slash"].clone();
    
    commands.insert_resource(PlayerAnimationNodes{
        tpose: graph.add_clip(gltf.named_animations["tpose"].clone(), 1.0, root_node),
        idle: graph.add_clip(gltf.named_animations["idle"].clone(), 1.0, root_node),
        roll: graph.add_clip(gltf.named_animations["roll"].clone(), 1.0, root_node),
        walk: graph.add_clip(gltf.named_animations["walk"].clone(), 1.0, root_node),
        run: graph.add_clip(gltf.named_animations["run"].clone(), 1.0, root_node),
        jump: graph.add_clip(gltf.named_animations["jump"].clone(), 1.0, root_node),
        attack: graph.add_clip(slash_anim.clone(), 1.0, root_node),
        // For demo purposes, we'll use the same slash animation for all combo stages
        // with different playback speeds to simulate different attacks
        attack2: graph.add_clip(slash_anim.clone(), 1.0, root_node),
        attack3: graph.add_clip(slash_anim.clone(), 1.0, root_node),
        fall: graph.add_clip(gltf.named_animations["fall"].clone(), 1.0, root_node),
    });

    commands
        .entity(animation_player_entity)
        .insert(AnimationGraphHandle(animation_graphs_assets.add(graph)));

    // So that we won't run this again
    commands.remove_resource::<PlayerGltfHandle>();
}
/*
pub fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<PlayerAnimationNodes>,
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
    animations: Res<PlayerAnimationNodes>,
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
                
                /*
                // Apply a quick movement in the direction the player is facing
                if any_movement {
                    // Roll in movement direction
                    transform.translation += direction * 2.0; // Quick boost
                } else {
                    // Roll forward if not moving
                    let forward_dir = transform.forward();
                    transform.translation += forward_dir * 2.0;
                }
                */
                
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
            
            // Check for jump input (Ctrl key)
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
    }
}

*/
fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut TnuaController, &mut Player, &mut AnimationStateMachine, &mut AnimationCancellation)>,
    camera_query: Query<&Transform, With<Camera3d>>,
    time: Res<Time>,
    mut attack_timer: Local<Option<Timer>>,
    mut combo_window_timer: Local<Option<Timer>>,
) {
    let Ok((mut controller, mut player, mut state_machine, mut anim_cancellation)) = query.get_single_mut() else {
        return;
    };
    
    // Initialize timers if needed
    if attack_timer.is_none() {
        *attack_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }
    
    if combo_window_timer.is_none() {
        *combo_window_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }
    
    // Handle timers
    if let Some(timer) = attack_timer.as_mut() {
        timer.tick(time.delta());
        
        // Update the animation cancellation system's time tracking
        if player.is_attacking {
            anim_cancellation.current_time += time.delta_secs();
            
            // Make attacks cancelable after a certain time (0.3 seconds)
            if anim_cancellation.current_time >= 0.3 && !anim_cancellation.cancelable {
                anim_cancellation.cancelable = true;
                anim_cancellation.cancelable_after_time = 0.3;
                
                // Set which states this attack can be canceled into
                anim_cancellation.can_cancel_into = vec![
                    PlayerAnimationState::Rolling,
                    PlayerAnimationState::Jumping,
                    // Add directions to make the compiler happy with the complex enum
                    PlayerAnimationState::Attacking(0, AttackDirection::Forward), 
                    PlayerAnimationState::Attacking(0, AttackDirection::Left),
                    PlayerAnimationState::Attacking(0, AttackDirection::Right),
                    PlayerAnimationState::Attacking(0, AttackDirection::Backward),
                ];
            }
        }
        
        // Attack animation finished
        if timer.just_finished() && player.is_attacking {
            player.is_attacking = false;
            state_machine.set_interruptible(true);
            
            // Reset animation cancellation state
            anim_cancellation.cancelable = false;
            anim_cancellation.current_time = 0.0;
            
            // Start combo window after attack finishes
            if let PlayerAnimationState::Attacking(combo, _) = state_machine.current_state {
                if combo < state_machine.max_combo_chain - 1 {
                    // Only open combo window if we haven't reached max combo
                    if let Some(combo_timer) = combo_window_timer.as_mut() {
                        combo_timer.set_duration(Duration::from_secs_f32(0.5)); // 0.5s combo window
                        combo_timer.reset();
                        state_machine.start_combo_window(0.5);
                    }
                } else {
                    // Reset combo after final hit
                    state_machine.reset_combo();
                }
            }
        }
    }
    
    if let Some(timer) = combo_window_timer.as_mut() {
        timer.tick(time.delta());
        
        // Update state machine timer
        state_machine.update(time.delta_secs());
    }

    // Get camera for movement direction
    let camera_transform = if let Ok(camera) = camera_query.get_single() {
        camera
    } else {
        return;
    };

    // Calculate camera directions for movement
    let forward = camera_transform.forward();
    let camera_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
    let camera_right = camera_forward.cross(Vec3::Y).normalize();
    
    // Initialize movement direction
    let mut direction = Vec3::ZERO;

    // Check each movement key and add its contribution
    if keyboard.pressed(KeyCode::KeyW) {
        direction += camera_forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= camera_forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= camera_right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += camera_right;
    }

    // Update player's moving state
    player.is_moving = direction != Vec3::ZERO;

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    
    // For characters with front in negative-Z direction, we need to invert the direction for desired_forward
    // This makes the character face the direction it's moving instead of backward
    let forward_dir = if direction != Vec3::ZERO {
        // Negate the direction to correct model orientation
        -direction
    } else {
        // If not moving, use camera forward as default orientation
        -camera_forward
    };
    
    // Calculate speed based on stamina
    let dt = time.delta_secs();
    // First, check if we need to update exhaustion state
    if player.stamina <= 10.0 && !player.exhausted {
        player.exhausted = true;
        player.exhaustion_timer = 3.0; // 3 seconds of exhaustion
    }
    
    let speed_modifier = if player.exhausted {
        0.5 // Very slow when exhausted
    } else if keyboard.pressed(KeyCode::ShiftLeft) && player.stamina > 10.0 {
        // Running speed when shift is pressed and enough stamina
        2.0
    } else {
        1.0
    };
    
    let base_speed = 4.0;
    let current_speed = base_speed * speed_modifier;
    
    // Handle stamina regeneration/depletion
    if player.is_moving {
        // Only use stamina when running (shift pressed)
        if keyboard.pressed(KeyCode::ShiftLeft) && !player.exhausted {
            // Deplete stamina only when running
            player.stamina = (player.stamina - player.stamina_use_rate * dt).max(0.0);
            
            // Check if we've reached exhaustion
            if player.stamina <= 10.0 && !player.exhausted {
                player.exhausted = true;
                player.exhaustion_timer = 2.0; // 2 seconds of exhaustion
            }
        } else if !player.exhausted {
            // When walking (not running), slowly regenerate stamina
            player.stamina = (player.stamina + player.stamina_regen_rate * 0.2 * dt).min(player.max_stamina);
        }
    } else if !player.exhausted {
        // Regenerate stamina faster when not moving and not exhausted
        player.stamina = (player.stamina + player.stamina_regen_rate * dt).min(player.max_stamina);
    } else {
        // Handle exhaustion recovery timer
        player.exhaustion_timer -= dt;
        if player.exhaustion_timer <= 0.0 {
            player.exhausted = false;
        }
        
        // Slower regeneration when exhausted
        if player.stamina < 30.0 {
            player.stamina = (player.stamina + player.stamina_regen_rate * 0.3 * dt).min(player.max_stamina);
        }
    }
    
    controller.basis(TnuaBuiltinWalk{
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * current_speed,
        // Make the character face in the opposite direction of movement
        desired_forward: Dir3::new(forward_dir).ok(),
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 0.1,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::ControlLeft) && player.stamina >= 10.0 && !player.exhausted {
        // Use stamina for jumping
        player.stamina = (player.stamina - 1.0).max(0.0);
        
        controller.action(TnuaBuiltinJump{
            // The height is the only mandatory field of the jump button.
            height: 3.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }

    if keyboard.pressed(KeyCode::Space) && player.stamina >= 10.0 && !player.exhausted && !player.is_attacking {
        // Use stamina for rolling
        player.stamina = (player.stamina - 1.0).max(0.0);
        
        // Get the movement direction based on what direction player is going
        let dash_direction = if direction != Vec3::ZERO {
            // Use player's current movement direction
            direction.normalize()
        } else {
            // If standing still, dash forward relative to camera
            camera_forward
        };
        
        controller.action(TnuaBuiltinDash{
            displacement: dash_direction * 3.0, // Increased distance
            speed: 5.0, // Increased speed
            ..Default::default()
        });
    }
    
    // Handle attack action with left mouse button
    if mouse_input.just_pressed(MouseButton::Left) && player.stamina >= 15.0 && !player.exhausted {
        let in_combo_window = state_machine.combo_window_active;
        
        if !player.is_attacking || in_combo_window || anim_cancellation.cancelable {
            // Determine attack direction based on movement keys
            let attack_direction = determine_attack_direction(&keyboard, &camera_transform.rotation);
            
            // Determine combo stage
            let combo_stage = if in_combo_window {
                // This will be a combo continuation
                if let PlayerAnimationState::Attacking(current_stage, _) = state_machine.current_state {
                    (current_stage + 1).min(state_machine.max_combo_chain - 1)
                } else {
                    0 // Shouldn't reach here, but just in case
                }
            } else {
                // This is a new attack
                0
            };
            
            // Try to transition to attacking state with direction
            let new_state = PlayerAnimationState::Attacking(combo_stage, attack_direction);
            
            if state_machine.try_transition(new_state, Some(&anim_cancellation)) {
                player.is_attacking = true;
                
                // Use stamina for attack (costs more for later combo stages)
                let stamina_cost = 15.0 + (combo_stage as f32 * 5.0);
                player.stamina = (player.stamina - stamina_cost).max(0.0);
                
                // Make animation non-interruptible at start
                state_machine.set_interruptible(false);
                
                // Reset cancellation system for new attack
                anim_cancellation.cancelable = false;
                anim_cancellation.current_time = 0.0;
                
                // Set attack timer - duration depends on combo stage
                if let Some(timer) = attack_timer.as_mut() {
                    // Each successive attack in combo is slightly faster
                    let duration = match combo_stage {
                        0 => 1.0,     // First attack: 1 second
                        1 => 0.8,     // Second attack: 0.8 seconds
                        _ => 0.6,     // Third attack: 0.6 seconds (faster finisher)
                    };
                    
                    timer.set_duration(Duration::from_secs_f32(duration));
                    timer.reset();
                }
                
                // Close combo window since we used it
                if in_combo_window {
                    state_machine.combo_window_active = false;
                }
            }
        }
    }
}

// Helper function to determine attack direction based on keyboard input
fn determine_attack_direction(keyboard: &ButtonInput<KeyCode>, _rotation: &Quat) -> AttackDirection {
    let forward_pressed = keyboard.pressed(KeyCode::KeyW);
    let backward_pressed = keyboard.pressed(KeyCode::KeyS);
    let left_pressed = keyboard.pressed(KeyCode::KeyA);
    let right_pressed = keyboard.pressed(KeyCode::KeyD);
    
    // Determine direction based on which keys are pressed
    if forward_pressed && !backward_pressed && !left_pressed && !right_pressed {
        AttackDirection::Forward
    } else if backward_pressed && !forward_pressed && !left_pressed && !right_pressed {
        AttackDirection::Backward
    } else if left_pressed && !right_pressed && !forward_pressed && !backward_pressed {
        AttackDirection::Left
    } else if right_pressed && !left_pressed && !forward_pressed && !backward_pressed {
        AttackDirection::Right
    } else {
        // Default to forward attack when no direction keys or multiple direction keys are pressed
        // Could be enhanced to use camera/player facing instead
        AttackDirection::Forward
    }
}

fn handle_animating(
    mut player_query: Query<(&TnuaController, &mut TnuaAnimatingState<PlayerAnimationState>, &Player, &AnimationStateMachine, &AnimationCancellation)>,
    mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animation_nodes: Option<Res<PlayerAnimationNodes>>,
    keyboard: Res<ButtonInput<KeyCode>>, 
) {
    // An actual game should match the animation player and the controller. Here we cheat for
    // simplicity and use the only controller and only player.
    let Ok((controller, mut animating_state, player, state_machine, _animation_cancellation)) = player_query.get_single_mut() else {
        return;
    };
    let Ok((mut animation_player, mut transitions)) = animation_query.get_single_mut() else {
        return;
    };
    let Some(animation_nodes) = animation_nodes else {
        return;
    };
    
    // Define transition durations for different animation states
    // Create a transition map for smoother animations
    let common_transition = Duration::from_secs_f32(0.2);
    let fast_transition = Duration::from_secs_f32(0.1);
    let very_fast_transition = Duration::from_secs_f32(0.05);
    
    // Default transition times
    let idle_transition = Duration::from_secs_f32(0.25);
    let walk_transition = Duration::from_secs_f32(0.25);
    let run_transition = Duration::from_secs_f32(0.2);
    let jump_transition = Duration::from_secs_f32(0.15);
    let fall_transition = Duration::from_secs_f32(0.15);
    let attack_transition = Duration::from_secs_f32(0.1);
    let roll_transition = Duration::from_secs_f32(0.1);
    
    // Note: In a more comprehensive implementation, we'd use animation blend spaces
    // to blend between different movement animations based on direction and speed

    // Here we use the data from TnuaController to determine what the character is currently doing,
    // so that we can later use that information to decide which animation to play.

    // Use the state machine as source of truth for animation state
    // This is a major improvement over the previous implementation
    let current_status_for_animating = if player.is_attacking {
        // When attacking, use the exact combo stage and direction from the state machine
        if let PlayerAnimationState::Attacking(combo_stage, direction) = state_machine.current_state {
            PlayerAnimationState::Attacking(combo_stage, direction)
        } else {
            // Fallback - should rarely happen
            PlayerAnimationState::Attacking(0, AttackDirection::Forward)
        }
    } else {
        // For non-attack states, determine based on physics state
        match controller.action_name() {
        // Unless you provide the action names yourself, prefer matching against the `NAME` const
        // of the `TnuaAction` trait. Once `type_name` is stabilized as `const` Tnua will use it to
        // generate these names automatically, which may result in a change to the name.
        Some(TnuaBuiltinJump::NAME) => {
            // In case of jump, we want to cast it so that we can get the concrete jump state.
            let (_, jump_state) = controller
                .concrete_action::<TnuaBuiltinJump>()
                .expect("action name mismatch");
            // Depending on the state of the jump, we need to decide if we want to play the jump
            // animation or the fall animation.
            match jump_state {
                TnuaBuiltinJumpState::NoJump => PlayerAnimationState::Idling,
                TnuaBuiltinJumpState::StartingJump { .. } => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::MaintainingJump => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::StoppedMaintainingJump => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::FallSection => PlayerAnimationState::Falling,
            }
        }
        Some(TnuaBuiltinDash::NAME) => PlayerAnimationState::Rolling,
        // Tnua should only have the `action_name` of the actions you feed to it. If it has
        // anything else - consider it a bug.
        Some(other) => panic!("Unknown action {other}"),
        // No action name means that no action is currently being performed - which means the
        // animation should be decided by the basis.
        None => {
            // If there is no action going on, we'll base the animation on the state of the
            // basis.
            let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                // Since we only use the walk basis in this example, if we can't get get this
                // basis' state it probably means the system ran before any basis was set, so we
                // just stkip this frame.
                return;
            };
            if basis_state.standing_on_entity().is_none() {
                // The walk basis keeps track of what the character is standing on. If it doesn't
                // stand on anything, `standing_on_entity` will be empty - which means the
                // character has walked off a cliff and needs to fall.
                PlayerAnimationState::Falling
            } else {
                let speed = basis_state.running_velocity.length();
                if 0.01 < speed {
                    // Use player state from the query
                    if player.exhausted {
                        PlayerAnimationState::Walking
                    } else if keyboard.pressed(KeyCode::ShiftLeft) {
                        PlayerAnimationState::Running
                    } else {
                        PlayerAnimationState::Walking
                    }
                } else {
                    PlayerAnimationState::Idling
                }
            }
        }
    }
    };

    let animating_directive = animating_state.update_by_discriminant(current_status_for_animating);

    match animating_directive {
        TnuaAnimatingStateDirective::Maintain { state } => {
            // `Maintain` means that we did not switch to a different variant, so there is no need
            // to change animations.

            // Specifically for the running animation, even when the state remains the speed can
            // still change. When it does, we simply need to update the speed in the animation
            // player.
            if let PlayerAnimationState::Running = state {
                if let Some(_) = animation_player.animation_mut(animation_nodes.run) {
                    // Check if player is exhausted
                    if player.exhausted {
                        // Use transition when going from running to walking due to exhaustion
                        transitions
                            .play(&mut animation_player, animation_nodes.walk, walk_transition)
                            .set_speed(0.6)  // Slower speed when exhausted
                            .repeat();
                    }
                }
            }
        }
        TnuaAnimatingStateDirective::Alter {
            old_state,
            state,
        } => {
            // `Alter` means that we have switched to a different variant and need to play a
            // different animation with proper transitions.

            // Instead of stopping all animations, we'll use transitions between states
            match state {
                PlayerAnimationState::Idling => {
                    // Different transition times for idle depending on previous state
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Walking) => fast_transition,
                        Some(PlayerAnimationState::Running) => common_transition,
                        Some(PlayerAnimationState::Jumping) |
                        Some(PlayerAnimationState::Falling) => common_transition,
                        Some(PlayerAnimationState::Rolling) => common_transition,
                        _ => idle_transition,
                    };
                    
                    transitions
                        .play(&mut animation_player, animation_nodes.idle, transition_time)
                        .set_speed(1.0)
                        .repeat();
                }
                PlayerAnimationState::Walking => {
                    // Use dynamic transition based on previous state
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Idling) => very_fast_transition,
                        Some(PlayerAnimationState::Running) => fast_transition,
                        Some(PlayerAnimationState::Falling) => common_transition,
                        Some(PlayerAnimationState::Jumping) => common_transition,
                        Some(PlayerAnimationState::Rolling) => fast_transition,
                        _ => walk_transition,
                    };
                    
                    transitions
                        .play(&mut animation_player, animation_nodes.walk, transition_time)
                        .set_speed(1.0)
                        .repeat();
                }
                PlayerAnimationState::Falling => {
                    // Different transition times depending on previous state
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Jumping) => very_fast_transition,
                        _ => fall_transition,
                    };
                    
                    transitions
                        .play(&mut animation_player, animation_nodes.fall, transition_time)
                        .set_speed(1.0)
                        .repeat();
                }
                PlayerAnimationState::Running => {
                    // Faster transition from walking to running
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Walking) => very_fast_transition,
                        Some(PlayerAnimationState::Jumping) => fast_transition,
                        Some(PlayerAnimationState::Rolling) => fast_transition,
                        _ => run_transition,
                    };
                    
                    transitions
                        .play(&mut animation_player, animation_nodes.run, transition_time)
                        .set_speed(1.0)
                        .repeat();
                }
                PlayerAnimationState::Jumping => {
                    // Use fastest transition when jumping from running for a more responsive feel
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Running) => very_fast_transition,
                        Some(PlayerAnimationState::Walking) => fast_transition,
                        _ => jump_transition,
                    };
                    
                    transitions
                        .play(&mut animation_player, animation_nodes.jump, transition_time)
                        .set_speed(1.0);
                }
                PlayerAnimationState::Attacking(combo_stage, direction) => {
                    // Fast transition to attack for responsiveness
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Running) |
                        Some(PlayerAnimationState::Walking) => very_fast_transition,
                        // Quick transition between combo stages
                        Some(PlayerAnimationState::Attacking(_, _)) => very_fast_transition,
                        _ => attack_transition,
                    };
                    
                    // Choose animation based on combo stage
                    let base_animation_node = match combo_stage {
                        0 => animation_nodes.attack,    // First attack - normal slash
                        1 => animation_nodes.attack2,   // Second attack - slightly faster
                        _ => animation_nodes.attack3,   // Final attack - much faster, more powerful
                    };
                    
                    // Adjust speed and rotation based on direction
                    // For actual directional attacks, you would use different animation clips
                    // Here we're simulating it with rotation adjustments
                    let (speed, _rotation_offset) = match direction {
                        AttackDirection::Forward => (1.8, 0.0),           // Normal forward attack
                        AttackDirection::Left => (1.7, -std::f32::consts::FRAC_PI_4), // Slightly slower side attack
                        AttackDirection::Right => (1.7, std::f32::consts::FRAC_PI_4), // Slightly slower side attack
                        AttackDirection::Backward => (2.0, std::f32::consts::PI), // Faster backward attack
                    };
                    
                    // Apply speed boost for higher combo stages
                    let combo_speed_boost = match combo_stage {
                        0 => 1.0,
                        1 => 1.1,
                        _ => 1.3, // Final attack is much faster
                    };
                    
                    transitions
                        .play(&mut animation_player, base_animation_node, transition_time)
                        .set_speed(speed * combo_speed_boost);
                        
                    // Note: In a real implementation, you would select different animations
                    // for different directions rather than just speed adjustments
                }
                PlayerAnimationState::Rolling => {
                    // Fast transition to roll for responsiveness
                    let transition_time = match old_state {
                        Some(PlayerAnimationState::Running) |
                        Some(PlayerAnimationState::Walking) => very_fast_transition,
                        _ => roll_transition,
                    };
                    
                    transitions
                        .play(&mut animation_player, animation_nodes.roll, transition_time)
                        .set_speed(1.5);
                }
                PlayerAnimationState::Tpose => {
                    transitions
                        .play(&mut animation_player, animation_nodes.tpose, Duration::ZERO)
                        .set_speed(0.0);
                }
            }
        }
    }
}

// Apply root motion from animations to character movement
fn apply_root_motion(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &Player, &mut RootMotionAnimation)>,
    animation_query: Query<&Transform, (With<SceneRoot>, Without<Player>)>,
) {
    // Only apply root motion to enabled animations
    for (mut transform, player, mut root_motion) in player_query.iter_mut() {
        if !root_motion.enabled {
            continue;
        }
        
        // Find the animation root node's transform
        for scene_transform in animation_query.iter() {
            // First time setup - store initial transform
            if root_motion.previous_root_transform.is_none() {
                root_motion.previous_root_transform = Some(*scene_transform);
                continue;
            }
            
            // Calculate the movement delta from the previous frame
            let prev_transform = root_motion.previous_root_transform.unwrap_or(*scene_transform);
            let motion_delta = scene_transform.translation - prev_transform.translation;
            
            // Don't apply vertical motion from animations - physics should handle that
            let planar_delta = Vec3::new(motion_delta.x, 0.0, motion_delta.z);
            
            // Only apply root motion for certain animations
            let motion_factor = if player.is_attacking || player.is_moving {
                root_motion.motion_strength * time.delta_secs() * 60.0
            } else {
                0.0
            };
            
            // Apply the motion to the actual transform
            transform.translation += planar_delta * motion_factor;
            
            // Store current transform for next frame
            root_motion.previous_root_transform = Some(*scene_transform);
            
            // Only process first scene transform
            break;
        }
    }
}

// Initialize player animations once the animation nodes are loaded
fn initialize_player_animations(
    animations: Option<Res<PlayerAnimationNodes>>,
    mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut already_initialized: Local<bool>,
) {
    // Only run this once
    if *already_initialized {
        return;
    }
    
    let Some(animations) = animations else {
        return;
    };
    
    for (mut player, mut transitions) in &mut animation_query {
        // Start with idle animation, using a seamless transition
        transitions
            .play(&mut player, animations.idle, Duration::from_secs_f32(0.25))
            .repeat();
        
        // Mark as initialized to avoid running this again
        *already_initialized = true;
    }
}

pub struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_animations)
            .add_systems(FixedUpdate, (
                apply_controls.in_set(TnuaUserControlsSystemSet),
                setup_animations,
                initialize_player_animations,
                handle_animating,
            ))
            // Add root motion system after animation updates
            .add_systems(PostUpdate, apply_root_motion);
    }
}