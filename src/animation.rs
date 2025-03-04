use std::time::Duration;
use bevy::{
    prelude::*,
    input::keyboard::KeyCode,
    animation::{AnimationTargetId, RepeatAnimation},
};

const CHARACTER_PATH: &str = "models/character.glb";

#[derive(Resource)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}

// ==============================================
// Setup player animation
// ==============================================
pub fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    println!("Setting up character animations...");

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
}

pub fn setup_scene_once_loaded(
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

pub fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        // Switch between animations (1, 2, 3 keys)
        if keyboard_input.just_pressed(KeyCode::Digit1) && *current_animation != 0 {
            *current_animation = 0;
            transitions
                .play(&mut player, animations.animations[0], Duration::from_secs_f32(0.5))
                .repeat();
        }
        
        if keyboard_input.just_pressed(KeyCode::Digit2) && *current_animation != 1 {
            *current_animation = 1;
            transitions
                .play(&mut player, animations.animations[1], Duration::from_secs_f32(0.5))
                .repeat();
        }
        
        if keyboard_input.just_pressed(KeyCode::Digit3) && *current_animation != 2 {
            *current_animation = 2;
            transitions
                .play(&mut player, animations.animations[2], Duration::from_secs_f32(0.5))
                .repeat();
        }

        // Pause/Resume animation
        if keyboard_input.just_pressed(KeyCode::KeyP) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            if playing_animation.is_paused() {
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }

        // Speed controls
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 0.8);
        }

        // Seek controls
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed + 0.1);
        }
    }
}

// Plugin for player animation 
pub struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_animations)
            .add_systems(Update, setup_scene_once_loaded)
            .add_systems(Update, keyboard_animation_control);
    }
}