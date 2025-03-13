use avian3d::prelude::{Collider, ColliderConstructor, RigidBody};
use bevy::{
    gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady, 
    // pbr::FogVolume,
};

use serde::{Deserialize, Serialize};
use std::{
    f32::consts::{FRAC_PI_4, PI},
    time::Duration,
};
use crate::player::Player;

const CHARACTER_PATH: &str = "models/character.glb";

#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub collider: BCollider,
    pub rigid_body: BRigidBody,
    pub cube_size: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cubiod,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}

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
        RigidBody::Kinematic,
        Player::default(),
    )).with_children(|children|{
        children.spawn((Collider::capsule(0.4, 1.0), Transform::from_xyz(0.0, 1.0, 0.0)));
    });

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

fn on_level_spawn(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    for entity in
        children.iter_descendants(trigger.entity())
    {
        let Ok(gltf_mesh_extras) = extras.get(entity)
        else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<BMeshExtras>(
            &gltf_mesh_extras.value,
        ) else {
            error!("couldn't deseralize extras");
            continue;
        };
        dbg!(&data);
        match data.collider {
            BCollider::TrimeshFromMesh => {
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => {
                            RigidBody::Static
                        }
                        BRigidBody::Dynamic => {
                            RigidBody::Dynamic
                        }
                    },
                    ColliderConstructor::TrimeshFromMesh,
                ));
            }
            BCollider::Cubiod => {
                let size = data.cube_size.expect(
                    "Cubiod collider must have cube_size",
                );
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => {
                            RigidBody::Static
                        }
                        BRigidBody::Dynamic => {
                            RigidBody::Dynamic
                        }
                    },
                    Collider::cuboid(
                        size.x, size.y, size.z,
                    ),
                ));
            }
        }
    }
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
        "ESC: Exit game\n",
    )
    .into()
}

fn setup(mut commands: Commands){
    spawn_text(&mut commands);
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
            app.add_systems(Startup, (setup, spawn_scene));
    }
}