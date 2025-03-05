// src/entities/npc/enemy.rs
use bevy::prelude::*;
use crate::stats::health::Health;
use crate::stats::attributes::Attributes;
use crate::combat::weapons::types::EquippedWeapon;
use crate::ai::behavior_tree::{BehaviorTree, BehaviorNode};

// Enemy component
#[derive(Component)]
pub struct Enemy {
    pub name: String,
    pub enemy_type: EnemyType,
    pub aggro_range: f32,       // Distance to detect player
    pub attack_range: f32,      // Distance to start attacking
    pub leash_range: f32,       // Maximum distance from spawn before returning
    pub perception_state: PerceptionState,
    pub behavior_state: BehaviorState,
    pub spawn_position: Vec3,    // Original spawn location
    pub drops: Vec<LootDrop>,    // Items dropped on death
    pub runes: u32,              // Runes/souls dropped
    pub is_boss: bool,           // Is this a boss enemy?
}

// Enemy types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyType {
    Humanoid,
    Beast,
    Undead,
    Giant,
    Dragon,
    Spirit,
    Construct,
    // Add more as needed
}

// AI perception states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PerceptionState {
    Unaware,     // Not aware of player
    Suspicious,  // Noticed something, investigating
    Alerted,     // Aware of player, engaging
}

// AI behavior states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviorState {
    Idle,        // Standing around
    Patrol,      // Following patrol route
    Investigate, // Checking out noise/disturbance
    Chase,       // Pursuing player
    Attack,      // Attacking player
    Retreat,     // Falling back to heal or regroup
    Return,      // Returning to spawn
    Staggered,   // Temporarily stunned
}

// Potential drops
#[derive(Clone, Debug)]
pub struct LootDrop {
    pub item_id: String,
    pub chance: f32,  // 0.0-1.0 probability
    pub quantity_range: (u32, u32), // Min-max quantity
}

// Patrol path component
#[derive(Component)]
pub struct PatrolPath {
    pub points: Vec<Vec3>,
    pub current_point: usize,
    pub wait_time: Timer,
    pub bidirectional: bool,
    pub direction: i32, // 1 for forward, -1 for backward
}

// Boss-specific component
#[derive(Component)]
pub struct Boss {
    pub phase: u32,               // Current boss phase
    pub max_phases: u32,          // Total phases
    pub phase_transition_health: Vec<f32>, // Health thresholds for phase transitions
    pub music_track: Option<Handle<AudioSource>>,
    pub intro_cutscene: Option<String>,
    pub health_bar_name: String,  // Name to display on boss health bar
}

// Enemy plugin
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            enemy_perception_system,
            enemy_behavior_system,
            patrol_system,
            boss_phase_system,
        ));
    }
}

// AI perception system
fn enemy_perception_system(
    player_query: Query<&Transform, With<crate::entities::player::Player>>,
    mut enemy_query: Query<(&mut Enemy, &Transform, &Health)>,
    time: Res<Time>,
) {
    // If we can get the player transform
    if let Ok(player_transform) = player_query.get_single() {
        for (mut enemy, transform, health) in &mut enemy_query {
            // Skip dead enemies
            if health.current <= 0.0 {
                continue;
            }
            
            let distance_to_player = transform.translation.distance(player_transform.translation);
            
            // Update perception based on distance and visibility
            match enemy.perception_state {
                PerceptionState::Unaware => {
                    if distance_to_player < enemy.aggro_range {
                        // TODO: Add line-of-sight check
                        enemy.perception_state = PerceptionState::Alerted;
                    }
                },
                PerceptionState::Suspicious => {
                    if distance_to_player < enemy.aggro_range * 0.5 {
                        enemy.perception_state = PerceptionState::Alerted;
                    }
                    // Time-based logic to return to unaware state
                },
                PerceptionState::Alerted => {
                    if distance_to_player > enemy.leash_range {
                        enemy.perception_state = PerceptionState::Unaware;
                        enemy.behavior_state = BehaviorState::Return;
                    }
                }
            }
        }
    }
}

