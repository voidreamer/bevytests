use bevy::{
    prelude::*,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    render::{
        render_resource::{
            AddressMode, FilterMode, SamplerDescriptor,
            TextureDescriptor, TextureDimension, TextureFormat, 
            TextureUsages, Extent3d,    
        },
        render_asset::RenderAssetUsages,
        renderer::RenderDevice,
        settings::{WgpuSettings, WgpuFeatures, RenderCreation},
        RenderApp, RenderSet,
    },
    core_pipeline::prepass::DepthPrepass,
};

// Components
#[derive(Component)]
pub struct HighQualityObject;

#[derive(Component)]
struct RayTracedObject;

//#[derive(AsRef, AsMult, Component, TypeUuid, Clone, Debug)]
pub struct GlobalIlluminationMaterial {
    albedo: Color,
    roughness: f32,
    metallic: f32,
    indirect_lighting_factor: f32,
}

impl Default for GlobalIlluminationMaterial {
    fn default() -> Self {
        Self {
            albedo: Color::srgb(0.8, 0.7, 0.6),
            roughness: 0.6,
            metallic: 0.0,
            indirect_lighting_factor: 0.5,
        }
    }
}

#[derive(Resource)]
pub struct DepthOfFieldSettings {
    enabled: bool,
    focus_length: f32,
    focal_plane_distance: f32,
    aperture: f32,
}

impl Default for DepthOfFieldSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            focus_length: 50.0,
            focal_plane_distance: 10.0,
            aperture: 2.8,
        }
    }
}

// Advanced Rendering Settings
#[derive(Resource)]
pub struct AdvancedRenderingSettings {
    // Bloom effect settings
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    
    // Screen Space Ambient Occlusion settings
    pub ssao_radius: f32,
    pub ssao_intensity: f32,
    
    // Additional rendering features
    pub ssr_enabled: bool,    // Screen Space Reflections
    pub taa_enabled: bool,    // Temporal Anti-Aliasing
}

impl Default for AdvancedRenderingSettings {
    fn default() -> Self {
        Self {
            bloom_intensity: 0.15,
            bloom_threshold: 0.8,
            ssao_radius: 1.0,
            ssao_intensity: 0.5,
            ssr_enabled: true,
            taa_enabled: true,
        }
    }
}

// Setup advanced rendering resources
fn setup_render_resources(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
    device: Res<RenderDevice>,
) {
    let window = windows.single();
    let width = window.physical_width();
    let height = window.physical_height();

    let gi_target = images.add(Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));

    let depth_target = images.add(Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    ));
    // Insert advanced rendering settings
    commands.insert_resource(AdvancedRenderingSettings::default());
    commands.insert_resource(DepthOfFieldSettings::default());
    
}

// Plugin for rendering features
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Enable high-quality shadows
        app.insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_systems(PreStartup, setup_render_resources);
    }
}