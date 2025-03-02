use bevy::{
    prelude::*,
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLightShadowMap,
    },
    core_pipeline::{
        bloom::Bloom,
        prepass::DepthPrepass,
    },
    window::{PrimaryWindow, CursorGrabMode, CursorOptions},
    input::{
        mouse::{MouseMotion, MouseWheel},
        keyboard::KeyCode,
    },
};

// Components


#[derive(Component)]
struct HighQualityObject;

#[derive(Component, Default)]
struct AnimatedLight {
    // We'll use this component to identify lights that should move
}

#[derive(Component)]
struct Player {
    speed: f32,
    turn_speed: f32,
    gravity: f32,
    jump_force: f32,
    ground_offset: f32,
    on_ground: bool,
    velocity: Vec3,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            turn_speed: 2.5,
            gravity: 20.0,
            jump_force: 8.0,
            ground_offset: 0.8, // Character height/2
            on_ground: false,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
struct ThirdPersonCamera {
    pitch: f32,
    yaw: f32,
    distance: f32,
    height_offset: f32,
    target_offset: f32,
    rotation_speed: f32,
    zoom_speed: f32,
    smoothness: f32, // Camera lag factor (0 = instant, 1 = no movement)
    collision_radius: f32,
    // Adding camera controls inversion flags
    invert_x: bool,
    invert_y: bool,
}

// Advanced Rendering Settings
#[derive(Resource)]
struct AdvancedRenderingSettings {
    bloom_intensity: f32,
    bloom_threshold: f32,
    ssao_radius: f32,
    ssao_intensity: f32,
    ssr_enabled: bool,
    taa_enabled: bool,
}

impl Default for AdvancedRenderingSettings {
    fn default() -> Self {
        Self {
            bloom_intensity: 0.15,
            bloom_threshold: 0.8,
            ssao_radius: 1.0,
            ssao_intensity: 0.5,
            ssr_enabled: true,
            taa_enabled: true,
        }
    }
}

// Setup advanced rendering resources
fn setup_render_resources(mut commands: Commands) {
    // Insert advanced rendering settings
    commands.insert_resource(AdvancedRenderingSettings::default());
}

// Systems
fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    advanced_settings: Res<AdvancedRenderingSettings>,
) {
    println!("Spawning third-person game world...");
    
    // Create a larger world
    let ground_size = 50.0;
    
    // ==============================================
    // Create ground plane
    // ==============================================
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.3, 0.2),
        metallic: 0.1,
        perceptual_roughness: 0.7,
        reflectance: 0.2,
        ..default()
    });
    
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(ground_size, ground_size));
    
    commands.spawn((
        MaterialMeshBundle {
            mesh: Mesh3d(ground_mesh),
            material: MeshMaterial3d(ground_material),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        HighQualityObject,
        DepthPrepass,
    ));
    
    // ==============================================
    // Create player character
    // ==============================================
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.2, 0.1),
        emissive: Color::srgb(0.14, 0.04, 0.02).into(), // Slight glow for effect
        metallic: 0.2,
        perceptual_roughness: 0.5,
        reflectance: 0.5,
        ..default()
    });
    
    // Player body (capsule would be better but we'll use a cylinder for simplicity)
    let player_body = meshes.add(Cylinder::new(0.5, 1.6));
    
    // Player entity
    let player = commands.spawn((
        MaterialMeshBundle {
            mesh: Mesh3d(player_body),
            material: MeshMaterial3d(player_material.clone()),
            transform: Transform::from_xyz(0.0, 0.8, 0.0),
            ..default()
        },
        Player::default(),
        HighQualityObject,
        DepthPrepass,
    )).id();
    
    // ==============================================
    // Third person camera
    // ==============================================
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
        ThirdPersonCamera {
            pitch: 0.4,          // Initial pitch angle in radians
            yaw: 0.0,            // Initial yaw angle in radians
            distance: 5.0,       // Distance behind player
            height_offset: 1.5,  // Camera height above player
            target_offset: 1.0,  // Look ahead offset
            rotation_speed: 0.005, // Mouse sensitivity (reduced for better control)
            zoom_speed: 0.5,     // Scroll zoom sensitivity
            smoothness: 0.85,    // Camera lag (0=instant, 1=max lag)
            collision_radius: 0.3, // Camera collision sphere size
            invert_x: false,     // Don't invert horizontal mouse
            invert_y: false,     // Don't invert vertical mouse
        },
    ));
    
    // ==============================================
    // Create environment props
    // ==============================================
    
    // Create some pillars to navigate around
    let pillar_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.8),
        metallic: 0.0,
        perceptual_roughness: 0.6,
        reflectance: 0.1,
        ..default()
    });
    
    let pillar_mesh = meshes.add(Cylinder::new(0.7, 6.0));
    
    // Create a circle of pillars
    for i in 0..12 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 12.0;
        let distance = 12.0;
        let x = angle.sin() * distance;
        let z = angle.cos() * distance;
        
        commands.spawn((
            MaterialMeshBundle {
                mesh: Mesh3d(pillar_mesh.clone()),
                material: MeshMaterial3d(pillar_material.clone()),
                transform: Transform::from_xyz(x, 3.0, z),
                ..default()
            },
            HighQualityObject,
            DepthPrepass,
        ));
    }
    
    // Create some smaller obstacles
    let obstacle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.4),
        metallic: 0.2,
        perceptual_roughness: 0.4,
        reflectance: 0.3,
        ..default()
    });
    
    let obstacle_mesh = meshes.add(Cuboid::new(1.5, 1.0, 1.5));
    
    // Scatter some cubes around
    for i in 0..20 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 20.0;
        let distance = 7.0 + (i as f32 * 0.3).sin() * 3.0;
        let x = angle.sin() * distance;
        let z = angle.cos() * distance;
        
        commands.spawn((
            MaterialMeshBundle {
                mesh: Mesh3d(obstacle_mesh.clone()),
                material: MeshMaterial3d(obstacle_material.clone()),
                transform: Transform::from_xyz(x, 0.5, z),
                ..default()
            },
            HighQualityObject,
            DepthPrepass,
        ));
    }
    
    // Add some decorative objects
    let sphere_mesh = meshes.add(Sphere::new(0.8));
    
    let chrome_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9),
        metallic: 1.0,
        perceptual_roughness: 0.1,  // Very polished 
        reflectance: 1.0,
        ..default()
    });
    
    // Add some reflective spheres
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 8.0;
        let distance = 15.0;
        let x = angle.sin() * distance;
        let z = angle.cos() * distance;
        
        commands.spawn((
            MaterialMeshBundle {
                mesh: Mesh3d(sphere_mesh.clone()),
                material: MeshMaterial3d(chrome_material.clone()),
                transform: Transform::from_xyz(x, 1.0, z),
                ..default()
            },
            HighQualityObject,
            DepthPrepass,
        ));
    }
    
    // ==============================================
    // Add lighting
    // ==============================================
    
    // Main directional light with cascaded shadow maps for sun
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 5.0,
        maximum_distance: 30.0,
        ..default()
    }
    .build();
    
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config,
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        ..default()
    });
    
    // Animated point light that will create dynamic reflections
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.5, 0.3),
            intensity: 5000.0,
            range: 15.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 0.0),
        AnimatedLight::default(),
    ));
    
    // Blue point light
    commands.spawn((
        PointLight {
            color: Color::srgb(0.1, 0.3, 1.0),
            intensity: 3000.0,
            range: 12.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 2.0, 5.0),
        AnimatedLight::default(),
    ));
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
        
        // Normalize horizontal movement if needed
        if direction.length_squared() > 0.001 {
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
        if direction.length_squared() > 0.001 {
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

// Animate lights for dynamic reflections
fn animate_lights(
    mut query: Query<(&mut Transform, Entity), With<AnimatedLight>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    
    // Animate each light with a unique pattern based on entity ID
    for (mut transform, entity) in &mut query {
        // Use entity ID to determine the pattern
        let is_first_light = entity.index() % 2 == 0;
        
        if is_first_light {
            // Warm light pattern (circular motion)
            transform.translation = Vec3::new(
                3.0 * f32::sin(t * 0.5),
                3.0 + 1.0 * f32::sin(t * 0.3),
                3.0 * f32::cos(t * 0.5),
            );
        } else {
            // Cool light pattern (figure eight)
            transform.translation = Vec3::new(
                -3.0 * f32::cos(t * 0.4),
                2.0 + 0.5 * f32::sin(t * 0.6),
                -3.0 * f32::sin(t * 0.4),
            );
        }
    }
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
                // Apply inversion if configured (note: we're using positive values here because
                // we've already adjusted the signs based on the expected behavior)
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

// Plugin for our third-person game
pub struct ThirdPersonGamePlugin;

impl Plugin for ThirdPersonGamePlugin {
    fn build(&self, app: &mut App) {
        // Enable high-quality shadows
        app.insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_systems(PreStartup, setup_render_resources)
            .add_systems(Startup, spawn_scene)
            // Add gameplay systems
            .add_systems(Update, player_controller)
            .add_systems(Update, third_person_camera)
            .add_systems(Update, animate_lights);
    }
}

fn main() {
    println!("Starting Third-Person Example...");
    println!("Controls:");
    println!("  - WASD: Move player");
    println!("  - Space: Jump");
    println!("  - Mouse: Control camera");
    println!("  - Mouse Wheel: Zoom in/out");
    println!("  - ESC: Exit game");
    
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Use vsync for better performance 
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    // Capture mouse for camera control
                    cursor_options: CursorOptions {
                        grab_mode: CursorGrabMode::Confined,
                        visible: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()))
        // Add our third-person game plugin
        .add_plugins(ThirdPersonGamePlugin)
        // Set a dark sky color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.15)))
        .run();
}