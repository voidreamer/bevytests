use avian3d::prelude::{
      Collider,
      ColliderConstructor,
      RigidBody,
      PhysicsDebugPlugin,
      PhysicsPlugins,
      Gravity,
};
use bevy::{
      prelude::*,
      app::{App, FixedUpdate, Plugin, Startup},
      ecs::system::Commands, math::Vec3,
      gltf::GltfMeshExtras, scene::SceneInstanceReady, 
};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;
use serde::{Deserialize, Serialize};

/// Extras for physics
#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub collider: BCollider,
    pub rigid_body: BRigidBody,
    pub cube_size: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cubiod,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}

// System to add physics to the scene from gltf extras
pub fn on_level_spawn(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    for entity in
        children.iter_descendants(trigger.entity())
    {
        let Ok(gltf_mesh_extras) = extras.get(entity)
        else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<BMeshExtras>(
            &gltf_mesh_extras.value,
        ) else {
            error!("couldn't deseralize extras");
            continue;
        };
        dbg!(&data);
        match data.collider {
            BCollider::TrimeshFromMesh => {
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => {
                            RigidBody::Static
                        }
                        BRigidBody::Dynamic => {
                            RigidBody::Dynamic
                        }
                    },
                    ColliderConstructor::TrimeshFromMesh,
                ));
            }
            BCollider::Cubiod => {
                let size = data.cube_size.expect(
                    "Cubiod collider must have cube_size",
                );
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => {
                            RigidBody::Static
                        }
                        BRigidBody::Dynamic => {
                            RigidBody::Dynamic
                        }
                    },
                    Collider::cuboid(
                        size.x, size.y, size.z,
                    ),
                ));
            }
        }
    }
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
        .insert_resource(Gravity(Vec3::NEG_Y * 19.6));
    }
}

