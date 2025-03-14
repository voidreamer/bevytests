use bevy::prelude::*;
use std::collections::HashMap;

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AchievementTracker>()
            .add_event::<AchievementEvent>()
            .add_systems(Startup, setup_achievements)
            .add_systems(Update, (
                process_achievement_events,
                update_milestone_progress,
            ));
    }
}

#[derive(Resource, Default)]
pub struct AchievementTracker {
    pub achievements: HashMap<String, Achievement>,
    pub milestones: HashMap<String, Milestone>,
}

pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub unlock_time: Option<f64>, // Time when unlocked
    pub icon: Option<String>,     // Path to icon
}

pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub current_progress: u32,
    pub required_progress: u32,
    pub completed: bool,
    pub rewards: Vec<MilestoneReward>,
}

pub enum MilestoneReward {
    Experience(u32),
    Item(String),
    Skill(String),
    CustomReward(String),
}

#[derive(Event)]
pub struct AchievementEvent {
    pub achievement_id: String,
    pub progress_amount: Option<u32>, // For milestones
}

fn setup_achievements(mut achievement_tracker: ResMut<AchievementTracker>) {
    info!("Setting up achievements system");
    
    // Add some example achievements
    let achievements = vec![
        Achievement {
            id: "first_boss".to_string(),
            name: "Challenger".to_string(),
            description: "Defeated your first boss".to_string(),
            unlocked: false,
            unlock_time: None,
            icon: None,
        },
        Achievement {
            id: "explorer".to_string(),
            name: "Explorer".to_string(),
            description: "Discovered 5 new areas".to_string(),
            unlocked: false,
            unlock_time: None,
            icon: None,
        },
        Achievement {
            id: "level_10".to_string(),
            name: "Apprentice".to_string(),
            description: "Reached player level 10".to_string(),
            unlocked: false,
            unlock_time: None,
            icon: None,
        },
        Achievement {
            id: "level_20".to_string(),
            name: "Adept".to_string(),
            description: "Reached player level 20".to_string(),
            unlocked: false,
            unlock_time: None,
            icon: None,
        },
        Achievement {
            id: "level_30".to_string(),
            name: "Expert".to_string(),
            description: "Reached player level 30".to_string(),
            unlocked: false,
            unlock_time: None,
            icon: None,
        },
    ];
    
    // Add some example milestones
    let milestones = vec![
        Milestone {
            id: "enemy_slayer".to_string(),
            name: "Enemy Slayer".to_string(),
            description: "Defeat 100 enemies".to_string(),
            current_progress: 0,
            required_progress: 100,
            completed: false,
            rewards: vec![
                MilestoneReward::Experience(500),
                MilestoneReward::Item("Special Weapon".to_string()),
            ],
        },
        Milestone {
            id: "dungeon_master".to_string(),
            name: "Dungeon Master".to_string(),
            description: "Complete 10 dungeons".to_string(),
            current_progress: 0,
            required_progress: 10,
            completed: false,
            rewards: vec![
                MilestoneReward::Experience(1000),
                MilestoneReward::Skill("Special Ability".to_string()),
            ],
        },
        Milestone {
            id: "player_level".to_string(),
            name: "Character Growth".to_string(),
            description: "Gain player levels".to_string(),
            current_progress: 1, // Start at level 1
            required_progress: 50, // Max level milestone
            completed: false,
            rewards: vec![
                MilestoneReward::Item("Legendary Weapon".to_string()),
            ],
        },
    ];
    
    // Add to tracker
    for achievement in achievements {
        achievement_tracker.achievements.insert(achievement.id.clone(), achievement);
    }
    
    for milestone in milestones {
        achievement_tracker.milestones.insert(milestone.id.clone(), milestone);
    }
}

fn process_achievement_events(
    mut event_reader: EventReader<AchievementEvent>,
    mut achievement_tracker: ResMut<AchievementTracker>,
    time: Res<Time>,
) {
    for event in event_reader.read() {
        // Handle direct achievements
        if let Some(achievement) = achievement_tracker.achievements.get_mut(&event.achievement_id) {
            if !achievement.unlocked {
                achievement.unlocked = true;
                achievement.unlock_time = Some(time.elapsed_secs().into());
                info!("Achievement unlocked: {}", achievement.name);
            }
        }
        
        // Handle milestone progress
        if let Some(milestone) = achievement_tracker.milestones.get_mut(&event.achievement_id) {
            if let Some(progress) = event.progress_amount {
                milestone.current_progress += progress;
                info!(
                    "Milestone progress: {} ({}/{})", 
                    milestone.name,
                    milestone.current_progress,
                    milestone.required_progress
                );
            }
        }
    }
}

fn update_milestone_progress(mut achievement_tracker: ResMut<AchievementTracker>) {
    // Check if any milestones are completed
    for (_, milestone) in achievement_tracker.milestones.iter_mut() {
        if !milestone.completed && milestone.current_progress >= milestone.required_progress {
            milestone.completed = true;
            info!("Milestone completed: {}", milestone.name);
            
            // Handle rewards
            for reward in &milestone.rewards {
                match reward {
                    MilestoneReward::Experience(amount) => {
                        info!("Rewarding {} experience", amount);
                        // We'll integrate with player progress later
                    }
                    MilestoneReward::Item(item) => {
                        info!("Rewarding item: {}", item);
                    }
                    MilestoneReward::Skill(skill) => {
                        info!("Unlocking skill: {}", skill);
                    }
                    MilestoneReward::CustomReward(desc) => {
                        info!("Custom reward: {}", desc);
                    }
                }
            }
        }
    }
}