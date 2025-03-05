// src/world/environment/regions.rs
use bevy::prelude::*;
use bevy::pbr::{VolumetricFog, DistanceFog, Msaa};
use std::collections::HashMap;

// Region component that defines environment properties
#[derive(Component)]
pub struct WorldRegion {
    pub name: String,
    pub environment_settings: EnvironmentSettings,
    pub boundary: RegionBoundary,
    pub music_track: Option<Handle<AudioSource>>,
    pub ambient_sounds: Vec<AmbientSound>,
    pub site_of_grace: Option<Vec3>, // Bonfire/checkpoint location
    pub fog_gates: Vec<FogGate>,     // Transition areas
    pub weather_allowed: Vec<WeatherType>,
}

// Environment settings resource
#[derive(Resource, Clone)]
pub struct EnvironmentSettings {
    pub ambient_light: Color,            // Global ambient light
    pub skybox: Handle<Image>,           // Skybox texture
    pub fog_settings: FogSettings,       // Fog configuration
    pub ambient_occlusion: f32,          // AO intensity
    pub bloom_settings: BloomSettings,   // Bloom configuration
    pub wind_direction: Vec2,            // Wind direction and intensity
    pub wind_strength: f32,
    pub particle_systems: Vec<ParticleSettings>, // Ambient particles (dust, leaves, etc.)
}

// Fog settings
#[derive(Clone)]
pub struct FogSettings {
    pub color: Color,
    pub density: f32,
    pub falloff_start: f32,
    pub falloff_end: f32,
    pub height_falloff: f32,
}

// Bloom settings
#[derive(Clone)]
pub struct BloomSettings {
    pub intensity: f32,
    pub threshold: f32,
    pub blur_passes: u32,
}

// Ambient particle settings
#[derive(Clone)]
pub struct ParticleSettings {
    pub particle_type: ParticleType,
    pub density: f32,
    pub speed: f32,
    pub size_range: (f32, f32),
    pub color: Color,
}

// Particle types
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticleType {
    Dust,
    Leaves,
    Embers,
    Snow,
    Rain,
    Fog,
    Bugs,
}

// Region boundary definition
#[derive(Clone)]
pub enum RegionBoundary {
    Box(Vec3, Vec3),     // Min and max corners
    Sphere(Vec3, f32),   // Center and radius
    Mesh(Handle<Mesh>),  // Boundary mesh
}

// Ambient sound definition
#[derive(Clone)]
pub struct AmbientSound {
    pub sound: Handle<AudioSource>,
    pub volume: f32,
    pub spatial: bool,           // Is the sound positioned in 3D space?
    pub position: Option<Vec3>,  // If spatial, where is it located?
    pub radius: f32,             // How far the sound can be heard
}

// Fog gate (transition between areas)
#[derive(Clone)]
pub struct FogGate {
    pub position: Vec3,
    pub rotation: Quat,
    pub destination_region: String,
    pub destination_position: Vec3,
    pub locked: bool,
    pub unlock_item: Option<String>,
}

// Weather types
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rainy,
    Stormy,
    Foggy,
    Snowy,
}

// Current region resource
#[derive(Resource, Default)]
pub struct CurrentRegion {
    pub name: String,
    pub transition_progress: f32,
    pub previous_region: Option<String>,
}

// Region plugin
pub struct RegionPlugin;

impl Plugin for RegionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentRegion>()
           .add_systems(Update, (
               region_transition_system,
               apply_environment_settings,
               fog_gate_interaction_system,
           ));
    }
}

// System to detect player region changes
fn region_transition_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<crate::entities::player::Player>>,
    region_query: Query<(Entity, &WorldRegion)>,
    mut current_region: ResMut<CurrentRegion>,
    time: Res<Time>,
    mut audio_events: EventWriter<RegionChangeEvent>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;
        let mut found_region = false;
        
        for (entity, region) in &region_query {
            if is_in_region(player_pos, &region.boundary) {
                // Player has entered a new region
                if current_region.name != region.name {
                    // Start transition
                    current_region.previous_region = Some(current_region.name.clone());
                    current_region.name = region.name.clone();
                    current_region.transition_progress = 0.0;
                    
                    // Send region change event
                    audio_events.send(RegionChangeEvent {
                        region_name: region.name.clone(),
                        music_track: region.music_track.clone(),
                    });
                }
                
                found_region = true;
                break;
            }
        }
        
        // Advance transition progress
        if current_region.transition_progress < 1.0 {
            current_region.transition_progress += time.delta_seconds() * 0.5; // 2 second transition
            current_region.transition_progress = current_region.transition_progress.min(1.0);
        }
        
        if !found_region {
            // Player is not in any defined region
            // Could handle "wilderness" or "void" behavior here
        }
    }
}

