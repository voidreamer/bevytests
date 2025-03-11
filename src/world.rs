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
    
    // ==============================================
    // Create player character 
    // ==============================================
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CHARACTER_PATH))),
        Player::default(),
    ))
    .insert(RigidBody::KinematicPositionBased)
    .insert(Collider::capsule_y(1.0, 0.3))
    .insert(KinematicCharacterController{
        offset: CharacterLength::Absolute(0.01),
        ..default()
    })
    .insert(Velocity{
        linvel: Vec3::new(0.0, 0.3, 0.0),
        angvel: Vec3::new(0.2, 0.0, 0.0),
    });
   

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
        ))
        .insert(RigidBody::Dynamic)
        .insert(Velocity{
            linvel: Vec3::new(0.0, 0.3, 0.0),
            angvel: Vec3::new(0.2, 0.0, 0.0),
        })
        .insert(Collider::ball(1.0))
        .insert(GravityScale(2.0))
        .insert(Ccd::enabled())
        .insert(Restitution::coefficient(0.8));
    }

    // Testing some assets
    commands.spawn(SceneRoot(asset_server.load(
        "models/girly.glb#Scene0")));

    commands.spawn((
        SceneRoot(asset_server.load(
        "models/huge_icelandic_lava_cliff_sieoz_high.glb#Scene0")),
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));
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