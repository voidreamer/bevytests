/* 
//Disable til this works.

use std::f32::consts::PI;

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_hanabi::prelude::*;
use bevy_hanabi::*;


#[derive(Component)]
struct RotateSpeed(pub f32);

const COLOR: Vec4 = Vec4::new(0.7, 0.7, 1.0, 1.0);
const SIZE: Vec3 = Vec3::splat(0.1);

fn base_effect<M, F>(name: impl Into<String>, mut make_modifier: F) -> EffectAsset
where
    M: Modifier + Send + Sync + 'static,
    F: FnMut(&ExprWriter) -> M,
{
    let writer = ExprWriter::new();

    let init = make_modifier(&writer);

    EffectAsset::new(32768, Spawner::once(COUNT.into(), true), writer.finish())
        .with_name(name)
        .with_simulation_space(SimulationSpace::Local)
        .init(init)
        .render(OrientModifier::new(OrientMode::FaceCameraPosition))
        .render(SetColorModifier {
            color: COLOR.into(),
        })
        .render(SetSizeModifier { size: SIZE.into() })
}

fn spawn_effect(
    commands: &mut Commands,
    name: String,
    speed: f32,
    transform: Transform,
    effect: Handle<EffectAsset>,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands
        .spawn((
            transform,
            Name::new(format!("{}_parent", name)),
            Visibility::default(),
        ))
        .with_children(|p| {
            p.spawn((
                Name::new(name),
                ParticleEffect::new(effect),
                RotateSpeed(speed),
            ))
            .with_children(|p| {
                // Reference cube to visualize the emit origin
                p.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
            });
        });
}

const COUNT: f32 = 500_f32;

fn setup(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    commands.spawn(DirectionalLight {
        color: Color::WHITE,
        // Crank the illuminance way (too) high to make the reference cube clearly visible
        illuminance: 1000.,
        shadows_enabled: false,
        ..Default::default()
    });

    let cube = meshes.add(Cuboid {
        half_size: Vec3::splat(0.5),
    });
    let mat = materials.add(Color::srgb(0.0, 0.0, 1.0));

    spawn_effect(
        &mut commands,
        "SetPositionCircleModifier".to_string(),
        3.,
        Transform::from_translation(Vec3::new(-20., 0., 0.)),
        effects.add(base_effect("SetPositionCircleModifier", |writer| {
            SetPositionCircleModifier {
                center: writer.lit(Vec3::ZERO).expr(),
                axis: writer.lit(Vec3::Z).expr(),
                radius: writer.lit(5.).expr(),
                dimension: ShapeDimension::Volume,
            }
        })),
        cube.clone(),
        mat.clone(),
    );

    spawn_effect(
        &mut commands,
        "SetPositionSphereModifier".to_string(),
        3.,
        Transform::from_translation(Vec3::new(0., 0., 0.)),
        effects.add(base_effect("SetPositionSphereModifier", |writer| {
            SetPositionSphereModifier {
                center: writer.lit(Vec3::ZERO).expr(),
                radius: writer.lit(5.).expr(),
                dimension: ShapeDimension::Volume,
            }
        })),
        cube.clone(),
        mat.clone(),
    );

    spawn_effect(
        &mut commands,
        "SetPositionCone3dModifier".to_string(),
        3.,
        Transform::from_translation(Vec3::new(20., 0., 0.)),
        effects.add(base_effect("SetPositionCone3dModifier", |writer| {
            SetPositionCone3dModifier {
                height: writer.lit(10.).expr(),
                base_radius: writer.lit(1.).expr(),
                top_radius: writer.lit(4.).expr(),
                dimension: ShapeDimension::Volume,
            }
        })),
        cube.clone(),
        mat.clone(),
    );
}


 pub struct FXPlugin;

impl Plugin for FXPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(HanabiPlugin)
        .add_systems(Startup, setup);
    }
}
*/