use bevy::{
    prelude::*,
    window::PrimaryWindow,
    input::{
        mouse::{MouseMotion, MouseWheel},
        keyboard::KeyCode,
    },
    core_pipeline::bloom::Bloom,
    core_pipeline::prepass::DepthPrepass,
};
use crate::player::Player;
use crate::rendering::AdvancedRenderingSettings;

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    pub height_offset: f32,
    // Target offset for camera focus
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub smoothness: f32, // Camera lag factor (0 = instant, 1 = no movement)
    // Camera controls inversion flags
    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            pitch: 0.4,          // Initial pitch angle in radians
            yaw: 0.0,            // Initial yaw angle in radians
            distance: 5.0,       // Distance behind player
            height_offset: 1.5,  // Camera height above player
            rotation_speed: 0.005, // Mouse sensitivity
            zoom_speed: 0.5,     // Scroll zoom sensitivity
            smoothness: 0.85,    // Camera lag
            invert_x: false,     // Don't invert horizontal mouse
            invert_y: false,     // Don't invert vertical mouse
        }
    }
}

// Spawn camera system
fn spawn_camera(
    mut commands: Commands,
    advanced_settings: Res<AdvancedRenderingSettings>,
) {
    // Set up the camera with base components
    let camera_transform = Transform::from_xyz(0.0, 0.0, 0.0);
    
    // Add bloom effect for emissive materials
    commands.spawn((
        // Core camera components
        Camera3d::default(),
        camera_transform,
        
        // Bloom effect for emissive materials
        Bloom {
            intensity: advanced_settings.bloom_intensity,
            ..default()
        },
        
        // Add depth prepass for post-processing
        DepthPrepass,
        
        // Add third-person camera controller
        ThirdPersonCamera::default(),
    ));
}

// Third-person camera controller
fn third_person_camera(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, (With<Player>, Without<ThirdPersonCamera>)>,
    mut camera_query: Query<(&mut Transform, &mut ThirdPersonCamera)>,
    time: Res<Time>,
    mut exit: EventWriter<AppExit>,
) {
    // Handle ESC key to exit the game
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::default());
    }
    
    // Only update if we have a player and a camera
    if let (Ok(player_transform), Ok((mut camera_transform, mut camera_params))) = 
          (player_query.get_single(), camera_query.get_single_mut()) {
        
        // Handle mouse input for camera rotation
        let window = primary_window.single();
        let window_focused = window.focused;
        
        if window_focused {
            // Update camera rotation based on mouse movement
            for event in mouse_motion.read() {
                // Apply inversion if configured
                let dx = if camera_params.invert_x { -event.delta.x } else { event.delta.x };
                let dy = if camera_params.invert_y { -event.delta.y } else { event.delta.y };
                
                // Apply rotation speed
                camera_params.yaw -= dx * camera_params.rotation_speed;
                camera_params.pitch += dy * camera_params.rotation_speed;
                
                // Clamp pitch to prevent flipping (limit how far up/down the camera can look)
                camera_params.pitch = camera_params.pitch.clamp(-1.4, 0.8);
            }
            
            // Handle zoom with mouse wheel
            for event in mouse_wheel.read() {
                camera_params.distance -= event.y * camera_params.zoom_speed;
                // Clamp distance to reasonable values
                camera_params.distance = camera_params.distance.clamp(2.0, 15.0);
            }
        }
        
        // Get player position as the center point
        let player_pos = player_transform.translation;
        
        // Create rotation quaternions from euler angles
        let pitch_rot = Quat::from_rotation_x(camera_params.pitch);
        let yaw_rot = Quat::from_rotation_y(camera_params.yaw);
        let camera_rotation = yaw_rot * pitch_rot;
        
        // Calculate the orbital camera position
        let camera_offset = camera_rotation * Vec3::new(
            0.0,
            camera_params.height_offset,
            camera_params.distance // Positive distance is behind in orbital coordinates
        );
        
        // The camera should be positioned behind the player
        let target_position = player_pos - camera_offset;
        
        // Calculate the focus point (where the camera should look)
        // Look at the player position with a slight height offset but don't use target_offset
        let focus_pos = player_pos + Vec3::new(0.0, camera_params.height_offset * 0.5, 0.0);
        
        // Apply smoothing for camera movement (creates a more natural following effect)
        let smooth_factor = camera_params.smoothness.clamp(0.0, 0.99);
        let lerp_factor = 1.0 - smooth_factor.powf(time.delta_secs() * 60.0); // Frame-rate independent
        
        // Smoothly move camera toward target position
        camera_transform.translation = camera_transform.translation.lerp(
            target_position,
            lerp_factor
        );
        
        // Make camera look at the focus point
        camera_transform.look_at(focus_pos, Vec3::Y);
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, third_person_camera);
    }
}