// Enemy behavior system that uses perception state to determine actions
fn enemy_behavior_system(
    player_query: Query<&Transform, With<crate::entities::player::Player>>,
    mut enemy_query: Query<(&mut Enemy, &mut Transform, &Health)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut enemy, mut transform, health) in &mut enemy_query {
            // Skip dead enemies
            if health.current <= 0.0 {
                continue;
            }
            
            let distance_to_player = transform.translation.distance(player_transform.translation);
            let distance_to_spawn = transform.translation.distance(enemy.spawn_position);
            
            // State machine for enemy behavior
            match enemy.behavior_state {
                BehaviorState::Idle => {
                    // Transition to other states based on perception
                    if enemy.perception_state == PerceptionState::Alerted {
                        enemy.behavior_state = BehaviorState::Chase;
                    }
                },
                BehaviorState::Patrol => {
                    // Patrol logic handled in patrol_system
                    if enemy.perception_state == PerceptionState::Alerted {
                        enemy.behavior_state = BehaviorState::Chase;
                    }
                },
                BehaviorState::Investigate => {
                    // Move toward suspicious location
                    if enemy.perception_state == PerceptionState::Alerted {
                        enemy.behavior_state = BehaviorState::Chase;
                    }
                },
                BehaviorState::Chase => {
                    if distance_to_player <= enemy.attack_range {
                        enemy.behavior_state = BehaviorState::Attack;
                    } else if distance_to_spawn > enemy.leash_range {
                        enemy.behavior_state = BehaviorState::Return;
                    } else {
                        // Move toward player
                        let direction = (player_transform.translation - transform.translation).normalize();
                        transform.translation += direction * 2.0 * time.delta_seconds();
                        
                        // Look at player
                        let look_at_player = Vec3::new(player_transform.translation.x, transform.translation.y, player_transform.translation.z);
                        transform.look_at(look_at_player, Vec3::Y);
                    }
                },
                BehaviorState::Attack => {
                    if distance_to_player > enemy.attack_range {
                        enemy.behavior_state = BehaviorState::Chase;
                    } else {
                        // Attack logic would go here
                        // This would trigger animations and weapon hitboxes
                    }
                },
                BehaviorState::Retreat => {
                    // Move away from player, possibly to heal
                    if health.current > health.maximum * 0.5 {
                        enemy.behavior_state = BehaviorState::Chase;
                    }
                },
                BehaviorState::Return => {
                    // Move back to spawn point
                    if distance_to_spawn < 1.0 {
                        enemy.behavior_state = BehaviorState::Idle;
                        enemy.perception_state = PerceptionState::Unaware;
                    } else {
                        let direction = (enemy.spawn_position - transform.translation).normalize();
                        transform.translation += direction * 3.0 * time.delta_seconds();
                    }
                },
                BehaviorState::Staggered => {
                    // Do nothing, animation system would handle this
                    // After stagger duration ends, return to previous state
                }
            }
        }
    }
}

// System to handle patrol paths
fn patrol_system(
    time: Res<Time>,
    mut query: Query<(&mut PatrolPath, &mut Transform, &mut Enemy)>,
) {
    for (mut path, mut transform, mut enemy) in &mut query {
        // Only process entities in patrol state
        if enemy.behavior_state != BehaviorState::Patrol {
            continue;
        }
        
        // Check if we're waiting at a point
        if !path.wait_time.tick(time.delta()).finished() {
            continue;
        }
        
        // Get current target point
        let target = path.points[path.current_point];
        let direction = (target - transform.translation).normalize_or_zero();
        
        // If we're close enough to the current point
        if transform.translation.distance(target) < 0.5 {
            // Reset wait timer
            path.wait_time.reset();
            
            // Move to next point
            if path.bidirectional {
                path.current_point = ((path.current_point as i32) + path.direction) as usize;
                
                // Check for reversing direction
                if path.current_point == 0 || path.current_point == path.points.len() - 1 {
                    path.direction *= -1;
                }
            } else {
                // Loop around for unidirectional paths
                path.current_point = (path.current_point + 1) % path.points.len();
            }
        } else {
            // Move toward the target point
            transform.translation += direction * 1.5 * time.delta_seconds();
            
            // Rotate to face movement direction
            let look_target = Vec3::new(target.x, transform.translation.y, target.z);
            transform.look_at(look_target, Vec3::Y);
        }
    }
}

// System to handle boss phase transitions
fn boss_phase_system(
    mut boss_query: Query<(&mut Boss, &Health)>,
    mut phase_events: EventWriter<BossPhaseChangeEvent>,
) {
    for (mut boss, health) in &mut boss_query {
        // Check if health threshold reached for phase transition
        let health_percent = health.current / health.maximum;
        
        for (phase_index, &threshold) in boss.phase_transition_health.iter().enumerate() {
            let next_phase = phase_index as u32 + 1;
            
            // If we cross a threshold and haven't already transitioned
            if health_percent <= threshold && boss.phase < next_phase {
                boss.phase = next_phase;
                
                // Send phase change event
                phase_events.send(BossPhaseChangeEvent {
                    phase: next_phase,
                    boss_health_percent: health_percent,
                });
                
                break;
            }
        }
    }
}

// Event for boss phase changes
#[derive(Event)]
pub struct BossPhaseChangeEvent {
    pub phase: u32,
    pub boss_health_percent: f32,
}