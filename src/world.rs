use bevy::{
    prelude::*,
    // pbr::FogVolume,
};
use bevy_rapier3d::prelude::*;
use crate::player::Player;

const CHARACTER_PATH: &str = "models/character.glb";

// Scene creation system with physics
pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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
        Mesh3d(_ground_mesh), 
        MeshMaterial3d(_ground_material), 
    ));
    
    // ==============================================
    // Create player character 
    // ==============================================
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CHARACTER_PATH))),
        Player::default(),
    ));
    
    // ==============================================
    // Create environment props
    // ==============================================

    /*
    commands.spawn(FogVolume{
        density_factor: 0.02,
        ..default()
    });
    */

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
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(pillar_material.clone()),
            Transform::from_xyz(x, 3.0, z),
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
            Mesh3d(meshes.add(Cuboid::from_length(2.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(x, 5.5, z),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Velocity{
            linvel: Vec3::new(0.0, 2.0, 0.0),
            angvel: Vec3::new(0.2, 0.0, 0.0),
        })
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(GravityScale(2.0))
        .insert(Ccd::enabled())
        .insert(Restitution::coefficient(0.2));
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
        ));
    }
}

// ==============================================
// Physics setup
// ==============================================

fn setup_physics(mut commands: Commands) {
    commands
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(Transform::from_xyz(0.0, -0.1, 0.0));

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Transform::from_xyz(0.0, 5.0, 0.0))
        .insert(Restitution::coefficient(0.7));
}


// ==============================================
// Some simple UI text 
// ==============================================

fn spawn_text(commands: &mut Commands){
    commands.spawn((
        create_help_text(),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }
    ));
}
fn create_help_text() -> Text {
    format!(
        "Lavid and Vlare adventures\n
WASD: Move player\n
Space: Jump\n
Mouse: Control camera\n
Mouse Wheel: Zoom in/out\n
Mouse Left Click: Attack!\n
Watch your stamina :P\n
ESC: Exit game\n",
    )
    .into()
}

fn setup(mut commands: Commands){
    spawn_text(&mut commands);
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Startup, spawn_scene)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())        
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, setup_physics);
    }
}