use bevy::{
    core_pipeline::{bloom::Bloom, experimental::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing}, motion_blur::MotionBlur, tonemapping::Tonemapping, Skybox}, input::{
        keyboard::KeyCode, mouse::{MouseMotion, MouseWheel}
    }, pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel, VolumetricFog}, prelude::*, render::view::RenderLayers, window::PrimaryWindow
};
use bevy_lunex::UiSourceCamera;
use crate::player::Player;

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    pub height_offset: f32,
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub smoothness: f32, // Camera lag factor (0 = instant, 1 = no movement)
    // Camera controls inversion flags
    pub invert_x: bool,
    pub invert_y: bool,
    // Camera collision settings
    pub collision_radius: f32,     // Radius for collision detection
    pub min_distance: f32,         // Minimum distance from player
    pub max_distance: f32,         // Maximum distance from player
    pub collision_offset: f32,     // Offset from collision point
    pub vertical_offset: f32,      // Offset for camera vertical position when colliding
    pub current_actual_distance: f32, // Current actual distance after collision checks
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            pitch: 0.4,          // Initial pitch angle in radians
            yaw: 0.0,            // Initial yaw angle in radians
            distance: 5.0,       // Distance behind player
            height_offset: 1.5,  // Camera height above player
            rotation_speed: 0.004, // Mouse sensitivity
            zoom_speed: 0.5,     // Scroll zoom sensitivity
            smoothness: 0.85,    // Camera lag
            invert_x: false,     // Don't invert horizontal mouse
            invert_y: false,     // Don't invert vertical mouse
            // Camera collision settings
            collision_radius: 0.3,  // Radius for camera collision detection sphere
            min_distance: 1.0,      // Minimum distance from player
            max_distance: 15.0,     // Maximum distance from player
            collision_offset: 0.2,  // How much to offset camera from collision point
            vertical_offset: 0.5,   // Extra vertical offset when colliding
            current_actual_distance: 5.0, // Initialize to match distance
        }
    }
}

