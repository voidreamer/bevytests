use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use bevy_lunex::Dimension;

use crate::physics::on_level_spawn;


// Scene creation system with physics
pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // ==============================================
    // Create the ground
    // ==============================================
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));
   

    // dynamic spheres
    let sphere_mesh = meshes.add(Sphere::new(0.8));
    
    let chrome_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9),
        metallic: 1.0,
        perceptual_roughness: 0.1,
        reflectance: 1.0,
        ..default()
    });
    
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 8.0;
        let distance = 15.0;
        let x = angle.sin() * distance;
        let z = angle.cos() * distance;
        
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(chrome_material.clone()),
            Transform::from_xyz(x, 1.0, z),
            RigidBody::Dynamic,
            Collider::sphere(0.8),
        ));
    }

    commands
        .spawn(SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0)
                    .from_asset("models/playground.glb"),
            ),
        ))
        .observe(on_level_spawn);


    /*
    // Testing some assets
    commands.spawn(SceneRoot(asset_server.load(
        "models/girly.glb#Scene0")));

    commands.spawn((
        SceneRoot(asset_server.load(
        "models/huge_icelandic_lava_cliff_sieoz_high.glb#Scene0")),
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));
    */
}


pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
            app
            // Set a dark sky color
            .insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.15)))
            .add_systems(Startup, spawn_scene);
    }
}