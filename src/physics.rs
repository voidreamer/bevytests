use avian3d::prelude::*;
use bevy::{app::{App, FixedUpdate, Plugin, Startup}, ecs::system::Commands, math::Vec3};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

pub fn setup(mut commands: Commands){
}

pub struct AvPhysicsPlugin;

impl Plugin for AvPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .insert_resource(Gravity(Vec3::NEG_Y * 19.6))
        .add_systems(Startup, setup);
    }
}