// Spawn camera system
fn spawn_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_xyz(0.0, 16.0, 40.0).looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        // This is for Lunex aparently, but doesnt work.
        UiSourceCamera::<0>,
        RenderLayers::from_layers(&[0, 2]),

        DistanceFog{
            color: Color::srgb_u8(43, 44, 100),
            falloff: FogFalloff::Exponential{
                density: 0.05,
            },
            ..default()
        },
        
        Bloom {
            intensity: 0.03,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        // Msaa is off to let ssao work.
        Msaa::Off,
        ScreenSpaceAmbientOcclusion::default(),
        TemporalAntiAliasing::default(),
        
        // Add depth prepass for post-processing
        MotionBlur{
            samples: 8,
            shutter_angle: 1.5,
            ..default()
        },
        VolumetricFog {
            ambient_intensity: 0.5,
            ..default()
        },

        EnvironmentMapLight{
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,
            ..default()
        },
        
        // Add third-person camera controller
        ThirdPersonCamera::default(),

    ))
    .insert(ScreenSpaceAmbientOcclusion{
        quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
        constant_object_thickness: 4.0,
    })
    .insert(Skybox{
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            brightness: 1000.0,
            ..default()
    });
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
                
                // Clamp pitch to prevent flipping and ground clipping
                // Negative pitch value = looking up, Positive pitch value = looking down
                // For Souls-like camera: 
                // - Limit looking down to prevent going through ground (-0.8 means can't look too far down)
                // - Limit looking up to reasonable angle (1.4 means can look pretty far up but not completely)
                camera_params.pitch = camera_params.pitch.clamp(-0.8, 1.4);
            }
            
            // Handle zoom with mouse wheel
            for event in mouse_wheel.read() {
                camera_params.distance -= event.y * camera_params.zoom_speed;
                // Clamp distance to reasonable values based on min/max in camera params
                camera_params.distance = camera_params.distance.clamp(camera_params.min_distance, camera_params.max_distance);
            }
        }
        
        // Get player position as the center point
        let player_pos = player_transform.translation;
        
        // Create rotation quaternions from euler angles
        let pitch_rot = Quat::from_rotation_x(camera_params.pitch);
        let yaw_rot = Quat::from_rotation_y(camera_params.yaw);
        let camera_rotation = yaw_rot * pitch_rot;
        
        // Camera pivot position (slightly above player's head)
        let camera_pivot = player_pos + Vec3::new(0.0, camera_params.height_offset, 0.0);
        
        // Calculate camera's direction and ideal position
        let camera_direction = camera_rotation * Vec3::new(0.0, 0.0, 1.0);
        let ideal_camera_pos = camera_pivot + camera_direction * camera_params.distance;
        
        // For proper third-person camera collision, we need to:
        // 1. Cast a ray from player (or slightly above) TO the camera's ideal position
        // 2. If this ray hits something, adjust the camera position to be in front of the hit
        
        // Setup for ray casting
        let ray_origin = camera_pivot; // Starting from player's head/pivot
        let ray_direction = (ideal_camera_pos - ray_origin).normalize(); // Direction TO camera
        let max_ray_distance = camera_params.distance;
        
        // For now, we'll simulate collisions based on camera direction
        // In a real implementation, we'd use Avian3D physics raycast here
        
        // Simulate collision based on camera angle (for demo purposes)
        // This is just for demonstration - in a real game, use the physics raycast
        let left_or_right_looking = camera_direction.x.abs() > 0.8; // Looking left/right
        
        // Simulate collision when looking sideways
        let collision_detected = left_or_right_looking;
        
        // Print collision status when F1 is pressed
        if keyboard.just_pressed(KeyCode::F1) {
            println!("===== COLLISION STATUS =====");
            println!("Collision detected: {}", collision_detected);
            println!("Camera direction: X={:.2}, Y={:.2}, Z={:.2}", 
                camera_direction.x, camera_direction.y, camera_direction.z);
            println!("Ray direction: X={:.2}, Y={:.2}, Z={:.2}",
                ray_direction.x, ray_direction.y, ray_direction.z);
            println!("Player position: {:.2}", player_pos);
            println!("Ideal camera position: {:.2}", ideal_camera_pos);
            println!("============================");
        }
        
        if collision_detected {
            // Apply Souls-like camera adjustments for our simulated collision
            
            // When camera collides, it will:
            // 1. Pull in closer to avoid clipping through walls
            // 2. Raise slightly to see over obstacles
            // 3. Peek around obstacles depending on collision side
            
            // Set a simulated hit distance (how far along the ray we "hit" something)
            let simulated_hit_distance = camera_params.distance * 0.3; // Hit 30% of the way to the ideal position
            
            // Calculate new distance based on the hit (minus a small offset)
            let new_distance = simulated_hit_distance - camera_params.collision_offset;
            
            // Set the actual camera distance 
            camera_params.current_actual_distance = new_distance.max(camera_params.min_distance);
            
            // Calculate how much we need to adjust (1.0 = full adjustment, 0.0 = no adjustment)
            let collision_progress = 1.0 - (new_distance / camera_params.distance);
            
            // Add vertical adjustment for looking over obstacles
            let vertical_adjustment = camera_params.vertical_offset * collision_progress;
            
            // For demo purposes, peek to the right when looking forward
            let wall_peek_direction = 1.0;
            
            // Calculate the peek amount based on collision severity
            let horizontal_peek = wall_peek_direction * collision_progress * 0.3;
            
            // Calculate how much we're looking down (for ground-clip prevention)
            let looking_down_factor = ((camera_params.pitch + 0.8) / 2.2).clamp(0.0, 1.0);
            let ground_clip_prevention = looking_down_factor * 1.5; // Up to 1.5 units extra height
            
            // Calculate adjusted camera position with both vertical adjustment and wall peeking
            let adjusted_camera_offset = camera_rotation * Vec3::new(
                horizontal_peek, // Add wall peek offset
                camera_params.height_offset + vertical_adjustment + ground_clip_prevention,
                camera_params.current_actual_distance // Use the collision-adjusted distance
            );
            
            // Position camera with collision adjustment
            let target_position = player_pos - adjusted_camera_offset;
            
            // Apply smoothing for camera movement
            let smooth_factor = camera_params.smoothness.clamp(0.0, 0.99);
            let lerp_factor = 1.0 - smooth_factor.powf(time.delta_secs() * 60.0);
            
            // Smoothly move camera toward collision-adjusted position
            camera_transform.translation = camera_transform.translation.lerp(
                target_position,
                lerp_factor
            );
        } else {
            // No collision, use full distance but reset gradually
            let lerp_speed = 2.0 * time.delta_secs(); // Adjust this value for faster/slower reset
            camera_params.current_actual_distance = camera_params.current_actual_distance.lerp(
                camera_params.distance,
                lerp_speed
            );
            
            // Calculate the orbital camera position with ground-clip prevention
            // When looking down, we need to raise the camera to avoid clipping through the ground
            
            // Calculate how much we're looking down (0 = looking straight, 1 = looking fully down)
            // This maps our pitch from -0.8 to 1.4 into a 0.0 to 1.0 "looking down" factor
            let looking_down_factor = ((camera_params.pitch + 0.8) / 2.2).clamp(0.0, 1.0);
            
            // Add extra height when looking down to prevent ground clipping
            let extra_height = looking_down_factor * 1.5; // Up to 1.5 units extra height
            
            // Apply the offset with ground-clip prevention
            let camera_offset = camera_rotation * Vec3::new(
                0.0,
                camera_params.height_offset + extra_height,
                camera_params.current_actual_distance
            );
            
            // The camera should be positioned behind the player
            let target_position = player_pos - camera_offset;
            
            // Apply smoothing for camera movement
            let smooth_factor = camera_params.smoothness.clamp(0.0, 0.99);
            let lerp_factor = 1.0 - smooth_factor.powf(time.delta_secs() * 60.0);
            
            // Smoothly move camera toward target position
            camera_transform.translation = camera_transform.translation.lerp(
                target_position,
                lerp_factor
            );
        }
        
        // Calculate the focus point (where the camera should look)
        let focus_pos = player_pos + Vec3::new(0.0, camera_params.height_offset * 0.5, 0.0);
        
        // Make camera look at the focus point
        camera_transform.look_at(focus_pos, Vec3::Y);
    }
}

