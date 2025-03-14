use std::time::Duration;
use bevy::{
    prelude::*,
    input::{
        mouse::MouseButton,
        keyboard::KeyCode,
    },
};
use bevy_tnua::{
    builtins::{TnuaBuiltinDash, TnuaBuiltinJumpState},
    prelude::*, TnuaAnimatingState, TnuaAnimatingStateDirective, TnuaUserControlsSystemSet};

use crate::player::{Player, PlayerGltfHandle};

const CHARACTER_PATH: &str = "models/character.glb";

pub enum PlayerAnimationState {
    Tpose,
    Idling,
    Jumping,
    Running(f32),
    Attacking,
    Rolling,
    Walking,
    Falling
}

#[derive(Resource)]
pub struct PlayerAnimationNodes {
    pub idle: AnimationNodeIndex,
    pub tpose: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub roll: AnimationNodeIndex,  
    pub walk: AnimationNodeIndex,  
    pub fall: AnimationNodeIndex,  
}

// Setup animations with the new structure
pub fn setup_animations(
    handle: Option<Res<PlayerGltfHandle>>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
    animation_player_query: Query<Entity, With<AnimationPlayer>>,
    mut animation_graphs_assets: ResMut<Assets<AnimationGraph>>,
) {
    let Some(handle) = handle else { return };
    let Some(gltf) = gltf_assets.get(&handle.0) else {
        return;
    };
    let Ok(animation_player_entity) = animation_player_query.get_single() else {
        return;
    };

    let mut graph = AnimationGraph::new();
    let root_node = graph.root;

    commands.insert_resource(PlayerAnimationNodes{
        tpose: graph.add_clip(gltf.named_animations["tpose"].clone(), 1.0, root_node),
        idle: graph.add_clip(gltf.named_animations["idle"].clone(), 1.0, root_node),
        roll: graph.add_clip(gltf.named_animations["roll"].clone(), 1.0, root_node),
        walk: graph.add_clip(gltf.named_animations["walk"].clone(), 1.0, root_node),
        run: graph.add_clip(gltf.named_animations["run"].clone(), 1.0, root_node),
        jump: graph.add_clip(gltf.named_animations["jump"].clone(), 1.0, root_node),
        attack: graph.add_clip(gltf.named_animations["slash"].clone(), 1.0, root_node),
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
*/
fn apply_controls(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut TnuaController>) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk{
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 10.0,
        desired_forward: Dir3::new(direction).ok(),
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 2.0,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::ControlLeft) {
        controller.action(TnuaBuiltinJump{
            // The height is the only mandatory field of the jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinDash{
            ..Default::default()
        });
    }
}

// This is the important system for this example
fn handle_animating(
    mut player_query: Query<(&TnuaController, &mut TnuaAnimatingState<PlayerAnimationState>)>,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    animation_nodes: Option<Res<PlayerAnimationNodes>>,
) {
    // An actual game should match the animation player and the controller. Here we cheat for
    // simplicity and use the only controller and only player.
    let Ok((controller, mut animating_state)) = player_query.get_single_mut() else {
        return;
    };
    let Ok(mut animation_player) = animation_player_query.get_single_mut() else {
        return;
    };
    let Some(animation_nodes) = animation_nodes else {
        return;
    };

    // Here we use the data from TnuaController to determine what the character is currently doing,
    // so that we can later use that information to decide which animation to play.

    // First we look at the `action_name` to determine which action (if at all) the character is
    // currently performing:
    let current_status_for_animating = match controller.action_name() {
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
                TnuaBuiltinJumpState::NoJump => return,
                TnuaBuiltinJumpState::StartingJump { .. } => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::MaintainingJump => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::StoppedMaintainingJump => PlayerAnimationState::Jumping,
                TnuaBuiltinJumpState::FallSection => PlayerAnimationState::Falling,
            }
        }
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
                    PlayerAnimationState::Running(0.1 * speed)
                } else {
                    PlayerAnimationState::Idling
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
            if let PlayerAnimationState::Running(speed) = state {
                if let Some(animation) = animation_player.animation_mut(animation_nodes.run) {
                    animation.set_speed(*speed);
                }
            }
        }
        TnuaAnimatingStateDirective::Alter {
            old_state: _,
            state,
        } => {
            // `Alter` means that we have switched to a different variant and need to play a
            // different animation.

            // First - stop the currently running animation. We don't check which one is running
            // here because we just assume it belongs to the old state, but more sophisticated code
            // can try to phase from the old animation to the new one.
            animation_player.stop_all();

            // Depending on the new state, we choose the animation to run and its parameters (here
            // they are the speed and whether or not to repeat)
            match state {
                PlayerAnimationState::Idling=> {
                    animation_player
                        .start(animation_nodes.idle)
                        .set_speed(1.0)
                        .repeat();
                }
                PlayerAnimationState::Walking=> {
                    animation_player
                        .start(animation_nodes.walk)
                        .set_speed(0.2)
                        .repeat();
                }
                PlayerAnimationState::Falling=> {
                    animation_player
                        .start(animation_nodes.fall)
                        .set_speed(0.3)
                        .repeat();
                }
                PlayerAnimationState::Running(speed) => {
                    animation_player
                        .start(animation_nodes.run)
                        // The running animation, in particular, has a speed that depends on how
                        // fast the character is running. Note that if the speed changes while the
                        // character is still running we won't get `Alter` again - so it's
                        // important to also update the speed in `Maintain { State: Running }`.
                        .set_speed(*speed)
                        .repeat();
                }
                PlayerAnimationState::Jumping => {
                    animation_player
                        .start(animation_nodes.jump)
                        .set_speed(1.0);
                }
                PlayerAnimationState::Attacking=> {
                    animation_player
                        .start(animation_nodes.attack)
                        .set_speed(2.0);
                }
                PlayerAnimationState::Rolling=> {
                    animation_player
                        .start(animation_nodes.roll)
                        .set_speed(2.0);
                }
                PlayerAnimationState::Tpose=> {
                    animation_player
                        .start(animation_nodes.tpose)
                        .set_speed(0.0);
                }
            }
        }
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
                handle_animating,
            ),
        );
    }
}