// Helper function to check if a point is inside a region boundary
fn is_in_region(point: Vec3, boundary: &RegionBoundary) -> bool {
    match boundary {
        RegionBoundary::Box(min, max) => {
            point.x >= min.x && point.x <= max.x &&
            point.y >= min.y && point.y <= max.y &&
            point.z >= min.z && point.z <= max.z
        },
        RegionBoundary::Sphere(center, radius) => {
            point.distance(*center) <= *radius
        },
        RegionBoundary::Mesh(_) => {
            // Complex mesh boundary check would need a spatial query
            // This is a simplification
            false
        }
    }
}

// System to apply environment settings based on current region
fn apply_environment_settings(
    current_region: Res<CurrentRegion>,
    region_query: Query<&WorldRegion>,
    mut fog_query: Query<&mut DistanceFog>,
    mut bloom_query: Query<&mut bevy::core_pipeline::bloom::Bloom>,
    mut ambient_query: Query<&mut AmbientLight>,
) {
    // Only process if we're in a transition or just completed one
    if current_region.transition_progress < 1.0 {
        let current_settings = region_query
            .iter()
            .find(|r| r.name == current_region.name)
            .map(|r| &r.environment_settings);
            
        let previous_settings = if let Some(prev_name) = &current_region.previous_region {
            region_query
                .iter()
                .find(|r| r.name == *prev_name)
                .map(|r| &r.environment_settings)
        } else {
            None
        };
        
        if let Some(current) = current_settings {
            // Apply settings with transition
            let t = current_region.transition_progress;
            
            // Update fog settings
            if let Ok(mut fog) = fog_query.get_single_mut() {
                if let Some(prev) = &previous_settings {
                    // Interpolate between previous and current fog settings
                    fog.color = prev.fog_settings.color.lerp(current.fog_settings.color, t);
                    // Apply other fog settings by interpolating
                } else {
                    // Just apply current settings
                    fog.color = current.fog_settings.color;
                    // Apply other fog settings directly
                }
            }
            
            // Update bloom settings
            if let Ok(mut bloom) = bloom_query.get_single_mut() {
                if let Some(prev) = &previous_settings {
                    // Interpolate bloom intensity
                    bloom.intensity = (1.0 - t) * prev.bloom_settings.intensity + t * current.bloom_settings.intensity;
                } else {
                    bloom.intensity = current.bloom_settings.intensity;
                }
            }
            
            // Update ambient light
            if let Ok(mut ambient) = ambient_query.get_single_mut() {
                if let Some(prev) = &previous_settings {
                    ambient.color = prev.ambient_light.lerp(current.ambient_light, t);
                } else {
                    ambient.color = current.ambient_light;
                }
            }
            
            // Additional environment settings would be updated here
        }
    }
}

// System to handle fog gate interactions
fn fog_gate_interaction_system(
    player_query: Query<&Transform, With<crate::entities::player::Player>>,
    region_query: Query<&WorldRegion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut fog_gate_events: EventWriter<FogGateEvent>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;
        
        // Check proximity to fog gates in all regions
        for region in &region_query {
            for fog_gate in &region.fog_gates {
                let distance = player_pos.distance(fog_gate.position);
                
                // If player is close to a fog gate
                if distance < 2.0 {
                    // Check for interaction key
                    if keyboard.just_pressed(KeyCode::KeyE) {
                        // Check if gate is unlocked
                        if !fog_gate.locked {
                            // Trigger transition
                            fog_gate_events.send(FogGateEvent {
                                destination_region: fog_gate.destination_region.clone(),
                                destination_position: fog_gate.destination_position,
                            });
                            
                            // Teleport player (could also be handled by the event system)
                            // commands.entity(player_entity).insert(TeleportTag {
                            //     destination: fog_gate.destination_position,
                            // });
                        } else {
                            // Display "locked" message or play sound
                            // Could check inventory for unlock item here
                        }
                    }
                }
            }
        }
    }
}

// Events
#[derive(Event)]
pub struct RegionChangeEvent {
    pub region_name: String,
    pub music_track: Option<Handle<AudioSource>>,
}

#[derive(Event)]
pub struct FogGateEvent {
    pub destination_region: String,
    pub destination_position: Vec3,
}