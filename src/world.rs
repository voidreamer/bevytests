use bevy::{
    prelude::*,
    core_pipeline::prepass::DepthPrepass,
};
use avian3d::prelude::*; // Add Avian3D prelude for physics components
use crate::rendering::{AdvancedRenderingSettings, HighQualityObject};
use crate::player::Player;

// Scene creation system with physics
pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _advanced_settings: Res<AdvancedRenderingSettings>,
) {
    println!("Spawning third-person game world with physics...");
    
    let ground_size = 50.0;
    
    // ==============================================
    // Create ground plane (static)
    // ==============================================
    let _ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.3, 0.2),
        metallic: 0.1,
        perceptual_roughness: 0.7,
        reflectance: 0.2,
        ..default()
    });
    
    let _ground_mesh = meshes.add(Plane3d::default().mesh().size(ground_size, ground_size));
    
    commands.spawn((
        RigidBody::Static,           // Static rigid body (immovable)
        Collider::cuboid(ground_size, 0.1, ground_size), // Thin box collider
        Mesh3d(_ground_mesh), // Mesh for physics visualization
        MeshMaterial3d(_ground_material), 
        HighQualityObject,
        DepthPrepass,
    ));
    
    // ==============================================
    // Create player character (dynamic for now)
    // ==============================================
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.2, 0.1),
        emissive: Color::srgb(0.14, 0.04, 0.02).into(),
        metallic: 0.2,
        perceptual_roughness: 0.5,
        reflectance: 0.5,
        ..default()
    });
    
    let player_body = meshes.add(Cylinder::new(0.5, 1.6));
    
    // Player entity
    let _player = commands.spawn((
        RigidBody::Static,        
        Mesh3d(player_body),
        MeshMaterial3d(player_material.clone()),
        Transform::from_xyz(0.0, 0.8, 0.0),
        Player::default(),
        HighQualityObject,
        DepthPrepass,
    )).id();
    
    // ==============================================
    // Create environment props
    // ==============================================
    
    // Create some pillars (static)
    let pillar_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.8),
        metallic: 0.0,
        perceptual_roughness: 0.6,
        reflectance: 0.1,
        ..default()
    });
    
    let pillar_mesh = meshes.add(Cylinder::new(0.7, 6.0));
    
    for i in 0..12 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 12.0;
        let distance = 12.0;
        let x = angle.sin() * distance;
        let z = angle.cos() * distance;
        
        commands.spawn((
            RigidBody::Static,
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(pillar_material.clone()),
            Transform::from_xyz(x, 3.0, z),
            HighQualityObject,
            DepthPrepass,
        ));
    }
    
    // Create some smaller obstacles (dynamic cubes)
    let _obstacle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.4),
        metallic: 0.2,
        perceptual_roughness: 0.4,
        reflectance: 0.3,
        ..default()
    });
    
    for i in 0..20 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 20.0;
        let distance = 7.0 + (i as f32 * 0.3).sin() * 3.0;
        let x = angle.sin() * distance;
        let z = angle.cos() * distance;
        
        commands.spawn((
            RigidBody::Dynamic,        
            Collider::cuboid(1.5, 1.0, 1.5), 
            AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
            Mesh3d(meshes.add(Cuboid::from_length(1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(x, 0.5, z),
            HighQualityObject,
            DepthPrepass,
        ));
    }
    
    // Add some decorative objects (dynamic spheres)
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
            HighQualityObject,
            DepthPrepass,
        ));
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_scene);
    }
}