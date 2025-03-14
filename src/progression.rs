use bevy::prelude::*;
use crate::player::Player;
use crate::achievements::{AchievementEvent, MilestoneReward};

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerProgress>()
            .add_event::<CombatEvent>()
            .add_event::<StatAllocationEvent>()
            .add_systems(Startup, setup_progression)
            .add_systems(Update, (
                update_player_progress,
                handle_level_up,
                process_combat_events,
                sync_player_stats,
                process_stat_allocation,
            ));
    }
}

// Core player progression data
#[derive(Resource, Default)]
pub struct PlayerProgress {
    pub level: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,
    
    // Elden Ring style stats
    pub vigor: u32,      // Health
    pub mind: u32,       // FP/Mana
    pub endurance: u32,  // Stamina and equip load
    pub strength: u32,   // Physical attack power
    pub dexterity: u32,  // Attack speed and fall damage reduction
    pub intelligence: u32, // Magic potency
    pub faith: u32,      // Incantation potency
    pub arcane: u32,     // Discovery and certain spells
    
    pub available_stat_points: u32,
}

// World progression state
#[derive(Component)]
pub struct QuestState {
    pub id: String,
    pub completed: bool,
    pub progress: u32,
    pub max_progress: u32,
}

#[derive(Component)]
pub struct WorldArea {
    pub id: String,
    pub name: String,
    pub discovered: bool,
}

fn setup_progression() {
    info!("Setting up progression system");
}

fn update_player_progress(
    mut player_progress: ResMut<PlayerProgress>,
    // We'll integrate with actual gameplay systems later
) {
    // Experience threshold formula
    // Base 1000 XP for first level, then increases by 500 per level
    player_progress.experience_to_next_level = 1000 + (player_progress.level * 500);
}

#[derive(Event)]
pub struct CombatEvent {
    pub enemy_type: String,
    pub experience_reward: u32,
    pub is_boss: bool,
}

fn handle_level_up(
    mut player_progress: ResMut<PlayerProgress>,
    mut achievement_events: EventWriter<AchievementEvent>,
) {
    // Check if player has enough XP to level up
    if player_progress.experience >= player_progress.experience_to_next_level {
        // Level up
        player_progress.level += 1;
        player_progress.experience -= player_progress.experience_to_next_level;
        
        // Grant stat point
        player_progress.available_stat_points += 5;
        
        info!("Level up! Now level {}", player_progress.level);
        
        // Send achievement milestone update for level progression
        achievement_events.send(AchievementEvent {
            achievement_id: "player_level".to_string(),
            progress_amount: Some(1),
        });
        
        // Every 10 levels unlocks an achievement
        if player_progress.level % 10 == 0 {
            achievement_events.send(AchievementEvent {
                achievement_id: format!("level_{}", player_progress.level),
                progress_amount: None,
            });
        }
    }
}

// Process combat events and award XP
fn process_combat_events(
    mut combat_events: EventReader<CombatEvent>,
    mut player_progress: ResMut<PlayerProgress>,
    mut achievement_events: EventWriter<AchievementEvent>,
) {
    for event in combat_events.read() {
        // Award XP for defeating an enemy
        player_progress.experience += event.experience_reward;
        
        // Track enemy kills for milestone
        achievement_events.send(AchievementEvent {
            achievement_id: "enemy_slayer".to_string(),
            progress_amount: Some(1),
        });
        
        // If it's a boss, trigger boss achievement
        if event.is_boss {
            achievement_events.send(AchievementEvent {
                achievement_id: "first_boss".to_string(),
                progress_amount: None,
            });
        }
        
        info!(
            "Defeated {}! Gained {} XP. Total XP: {}/{}",
            event.enemy_type,
            event.experience_reward,
            player_progress.experience,
            player_progress.experience_to_next_level
        );
    }
}

// System to sync player stats with progression stats
fn sync_player_stats(
    player_progress: Res<PlayerProgress>,
    mut players: Query<&mut Player>,
) {
    // Only run if we have both a player and progress
    if let Ok(mut player) = players.get_single_mut() {
        // Calculate health bonus from vigor (5 health per point)
        let base_health = 100.0;
        let vigor_bonus = player_progress.vigor as f32 * 5.0;
        
        // Calculate stamina bonus from endurance (3 stamina per point)
        let base_stamina = 100.0;
        let endurance_bonus = player_progress.endurance as f32 * 3.0;
        
        // Update player's max stats
        player.max_health = base_health + vigor_bonus;
        player.max_stamina = base_stamina + endurance_bonus;
        
        // Ensure current values don't exceed max
        player.health = player.health.min(player.max_health);
        player.stamina = player.stamina.min(player.max_stamina);
    }
}

// For allocating stat points
#[derive(Event)]
pub struct StatAllocationEvent {
    pub stat_name: String,
    pub amount: u32,
}

// Handle stat point allocation
fn process_stat_allocation(
    mut events: EventReader<StatAllocationEvent>,
    mut player_progress: ResMut<PlayerProgress>,
) {
    for event in events.read() {
        // Check if player has enough points
        if player_progress.available_stat_points < event.amount {
            info!("Not enough stat points available!");
            continue;
        }
        
        // Allocate based on stat name
        let applied = match event.stat_name.as_str() {
            "vigor" => {
                player_progress.vigor += event.amount;
                true
            },
            "mind" => {
                player_progress.mind += event.amount;
                true
            },
            "endurance" => {
                player_progress.endurance += event.amount;
                true
            },
            "strength" => {
                player_progress.strength += event.amount;
                true
            },
            "dexterity" => {
                player_progress.dexterity += event.amount;
                true
            },
            "intelligence" => {
                player_progress.intelligence += event.amount;
                true
            },
            "faith" => {
                player_progress.faith += event.amount;
                true
            },
            "arcane" => {
                player_progress.arcane += event.amount;
                true
            },
            _ => {
                info!("Unknown stat: {}", event.stat_name);
                false
            }
        };
        
        // Deduct points if allocation was successful
        if applied {
            player_progress.available_stat_points -= event.amount;
            info!(
                "Allocated {} points to {}. New value: {}. Remaining points: {}", 
                event.amount,
                event.stat_name,
                match event.stat_name.as_str() {
                    "vigor" => player_progress.vigor,
                    "mind" => player_progress.mind,
                    "endurance" => player_progress.endurance,
                    "strength" => player_progress.strength,
                    "dexterity" => player_progress.dexterity,
                    "intelligence" => player_progress.intelligence,
                    "faith" => player_progress.faith,
                    "arcane" => player_progress.arcane,
                    _ => 0,
                },
                player_progress.available_stat_points
            );
        }
    }
}