#[derive(Resource)]
pub struct CameraDebugSettings {
    pub show_raycast: bool,
}

impl Default for CameraDebugSettings {
    fn default() -> Self {
        Self {
            show_raycast: true, // Enable by default so we can see it
        }
    }
}

// Debug system to visualize camera raycasts - useful for tuning collision
fn debug_camera_raycast(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<CameraDebugSettings>,
    mut gizmos: Gizmos,
    player_query: Query<&Transform, (With<Player>, Without<ThirdPersonCamera>)>,
    camera_query: Query<(&Transform, &ThirdPersonCamera)>,
    camera3d_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    // Toggle debug visualization with F1 key
    if keyboard.just_pressed(KeyCode::F1) {
        debug_settings.show_raycast = !debug_settings.show_raycast;
        println!("Camera debug visualization: {}", if debug_settings.show_raycast { "ON" } else { "OFF" });
        
        // Print explanation of the visualization elements when turning it on
        if debug_settings.show_raycast {
            println!("\nVISUALIZATION GUIDE:");
            println!("- RED SPHERE: Player head/pivot position (ray origin)");
            println!("- BLUE SPHERE: Ideal camera position (theoretical, no collisions)");
            println!("- GREEN/YELLOW SPHERES: ACTUAL camera position (engine camera)");
            println!("- ORANGE SPHERE: Calculated camera position (from our code)");
            println!("- WHITE LINE: Collision ray (from player to actual camera)");
            println!("- BLUE LINE: Path from player to ideal camera position");
            println!("- GREEN LINE: Path from player to ACTUAL engine camera");
            println!("- ORANGE LINE: Path from player to calculated camera position");
            println!("- RGB AXES: World coordinate system (Red=X, Green=Y, Blue=Z)");
            println!("\nIf GREEN and ORANGE spheres don't match, there's a discrepancy between\nwhere our code thinks the camera is and where it actually is.");
        }
    }
    
    if !debug_settings.show_raycast {
        return;
    }
    
    // Only debug if we have a player, camera params, and actual camera
    if let (Ok(player_transform), Ok((_, camera_params)), Ok(actual_camera_transform)) = 
          (player_query.get_single(), camera_query.get_single(), camera3d_query.get_single()) {
        
        // Get player position and actual camera position
        let player_pos = player_transform.translation;
        let actual_camera_pos = actual_camera_transform.translation; // This is the REAL camera position
        
        // Create rotation quaternions
        let pitch_rot = Quat::from_rotation_x(camera_params.pitch);
        let yaw_rot = Quat::from_rotation_y(camera_params.yaw);
        let camera_rotation = yaw_rot * pitch_rot;
        
        // Camera pivot and ray direction
        let camera_pivot = player_pos + Vec3::new(0.0, camera_params.height_offset, 0.0);
        let camera_direction = camera_rotation * Vec3::new(0.0, 0.0, 1.0);
        
        // For a third-person camera, we need to cast from player TO camera, not the other way around
        
        // Calculate ideal camera position (where it would be without collisions)
        let ideal_camera_pos = camera_pivot + camera_direction * camera_params.distance;
        
        // Draw main ray - from player to camera (THIS is the one that matters for collisions!)
        gizmos.line(
            camera_pivot,
            ideal_camera_pos,
            Color::srgb(1.0, 1.0, 0.0) // Yellow color
        );
        
        // Draw rays from multiple points on player to better visualize the collision detection
        let offset = 0.2;
        
        // Right ray
        gizmos.line(
            camera_pivot + Vec3::new(offset, 0.0, 0.0),
            ideal_camera_pos,
            Color::srgb(1.0, 0.0, 0.0) // Red color
        );
        
        // Left ray
        gizmos.line(
            camera_pivot - Vec3::new(offset, 0.0, 0.0),
            ideal_camera_pos,
            Color::srgb(1.0, 0.0, 0.0) // Red color
        );
        
        // Draw camera positions with more visible shapes
        
        // Calculate positions - note we're using the actual camera position now
        // For the "ideal" position, we still use the theoretical calculation
        let calculated_position = camera_pivot + camera_direction * camera_params.current_actual_distance;
        let ideal_position = camera_pivot + camera_direction * camera_params.distance;
        
        // Draw the calculated position (where our camera code thinks the camera is)
        gizmos.sphere(calculated_position, camera_params.collision_radius * 1.5, Color::srgb(1.0, 0.5, 0.0)); // Orange
        
        // ACTUAL CAMERA POSITION: This is where the camera REALLY is (from the game engine)
        // Represented by nested green/yellow spheres
        gizmos.sphere(actual_camera_pos, camera_params.collision_radius * 3.0, Color::srgb(0.0, 1.0, 0.0)); // Green outer
        gizmos.sphere(actual_camera_pos, camera_params.collision_radius * 2.5, Color::srgb(1.0, 1.0, 0.0)); // Yellow middle
        gizmos.sphere(actual_camera_pos, camera_params.collision_radius * 2.0, Color::srgb(0.0, 1.0, 0.0)); // Green inner
        
        // IDEAL CAMERA POSITION: This is where the camera would be if there were no collisions
        // Represented by a blue sphere
        gizmos.sphere(ideal_position, camera_params.collision_radius * 2.0, Color::srgba(0.0, 0.5, 1.0, 1.0)); // Blue
        
        // PLAYER HEAD/PIVOT: Draw a red sphere at the camera pivot (player's head position)
        // This is where rays are cast FROM for collision detection
        gizmos.sphere(camera_pivot, camera_params.collision_radius * 1.2, Color::srgb(1.0, 0.0, 0.0)); // Red
        
        // Draw labels next to each important point to explain what they represent
        
        // Draw connecting lines with multiple colors to make them more visible
        // GREEN LINE: Shows the path from player pivot to actual camera position (ENGINE CAMERA)
        gizmos.line(
            camera_pivot,
            actual_camera_pos,
            Color::srgb(0.0, 1.0, 0.0) // Green
        );
        
        // ORANGE LINE: Shows the path from player pivot to calculated camera position
        gizmos.line(
            camera_pivot,
            calculated_position,
            Color::srgb(1.0, 0.5, 0.0) // Orange
        );
        
        // Draw lines from pivot to ideal position
        // BLUE LINE: Shows the path from player pivot to ideal (non-colliding) camera position
        gizmos.line(
            camera_pivot,
            ideal_position,
            Color::srgb(0.0, 0.0, 1.0) // Blue
        );
        
        // Draw a thick white line showing ray from player TO actual camera (THIS is the collision ray)
        // WHITE LINE: This is the ray used for collision detection (player → camera)
        let ray_thickness = 3.0;
        for i in 0..3 {
            // Draw slightly offset lines to create a thicker effect
            let offset = Vec3::new(0.02 * i as f32, 0.02 * i as f32, 0.0);
            gizmos.line(
                camera_pivot + offset,
                actual_camera_pos + offset,
                Color::srgb(1.0, 1.0, 1.0) // White
            );
        }
        
        // Draw axes at the camera pivot to show orientation
        // These help visualize the world coordinate system
        let axis_length = 1.0; // Make them longer for visibility
        // X axis (red)
        gizmos.line(camera_pivot, camera_pivot + Vec3::new(axis_length, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0));
        // Y axis (green) - UP direction
        gizmos.line(camera_pivot, camera_pivot + Vec3::new(0.0, axis_length, 0.0), Color::srgb(0.0, 1.0, 0.0));
        // Z axis (blue) - FORWARD direction
        gizmos.line(camera_pivot, camera_pivot + Vec3::new(0.0, 0.0, axis_length), Color::srgb(0.0, 0.0, 1.0));
    }
}

