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

#[derive(Resource)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}

// ==============================================
// Setup player animation
// ==============================================
pub fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    println!("Setting up character animations...");

    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(3).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(4).from_asset(CHARACTER_PATH)),
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations{
        animations: node_indices,
        graph: graph_handle,
    });
}

pub fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[1], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}

pub fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut player_query: Query<&mut Player>,
    mut current_animation: Local<usize>,
    mut is_moving: Local<bool>,
    mut is_jumping: Local<bool>,
    mut is_attacking: Local<bool>,
    mut attack_timer: Local<Option<Timer>>,
    mut jump_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    // Initialize the attack timer if it's None
    if attack_timer.is_none() {
        *attack_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }
    
    // Initialize the jump timer if it's None
    if jump_timer.is_none() {
        *jump_timer = Some(Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once));
    }

    // Get player reference
    let mut player = if let Ok(p) = player_query.get_single_mut() {
        p
    } else {
        return;
    };

    for (mut anim_player, mut transitions) in &mut animation_players {
        // Update attack timer if we're attacking
        if *is_attacking {
            if let Some(timer) = attack_timer.as_mut() {
                // Progress timer
                timer.tick(time.delta());
                
                // If attack animation is complete, return to idle
                if timer.finished() {
                    *is_attacking = false;
                    *current_animation = 0;
                    
                    // Return to idle or running based on movement state
                    if player.is_moving && player.stamina > 10.0 && !player.exhausted {
                        transitions
                            .play(&mut anim_player, animations.animations[3], Duration::from_secs_f32(0.25));
                        *current_animation = 3;
                    } else {
                        transitions
                            .play(&mut anim_player, animations.animations[1], Duration::from_secs_f32(0.25))
                            .repeat();
                    }
                }
                
                // Skip other animation checks while attacking
                continue;
            }
        }
        
        // Update jump timer if we're jumping
        if *is_jumping {
            if let Some(timer) = jump_timer.as_mut() {
                // Scale animation rate based on stamina level
                let animation_rate = if player.stamina < 30.0 {
                    // Scale: 0.7 to 1.0 based on stamina
                    0.7 + (player.stamina / 30.0) * 0.3
                } else {
                    1.0
                };
                
                // Progress timer - adjust based on stamina level
                // If player is exhausted or has low stamina, the animation plays slower
                // which we simulate by advancing the timer more slowly
                let dt_scale = if player.exhausted { 
                    0.6
                } else { 
                    animation_rate 
                };
                let adjusted_dt = time.delta_secs() * dt_scale;
                timer.tick(Duration::from_secs_f32(adjusted_dt));
                
                // If jump animation is complete, return to idle or running
                if timer.finished() {
                    *is_jumping = false;
                    
                    // Return to running if W is still pressed and not exhausted, otherwise idle
                    if keyboard_input.pressed(KeyCode::KeyW) && player.stamina > 10.0 && !player.exhausted {
                        *is_moving = true;
                        *current_animation = 3;
                        transitions
                            .play(&mut anim_player, animations.animations[3], Duration::from_secs_f32(0.25))
                            .repeat();
                    } else {
                        *is_moving = false;
                        *current_animation = 0;
                        transitions
                            .play(&mut anim_player, animations.animations[1], Duration::from_secs_f32(0.25))
                            .repeat();
                    }
                }
                
                // Skip other animation checks while jumping
                continue;
            }
        }

        // Handle attack animation with left mouse button
        // Only if enough stamina and not exhausted
        if mouse_button_input.just_pressed(MouseButton::Left) && 
           !*is_attacking && !*is_jumping && 
           player.stamina >= 15.0 && !player.exhausted {
            
            *is_attacking = true;
            *is_moving = false;
            *is_jumping = false;
            *current_animation = 4;
            
            // Use stamina for attack
            player.stamina = (player.stamina - 15.0).max(0.0);
            
            // Start the attack animation and set the timer
            transitions
                .play(&mut anim_player, animations.animations[4], Duration::from_secs_f32(0.25));
            
            if let Some(timer) = attack_timer.as_mut() {
                // Set timer for the attack animation's duration
                timer.set_duration(Duration::from_secs_f32(1.5));
                timer.reset();
            }
            
            continue; // Skip other animation checks
        }

        // Only process other animations if we're not attacking or jumping
        if !*is_attacking && !*is_jumping {
            let was_moving = *is_moving;
            
            // Check for Space key (jump) - this now takes priority over running
            // Only jump if we have enough stamina and not exhausted
            if keyboard_input.just_pressed(KeyCode::Space) && 
               player.stamina >= 20.0 && !player.exhausted {
                
                *is_jumping = true;
                *current_animation = 2;
                
                // Play jump animation
                transitions
                    .play(&mut anim_player, animations.animations[2], Duration::from_secs_f32(0.25))
                    .repeat();
                
                // Set jump timer
                if let Some(timer) = jump_timer.as_mut() {
                    // Set jump animation duration - adjust as needed
                    timer.set_duration(Duration::from_secs_f32(1.0)); 
                    timer.reset();
                }
                
                continue;
            }
            // Check if W is pressed to trigger running animation
            // Only run if not exhausted
            else if keyboard_input.pressed(KeyCode::KeyW) && !player.exhausted {
                *is_moving = true;
                
                // Only switch if we weren't already moving or current animation is not running
                if !was_moving || *current_animation != 3 {
                    *current_animation = 3;
                    transitions
                        .play(&mut anim_player, animations.animations[3], Duration::from_secs_f32(0.25))
                        .repeat();
                }
            }
            // Use tired/slower walking animation when exhausted but trying to move
            else if keyboard_input.pressed(KeyCode::KeyW) && player.exhausted {
                *is_moving = true;
                
                // Special case - when exhausted but still trying to move
                if *current_animation != 1 {
                    *current_animation = 1; // Use default animation as "tired" animation
                    transitions
                        .play(&mut anim_player, animations.animations[1], Duration::from_secs_f32(0.25))
                        .repeat();
                }
            } else {
                // Return to idle when no keys are pressed
                *is_moving = false;
                
                if was_moving || (*current_animation != 0 && *current_animation != 1) {
                    *current_animation = 0;
                    transitions
                        .play(&mut anim_player, animations.animations[1], Duration::from_secs_f32(0.25))
                        .repeat();
                }
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
            .add_systems(Update, keyboard_animation_control);
    }
}