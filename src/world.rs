use bevy::{
    gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady, text::cosmic_text::fontdb::Query
    // pbr::FogVolume,
};

use serde::{Deserialize, Serialize};
use std::{
    f32::consts::{FRAC_PI_4, PI},
    time::Duration,
};
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
    ));
   

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

    commands
        .spawn(SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0)
                    .from_asset("models/playground.glb"),
            ),
        ));
        //.observe(on_level_spawn);


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

/*
fn on_level_spawn(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
){
    for entity in
        children.iter_descendants(trigger.entity()
        {
            let Ok(value) = extras.get(entity) else {
                continue;
            };
            dbg!(value);
        }
    )
}
*/

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