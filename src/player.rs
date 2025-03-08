use bevy::{
    prelude::*,
    input::keyboard::KeyCode,
};
use crate::camera::ThirdPersonCamera;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub turn_speed: f32,
    pub gravity: f32,
    pub jump_force: f32,
    pub ground_offset: f32,
    pub on_ground: bool,
    pub velocity: Vec3,
    pub is_moving: bool,
    
    // Added for UI
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_regen_rate: f32,
    pub stamina_use_rate: f32,
    pub exhausted: bool,       // Flag for when stamina is depleted
    pub exhaustion_timer: f32, // Time before stamina starts regenerating
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            turn_speed: 5.5,
            gravity: 20.0,
            jump_force: 7.0,
            ground_offset: 0.0, 
            on_ground: false,
            velocity: Vec3::ZERO,
            is_moving: false,
            
            // Stats for UI
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_regen_rate: 15.0, // Stamina gained per second when not using
            stamina_use_rate: 25.0,   // Stamina used per second when running
            exhausted: false,
            exhaustion_timer: 0.0,
        }
    }
}

// Player movement system
fn player_controller(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    camera_query: Query<(&Transform, &ThirdPersonCamera), Without<Player>>,
) {
    let dt = time.delta_secs();
    
    // Get camera transform for movement relative to camera view
    let camera_transform = if let Ok((cam_transform, _)) = camera_query.get_single() {
        Some(cam_transform)
    } else {
        None
    };
    
    for (mut player, mut transform) in player_query.iter_mut() {
        // Default to keep existing velocity but apply gravity
        let mut direction = Vec3::ZERO;
        player.velocity.y -= player.gravity * dt; // Apply gravity
        
        // Handle exhaustion recovery timer
        if player.exhausted {
            player.exhaustion_timer -= dt;
            if player.exhaustion_timer <= 0.0 {
                player.exhausted = false;
            }
        }
        
        // Handle stamina regeneration/depletion
        if player.is_moving {
            // Use stamina while moving
            player.stamina = (player.stamina - player.stamina_use_rate * dt).max(0.0);
            
            // Check if we've reached exhaustion
            if player.stamina <= 0.1 && !player.exhausted {
                player.exhausted = true;
                player.exhaustion_timer = 2.0; // 2 seconds of exhaustion
                // Could play a heavy breathing or fatigue sound here
            }
        } else if !player.exhausted {
            // Regenerate stamina when not moving and not exhausted
            player.stamina = (player.stamina + player.stamina_regen_rate * dt).min(player.max_stamina);
        } else if !player.is_moving {
            // Slower regeneration when exhausted but not moving
            player.stamina = (player.stamina + player.stamina_regen_rate * 0.3 * dt).min(player.max_stamina);
        }
        
        // Get movement direction based on camera orientation
        if let Some(camera) = camera_transform {
            // Get camera's forward and right, but project onto XZ plane (ignore Y-component)
            // This ensures WASD controls align with the camera's view direction
            let cam_forward = camera.forward();
            let cam_right = camera.right();
            
            // Project movement onto XZ plane by creating new vectors
            let forward = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
            let right = Vec3::new(cam_right.x, 0.0, cam_right.z).normalize_or_zero();
            
            // Calculate movement direction based on WASD keys
            if keyboard.pressed(KeyCode::KeyW) {
                direction += forward; // W moves forward relative to camera view
            }
            if keyboard.pressed(KeyCode::KeyS) {
                direction -= forward; // S moves backward relative to camera view
            }
            if keyboard.pressed(KeyCode::KeyA) {
                direction -= right; // A moves left relative to camera view
            }
            if keyboard.pressed(KeyCode::KeyD) {
                direction += right; // D moves right relative to camera view
            }
        } else {
            // Fallback to world-space movement if no camera is available
            if keyboard.pressed(KeyCode::KeyW) {
                direction += Vec3::NEG_Z;
            }
            if keyboard.pressed(KeyCode::KeyS) {
                direction += Vec3::Z;
            }
            if keyboard.pressed(KeyCode::KeyA) {
                direction += Vec3::NEG_X;
            }
            if keyboard.pressed(KeyCode::KeyD) {
                direction += Vec3::X;
            }
        }
        
        // Jump when on ground and space pressed - require minimum stamina
        if player.on_ground && keyboard.just_pressed(KeyCode::Space) && player.stamina >= 20.0 {
            player.velocity.y = player.jump_force;
            player.on_ground = false;
            
            // Jumping uses stamina
            player.stamina = (player.stamina - 20.0).max(0.0);
        }
        
        // Check if player is moving horizontally
        let is_moving = direction.length_squared() > 0.001;
        player.is_moving = is_moving;
        
        // Normalize horizontal movement if needed
        if is_moving {
            direction = direction.normalize();
        }
        
        // Apply movement with appropriate speed - more dramatic stamina effects
        let mut speed_modifier = 1.0;
        
        // Reduce speed based on stamina level
        if player.stamina < 20.0 {
            // Progressively slower as stamina depletes
            speed_modifier = 0.6 + (player.stamina / 20.0) * 0.4;
        }
        
        // When exhausted, severely limit movement speed
        if player.exhausted {
            speed_modifier = 0.3; // Very slow when exhausted
        }
        
        let target_velocity = direction * player.speed * speed_modifier;
        
        // Responsiveness is also affected by stamina
        let blend_factor = if player.exhausted {
            0.6 // More sluggish controls when exhausted
        } else if player.stamina < 30.0 {
            0.7 + (player.stamina / 30.0) * 0.2
        } else {
            0.8 // Normal responsiveness
        };
        
        // Smoothly blend horizontal velocity (XZ only) for more natural movement
        // Uses dynamic blend factor based on stamina
        player.velocity.x = player.velocity.x * blend_factor + target_velocity.x * (1.0 - blend_factor);
        player.velocity.z = player.velocity.z * blend_factor + target_velocity.z * (1.0 - blend_factor);
        
        // Apply velocity to position
        let mut displacement = player.velocity * dt;
        
        // Simple ground collision
        if player.velocity.y <= 0.0 && transform.translation.y + displacement.y <= player.ground_offset {
            player.velocity.y = 0.0;
            displacement.y = player.ground_offset - transform.translation.y;
            player.on_ground = true;
        } else {
            player.on_ground = false;
        }
        
        // Update position
        transform.translation += displacement;
        
        // Only rotate player if there's horizontal movement
        if is_moving {
            // Calculate the target rotation to face the movement direction
            let target_rotation = Quat::from_rotation_arc(
                Vec3::Z, 
                direction.normalize()
            );
            
            // Rotation speed is also affected by stamina
            let rotation_speed = if player.exhausted {
                player.turn_speed * 0.5
            } else {
                player.turn_speed
            };
            
            // Smoothly rotate towards the target rotation
            transform.rotation = transform.rotation.slerp(
                target_rotation, 
                rotation_speed * dt
            );
        }
    }
}

// Debug system for health regeneration over time
fn update_player_stats(
    mut player_query: Query<&mut Player>,
    time: Res<Time>,
) {
    for mut player in &mut player_query {
        // Very slow natural health regeneration - only when not exhausted
        if !player.exhausted && player.health < player.max_health {
            player.health = (player.health + 0.5 * time.delta_secs()).min(player.max_health);
        }
    }
}

// Plugin for player functionality
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (player_controller, update_player_stats));
        // Animation control is now handled in animation.rs
    }
}