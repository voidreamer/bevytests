use avian3d::prelude::*;
use bevy::{app::{App, FixedUpdate, Startup, Plugin}, ecs::system::Commands};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

pub fn setup(mut commands: Commands){
}

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((
            PhysicsPlugins::default(),
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_systems(Startup, setup);
    }
}

