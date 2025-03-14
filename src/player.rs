use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::{
    input::keyboard::KeyCode, prelude::*
};
use bevy_tnua::{prelude::TnuaController, TnuaAnimatingState};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use crate::camera::ThirdPersonCamera;
use crate::animation::{
    PlayerAnimationState, 
    RootMotionAnimation, 
    AnimationStateMachine,
    AnimationCancellation
};

const CHARACTER_PATH: &str = "models/character.glb";

#[derive(Component)]
pub struct Player {
    pub is_moving: bool,
    pub is_attacking: bool,    // Flag for attack animation state
    
    // Added for UI
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_regen_rate: f32,
    pub stamina_use_rate: f32,
    pub exhausted: bool,       // Flag for when stamina is depleted
    pub exhaustion_timer: f32, // Time before stamina starts regenerating
}

impl Default for Player {
    fn default() -> Self {
        Self {
            is_moving: false,
            is_attacking: false,
            
            // Stats for UI
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_regen_rate: 30.0, // Stamina gained per second when not using
            stamina_use_rate: 15.0,   // Stamina used per second when running
            exhausted: false,
            exhaustion_timer: 0.0,
        }
    }
}

// Player movement system
fn player_controller(
    _keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    _player_query: Query<(&mut Player, &mut Transform)>,
    camera_query: Query<(&Transform, &ThirdPersonCamera), Without<Player>>,
) {
    let _dt = time.delta_secs();
    
    // Get camera transform for movement relative to camera view
    let _camera_transform = if let Ok((cam_transform, _)) = camera_query.get_single() {
        Some(cam_transform)
    } else {
        None
    };
}

// Debug system for health regeneration over time
fn update_player_stats(
    mut player_query: Query<&mut Player>,
    time: Res<Time>,
) {
    for mut player in &mut player_query {
        // Very slow natural health regeneration - only when not exhausted
        if !player.exhausted && player.health < player.max_health {
            player.health = (player.health + 0.5 * time.delta_secs()).min(player.max_health);
        }
    }
}

#[derive(Resource)]
pub struct PlayerGltfHandle(pub Handle<Gltf>);

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>
){
    // ==============================================
    // Create player character 
    // ==============================================
    commands.insert_resource(PlayerGltfHandle(asset_server.load(CHARACTER_PATH)));
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CHARACTER_PATH))),
        RigidBody::Dynamic,
        TnuaAnimatingState::<PlayerAnimationState>::default(),
        TnuaController::default(),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        Player::default(),
        RootMotionAnimation {
            enabled: true,
            previous_root_transform: None,
            motion_strength: 0.6, // Adjust strength of root motion (0.0 - 1.0)
        },
        AnimationStateMachine::new(), // Add our state machine
        AnimationCancellation::default(), // Add cancellation component
        Transform::from_xyz(0.0, 2.0, 0.0), // Initial position slightly above ground
    )).with_children(|children|{
        children.spawn((Collider::capsule(0.5, 1.0), Transform::from_xyz(0.0, 1.0, 0.0)));
    });
}

// Plugin for player functionality
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup_player)
        .add_systems(Update, (player_controller, update_player_stats));
        // Animation control is now handled in animation.rs
    }
}