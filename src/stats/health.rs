use bevy::prelude::*;
use std::collections::HashMap;

// Basic damage types - make this public so we can use it in the player module
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DamageType {
    Physical,
    Fire,
    Magic,
    Lightning,
    Holy,
    Poison,
    Bleed,
}

// Health component with Elden Ring-like attributes
#[derive(Component, Default, Clone)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
    pub resistances: HashMap<DamageType, f32>, // Physical, Fire, Magic, etc.
    pub poise: f32,              // Stagger resistance
    pub poise_max: f32,
    pub poise_recovery_rate: f32,
    pub recovery_rate: f32,      // Health regeneration
}

impl Health {
    pub fn new(max_health: f32) -> Self {
        let mut resistances = HashMap::new();
        // Default resistances
        resistances.insert(DamageType::Physical, 0.0);
        resistances.insert(DamageType::Fire, 0.0);
        resistances.insert(DamageType::Magic, 0.0);
        resistances.insert(DamageType::Lightning, 0.0);
        resistances.insert(DamageType::Holy, 0.0);
        resistances.insert(DamageType::Poison, 0.0);
        resistances.insert(DamageType::Bleed, 0.0);
        
        Self {
            current: max_health,
            maximum: max_health,
            resistances,
            poise: 100.0,
            poise_max: 100.0,
            poise_recovery_rate: 10.0,
            recovery_rate: 0.0,  // No passive health regen by default
        }
    }
    
    // Take damage with type-based resistance
    pub fn take_damage(&mut self, amount: f32, damage_type: DamageType) -> f32 {
        let resistance = self.resistances.get(&damage_type).unwrap_or(&0.0);
        let damage_multiplier = 1.0 - resistance / 100.0;
        let actual_damage = amount * damage_multiplier;
        
        self.current -= actual_damage;
        if self.current < 0.0 {
            self.current = 0.0;
        }
        
        actual_damage
    }
    
    // Heal health
    pub fn heal(&mut self, amount: f32) {
        self.current += amount;
        if self.current > self.maximum {
            self.current = self.maximum;
        }
    }
    
    // Get health percentage
    pub fn get_percentage(&self) -> f32 {
        if self.maximum <= 0.0 {
            return 0.0;
        }
        (self.current / self.maximum).clamp(0.0, 1.0)
    }
    
    // Check if entity is dead
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

// Stamina component for actions
#[derive(Component, Default, Clone)]
pub struct Stamina {
    pub current: f32,
    pub maximum: f32,
    pub recovery_rate: f32,      // Stamina regeneration
    pub recovery_delay: Timer,   // Delay before stamina regenerates
}

impl Stamina {
    pub fn new(max_stamina: f32) -> Self {
        Self {
            current: max_stamina,
            maximum: max_stamina, 
            recovery_rate: 30.0,  // Stamina recovers quickly
            recovery_delay: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
    
    // Use stamina for an action
    pub fn use_stamina(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            self.recovery_delay.reset();
            true
        } else {
            false // Not enough stamina
        }
    }
    
    // Get stamina percentage
    pub fn get_percentage(&self) -> f32 {
        if self.maximum <= 0.0 {
            return 0.0;
        }
        (self.current / self.maximum).clamp(0.0, 1.0)
    }
}

// Event for when an entity takes damage
#[derive(Event)]
pub struct DamageEvent {
    pub entity: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}

// Event for when an entity dies
#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
}

// Health update system
fn update_health_system(
    time: Res<Time>,
    mut health_query: Query<&mut Health>,
) {
    for mut health in health_query.iter_mut() {
        // Natural health recovery (very slow in Souls games)
        if health.current < health.maximum && health.recovery_rate > 0.0 {
            health.current = (health.current + health.recovery_rate * time.delta().as_secs_f32())
                .min(health.maximum);
        }
        
        // Poise recovery
        if health.poise < health.poise_max {
            health.poise = (health.poise + health.poise_recovery_rate * time.delta().as_secs_f32())
                .min(health.poise_max);
        }
    }
}

// Stamina update system
fn update_stamina_system(
    time: Res<Time>,
    mut stamina_query: Query<&mut Stamina>,
) {
    for mut stamina in &mut stamina_query {
        // Check if delay has passed
        if stamina.recovery_delay.tick(time.delta()).finished() {
            // Regenerate stamina
            if stamina.current < stamina.maximum {
                stamina.current = (stamina.current + stamina.recovery_rate * time.delta().as_secs_f32())
                    .min(stamina.maximum);
            }
        }
    }
}

// System to apply damage from damage events
fn process_damage_system(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(event.entity) {
            // Apply damage with resistance
            health.take_damage(event.amount, event.damage_type);
            
            // Check for death
            if health.is_dead() {
                death_events.send(DeathEvent { 
                    entity: event.entity,
                });
            }
        }
    }
}

// Plugin for health systems
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_systems(Update, (
                update_health_system,
                update_stamina_system,
                process_damage_system,
            ));
    }
}