pub struct CameraPlugin;

// Just use console/terminal for debugging information
fn display_camera_debug_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_params: Query<&ThirdPersonCamera>,
    debug_settings: Res<CameraDebugSettings>,
) {
    // Check for F1 key to print debug info once
    if keyboard.just_pressed(KeyCode::F1) && debug_settings.show_raycast {
        if let Ok(params) = camera_params.get_single() {
            // Calculate additional debug info
            let looking_down_factor = ((params.pitch + 0.8) / 2.2).clamp(0.0, 1.0);
            let extra_height = looking_down_factor * 1.5;
            
            println!("===== CAMERA DEBUG INFO =====");
            println!("Distance: {:.2}", params.distance);
            println!("Actual Distance: {:.2}", params.current_actual_distance);
            println!("Min Distance: {:.2}", params.min_distance);
            println!("Max Distance: {:.2}", params.max_distance);
            println!("Pitch: {:.2} (min=-0.8, max=1.4)", params.pitch);
            println!("Yaw: {:.2}", params.yaw);
            println!("Looking down factor: {:.2}", looking_down_factor);
            println!("Extra height for ground prevention: {:.2}", extra_height);
            println!("============================");
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDebugSettings>()
           .add_systems(Startup, spawn_camera)
           .add_plugins(TemporalAntiAliasPlugin)
           .add_systems(Update, (
               third_person_camera, 
               debug_camera_raycast,
               display_camera_debug_info,
           ));
    }
}