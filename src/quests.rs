use bevy::prelude::*;
use std::collections::HashMap;
use crate::progression::{CombatEvent, PlayerProgress};
use crate::achievements::AchievementEvent;

pub struct QuestsPlugin;

impl Plugin for QuestsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestTracker>()
            .add_event::<QuestEvent>()
            .add_systems(Startup, setup_quests)
            .add_systems(Update, (
                process_quest_events,
                check_quest_completion,
                update_quest_objectives,
            ));
    }
}

#[derive(Resource, Default)]
pub struct QuestTracker {
    pub quests: HashMap<String, Quest>,
    pub active_quests: Vec<String>,
    pub completed_quests: Vec<String>,
}

pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub available: bool,
    pub active: bool,
    pub completed: bool,
    pub failed: bool,
    
    // Requirements to unlock
    pub level_requirement: Option<u32>,
    pub quest_prerequisites: Vec<String>,
}

pub struct QuestObjective {
    pub id: String,
    pub description: String,
    pub objective_type: QuestObjectiveType,
    pub completed: bool,
    pub failed: bool,
}

pub enum QuestObjectiveType {
    KillEnemies {
        enemy_type: String,
        count: u32,
        current: u32,
    },
    ExploreArea {
        area_id: String,
        discovered: bool,
    },
    CollectItem {
        item_id: String,
        count: u32,
        current: u32,
    },
    ReachLevel {
        level: u32,
    },
}

pub enum QuestReward {
    Experience(u32),
    Item(String),
    Stat(String, u32),
    Custom(String),
}

#[derive(Event)]
pub struct QuestEvent {
    pub quest_id: String,
    pub objective_id: String,
    pub event_type: QuestEventType,
}

pub enum QuestEventType {
    Progress { amount: u32 },
    Complete,
    Fail,
}

fn setup_quests(mut quest_tracker: ResMut<QuestTracker>) {
    info!("Setting up quest system");
    
    // Example quests
    let example_quests = vec![
        Quest {
            id: "tutorial".to_string(),
            name: "First Steps".to_string(),
            description: "Learn the basics of combat".to_string(),
            objectives: vec![
                QuestObjective {
                    id: "kill_tutorial_enemies".to_string(),
                    description: "Defeat 3 tutorial enemies".to_string(),
                    objective_type: QuestObjectiveType::KillEnemies {
                        enemy_type: "tutorial_enemy".to_string(),
                        count: 3,
                        current: 0,
                    },
                    completed: false,
                    failed: false,
                },
            ],
            rewards: vec![
                QuestReward::Experience(500),
                QuestReward::Item("Starter Weapon".to_string()),
            ],
            available: true,
            active: true,
            completed: false,
            failed: false,
            level_requirement: None,
            quest_prerequisites: vec![],
        },
        Quest {
            id: "first_boss".to_string(),
            name: "Trial by Fire".to_string(),
            description: "Defeat the first boss".to_string(),
            objectives: vec![
                QuestObjective {
                    id: "defeat_first_boss".to_string(),
                    description: "Defeat the tutorial boss".to_string(),
                    objective_type: QuestObjectiveType::KillEnemies {
                        enemy_type: "tutorial_boss".to_string(),
                        count: 1,
                        current: 0,
                    },
                    completed: false,
                    failed: false,
                },
            ],
            rewards: vec![
                QuestReward::Experience(1000),
                QuestReward::Item("Boss Weapon".to_string()),
            ],
            available: true,
            active: false,
            completed: false,
            failed: false,
            level_requirement: Some(3),
            quest_prerequisites: vec!["tutorial".to_string()],
        },
    ];
    
    // Add quests to tracker
    for quest in example_quests {
        // Auto-activate starter quest
        if quest.id == "tutorial" {
            quest_tracker.active_quests.push(quest.id.clone());
        }
        
        quest_tracker.quests.insert(quest.id.clone(), quest);
    }
}

