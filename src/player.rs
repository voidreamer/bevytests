use bevy::{
    prelude::*,
    input::keyboard::KeyCode,
};
use crate::camera::ThirdPersonCamera;
use std::time::Duration;

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
    pub current_animation: u8, // 0: idle, 1: tpose, 2: running
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            turn_speed: 5.5,
            gravity: 20.0,
            jump_force: 8.0,
            ground_offset: 0.0, // Character height/2
            on_ground: false,
            velocity: Vec3::ZERO,
            is_moving: false,
            current_animation: 0, // Start with idle animation
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
        
        // Jump when on ground and space pressed
        if player.on_ground && keyboard.just_pressed(KeyCode::Space) {
            player.velocity.y = player.jump_force;
            player.on_ground = false;
        }
        
        // Check if player is moving horizontally
        let is_moving = direction.length_squared() > 0.001;
        player.is_moving = is_moving;
        
        // Normalize horizontal movement if needed
        if is_moving {
            direction = direction.normalize();
        }
        
        // Apply movement with appropriate speed
        let target_velocity = direction * player.speed;
        
        // Smoothly blend horizontal velocity (XZ only) for more natural movement
        player.velocity.x = player.velocity.x * 0.8 + target_velocity.x * 0.2;
        player.velocity.z = player.velocity.z * 0.8 + target_velocity.z * 0.2;
        
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
            
            // Smoothly rotate towards the target rotation
            transform.rotation = transform.rotation.slerp(
                target_rotation, 
                player.turn_speed * dt
            );
        }
    }
}

// Note: Animation control is now handled directly in animation.rs module

// Plugin for player functionality
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_controller);
        // Animation control is now handled in animation.rs
    }
}