use std::time::Duration;
use bevy::{
    prelude::*,
    input::keyboard::KeyCode,
    animation::{AnimationTargetId, RepeatAnimation},
};

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    println!("Spawning third-person game world with physics...");
    
    let ground_size = 50.0;


    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(CHARACTER_PATH)),
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations{
        animations: node_indices,
        graph: graph_handle,
    });

// ==============================================
// Setup player animation
// ==============================================
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    graphs: Res<Assets<AnimationGraph>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    fn get_clip<'a>(
        node: AnimationNodeIndex,
        graph: &AnimationGraph,
        clips: &'a mut Assets<AnimationClip>,
    ) -> &'a mut AnimationClip {
        let node = graph.get(node).unwrap();
        let clip = match &node.node_type {
            AnimationNodeType::Clip(handle) => clips.get_mut(handle),
            _ => unreachable!(),
        };
        clip.unwrap()
    }

    for (entity, mut player) in &mut players {
        let graph = graphs.get(&animations.graph).unwrap();

        let running_animation = get_clip(animations.animations[0], graph, &mut clips);
        //println!("Running animation: {:?}", running_animation);
        // You can determine the time an event should trigger if you know witch frame it occurs and
        // the frame rate of the animation. Let's say we want to trigger an event at frame 15,
        // and the animation has a frame rate of 24 fps, then time = 15 / 24 = 0.625.

        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}

// Plugin for player animation 
pub struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_scene_once_loaded);
    }
}