fn process_quest_events(
    mut quest_events: EventReader<QuestEvent>,
    mut quest_tracker: ResMut<QuestTracker>,
) {
    for event in quest_events.read() {
        // Get the quest
        if let Some(quest) = quest_tracker.quests.get_mut(&event.quest_id) {
            // Find the specific objective
            for objective in &mut quest.objectives {
                if objective.id == event.objective_id {
                    match &event.event_type {
                        QuestEventType::Progress { amount } => {
                            // Update progress based on objective type
                            match &mut objective.objective_type {
                                QuestObjectiveType::KillEnemies { current, count, .. } => {
                                    *current += amount;
                                    info!(
                                        "Quest '{}', Objective '{}': {}/{}", 
                                        quest.name, objective.description, current, count
                                    );
                                }
                                QuestObjectiveType::CollectItem { current, count, .. } => {
                                    *current += amount;
                                    info!(
                                        "Quest '{}', Objective '{}': {}/{}", 
                                        quest.name, objective.description, current, count
                                    );
                                }
                                _ => {}
                            }
                        }
                        QuestEventType::Complete => {
                            objective.completed = true;
                            info!("Objective completed: {}", objective.description);
                        }
                        QuestEventType::Fail => {
                            objective.failed = true;
                            info!("Objective failed: {}", objective.description);
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn check_quest_completion(
    mut quest_tracker: ResMut<QuestTracker>,
    mut player_progress: ResMut<PlayerProgress>,
    mut achievement_events: EventWriter<AchievementEvent>,
) {
    let mut completed_quests = Vec::new();
    
    // Check active quests for completion
    for quest_id in &quest_tracker.active_quests {
        if let Some(quest) = quest_tracker.quests.get(quest_id) {
            // Check if all objectives are complete
            let all_complete = quest.objectives.iter().all(|obj| obj.completed);
            let any_failed = quest.objectives.iter().any(|obj| obj.failed);
            
            if all_complete {
                completed_quests.push(quest_id.clone());
                info!("Quest completed: {}", quest.name);
                
                // Award quest rewards when completed
                for reward in &quest.rewards {
                    match reward {
                        QuestReward::Experience(amount) => {
                            player_progress.experience += amount;
                            info!("Rewarded {} experience", amount);
                        }
                        QuestReward::Item(item) => {
                            info!("Rewarded item: {}", item);
                        }
                        QuestReward::Stat(stat, amount) => {
                            info!("Increased stat: {} by {}", stat, amount);
                            // Would apply stat bonuses here
                        }
                        QuestReward::Custom(desc) => {
                            info!("Custom reward: {}", desc);
                        }
                    }
                }
                
                // Send achievement for completed quest
                achievement_events.send(AchievementEvent {
                    achievement_id: "quest_completion".to_string(),
                    progress_amount: Some(1),
                });
                
                // If it's a boss quest, trigger boss achievement
                if quest_id == "first_boss" {
                    achievement_events.send(AchievementEvent {
                        achievement_id: "first_boss".to_string(),
                        progress_amount: None,
                    });
                }
            } else if any_failed {
                info!("Quest failed: {}", quest.name);
                // Handle failed quests
            }
        }
    }
    
    // Move completed quests to completed list
    for quest_id in &completed_quests {
        if let Some(quest) = quest_tracker.quests.get_mut(quest_id) {
            quest.completed = true;
            quest.active = false;
        }
        
        // Remove from active quests
        if let Some(index) = quest_tracker.active_quests.iter().position(|id| id == quest_id) {
            quest_tracker.active_quests.remove(index);
        }
        
        // Add to completed quests
        quest_tracker.completed_quests.push(quest_id.clone());
        
        // Check if any new quests are now available
        for (potential_quest_id, potential_quest) in &mut quest_tracker.quests {
            if !potential_quest.available && !potential_quest.active && !potential_quest.completed {
                // Check prerequisites
                let prereqs_met = potential_quest.quest_prerequisites.iter()
                    .all(|prereq_id| quest_tracker.completed_quests.contains(prereq_id));
                
                if prereqs_met {
                    potential_quest.available = true;
                    info!("New quest available: {}", potential_quest.name);
                }
            }
        }
    }
}

// Update quest objectives based on game events
fn update_quest_objectives(
    mut quest_tracker: ResMut<QuestTracker>,
    mut combat_events: EventReader<CombatEvent>,
    player_progress: Res<PlayerProgress>,
) {
    // Process combat events for kill objectives
    for event in combat_events.read() {
        for quest_id in &quest_tracker.active_quests {
            if let Some(quest) = quest_tracker.quests.get_mut(quest_id) {
                for objective in &mut quest.objectives {
                    if let QuestObjectiveType::KillEnemies { enemy_type, current, count } = &mut objective.objective_type {
                        if *enemy_type == event.enemy_type || enemy_type == "any" {
                            *current += 1;
                            
                            // Check if objective is now complete
                            if *current >= *count {
                                objective.completed = true;
                                info!("Quest objective completed: {}", objective.description);
                            } else {
                                info!(
                                    "Quest '{}' progress: {}/{}", 
                                    quest.name, current, count
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Check level-based objectives
    for quest_id in &quest_tracker.active_quests {
        if let Some(quest) = quest_tracker.quests.get_mut(quest_id) {
            for objective in &mut quest.objectives {
                if let QuestObjectiveType::ReachLevel { level } = &objective.objective_type {
                    if player_progress.level >= *level && !objective.completed {
                        objective.completed = true;
                        info!("Level objective completed: {}", objective.description);
                    }
                }
            }
        }
    }
}