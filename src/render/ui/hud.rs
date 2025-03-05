use bevy::prelude::*;
use crate::stats::health::{Health, Stamina};
use crate::player::Player;

// HUD Root entity marker
#[derive(Component)]
struct PlayerHud;

// Health Bar component
#[derive(Component)]
struct HealthBar;

// Stamina Bar component
#[derive(Component)]
struct StaminaBar;

// Bar fill components
#[derive(Component)]
struct BarFill;

// Text components
#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct StaminaText;

// Style constants for the UI
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HEALTH_COLOR: Color = Color::srgb(0.8, 0.1, 0.1);
const STAMINA_COLOR: Color = Color::srgb(0.1, 0.8, 0.1);
const BAR_WIDTH: f32 = 300.0;
const BAR_HEIGHT: f32 = 30.0;
const BAR_BORDER: f32 = 2.0;

// Setup UI system
fn setup_hud(mut commands: Commands) {
    // Root UI container
    commands
        .spawn(NodeBundle::default())
        .insert(PlayerHud)
        .with_children(|parent| {
            // Health and stamina will go here
            parent.spawn(NodeBundle::default());
        });
}

// System to update HUD based on player stats
fn update_hud_system(
    player_query: Query<&Health, With<Player>>,
) {
    // Get player health
    if let Ok(_health) = player_query.get_single() {
        // Update health display (simplified for now)
    }
}

// Plugin for HUD components
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_hud)
            .add_systems(Update, update_hud_system);
    }
}