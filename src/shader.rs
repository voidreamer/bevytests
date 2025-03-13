use bevy::{
      math::VectorSpace, prelude::*, reflect::TypePath, render::render_resource::{AsBindGroup, ShaderRef}
};

const AURORA_SHADER_PATH: &str = "shaders/aurora.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct AuroraShaderMaterial {}

impl Material for AuroraShaderMaterial{
    fn fragment_shader() -> ShaderRef {
        AURORA_SHADER_PATH.into()
    }
}

fn setup(
      mut commands: Commands,
      mut meshes: ResMut<Assets<Mesh>>,
      mut materials: ResMut<Assets<AuroraShaderMaterial>>,
){
      commands.spawn((
            Mesh3d(meshes.add(Sphere::new(500.0))),
            MeshMaterial3d(materials.add(AuroraShaderMaterial {})),
            Transform::from_xyz(500.0, 500.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
      ));
}

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
           .add_plugins(MaterialPlugin::<AuroraShaderMaterial>::default());
    }
}