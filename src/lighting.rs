use bevy::{
    prelude::*,
    pbr::CascadeShadowConfigBuilder
};


// Spawn lighting for the scene
fn spawn_lighting(
    mut commands: Commands,
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
            illuminance: 15000.0,
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


}

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_lighting);
    }
}