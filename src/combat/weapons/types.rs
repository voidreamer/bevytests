// src/combat/weapons/types.rs
use bevy::prelude::*;
use std::collections::HashMap;
use crate::stats::attributes::AttributeType;
use crate::combat::damage::DamageType;
use crate::animation::controller::AnimationState;

// Weapon categories like in Elden Ring
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WeaponCategory {
    Dagger,
    StraightSword,
    Greatsword,
    UltraGreatsword,
    CurvedSword,
    Katana,
    Axe,
    Greataxe,
    Hammer,
    GreatHammer,
    Spear,
    Halberd,
    Reaper,
    Whip,
    Fist,
    Bow,
    Crossbow,
    Staff,
    SealTalisman,
    Shield,
    // Add more weapon types as needed
}

// Weapon scaling grades (S, A, B, C, D, E)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ScalingGrade {
    S,
    A,
    B,
    C,
    D,
    E,
    None,
}

impl ScalingGrade {
    pub fn to_multiplier(&self) -> f32 {
        match self {
            ScalingGrade::S => 1.0,
            ScalingGrade::A => 0.8,
            ScalingGrade::B => 0.6,
            ScalingGrade::C => 0.4,
            ScalingGrade::D => 0.25,
            ScalingGrade::E => 0.1,
            ScalingGrade::None => 0.0,
        }
    }
}

// Weapon component
#[derive(Component)]
pub struct Weapon {
    pub name: String,
    pub category: WeaponCategory,
    pub base_damage: HashMap<DamageType, f32>,
    pub scaling: HashMap<AttributeType, ScalingGrade>,
    pub weight: f32,
    pub durability: f32,
    pub max_durability: f32,
    pub skill: Option<WeaponSkill>,
    pub two_handed: bool,
    pub requirements: HashMap<AttributeType, u32>,
    pub attack_animations: HashMap<AttackType, String>,
    pub model: Handle<Scene>,
}

// Attack types
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttackType {
    LightAttack,
    HeavyAttack,
    ChargedAttack,
    JumpAttack,
    RunningAttack,
    SkillAttack,
    GuardCounterAttack,
}

// Weapon skill (Ashes of War equivalent)
#[derive(Clone, Debug)]
pub struct WeaponSkill {
    pub name: String,
    pub fp_cost: f32, // Mana cost
    pub stamina_cost: f32,
    pub animation_name: String,
    pub damage_multiplier: f32,
    pub skill_type: WeaponSkillType,
    pub effects: Vec<SkillEffect>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WeaponSkillType {
    Active,      // Directly activated skill
    Passive,     // Always-on effect
    Stance,      // Changes stance/moveset
    Buff,        // Temporary buff
    Projectile,  // Fires projectile
}

#[derive(Clone, Debug)]
pub enum SkillEffect {
    DamageBoost(f32),
    ElementalDamage(DamageType, f32),
    StatusEffect(StatusEffectType, f32),
    Stagger(f32),
    // Add more effects as needed
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StatusEffectType {
    Bleed,
    Poison,
    Frost,
    Sleep,
    Madness,
    DeathBlight,
    // Add more as needed
}

// Hitbox component for weapon attacks
#[derive(Component)]
pub struct WeaponHitbox {
    pub active: bool,
    pub attack_type: AttackType,
    pub size: Vec3,
    pub offset: Vec3,
}

// Equipped weapon component
#[derive(Component)]
pub struct EquippedWeapon {
    pub right_hand: Option<Entity>,
    pub left_hand: Option<Entity>,
    pub current_hand: HandSlot,
    pub is_two_handing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandSlot {
    Left,
    Right,
}