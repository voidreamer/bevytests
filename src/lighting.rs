use bevy::{
    prelude::*,
    pbr::CascadeShadowConfigBuilder,
    pbr::VolumetricLight,
};

#[derive(Component, Default)]
pub struct AnimatedLight {
    // We'll use this component to identify lights that should move
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

// Spawn lighting for the scene
fn spawn_lighting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Main directional light with cascaded shadow maps for sun
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        num_cascades: 4,
        first_cascade_far_bound: 5.0,
        maximum_distance: 30.0,
        ..default()
    }
    .build();
    
    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config,
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
    ));
    
    // Animated point light that will create dynamic reflections
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.5, 0.3),
            intensity: 5000.0,
            range: 15.0,
            shadows_enabled: true,
            ..default()
        },
        VolumetricLight,
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
        VolumetricLight,
        Transform::from_xyz(5.0, 2.0, 5.0),
        AnimatedLight::default(),
    ));

    commands.spawn(SceneRoot(asset_server.load(
        "models/girly.glb#Scene0")));


    commands.spawn(SceneRoot(asset_server.load(
        "models/scenario.glb#Scene0")));

}

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_lighting)
            .add_systems(Update, animate_lights);
    }
}