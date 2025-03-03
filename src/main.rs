use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, Window, WindowResolution},
};
use avian3d::prelude::*;

mod rendering;
mod player;
mod camera;
mod world;
mod lighting;

fn main() {
    println!("Starting Third-Person Example...");
    println!("Controls:");
    println!("  - WASD: Move player");
    println!("  - Space: Jump");
    println!("  - Mouse: Control camera");
    println!("  - Mouse Wheel: Zoom in/out");
    println!("  - ESC: Exit game");
    
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Use vsync for better performance 
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    resolution: WindowResolution::new(1920., 1080.).with_scale_factor_override(1.0),
                    // Capture mouse for camera control
                    cursor_options: CursorOptions {
                        grab_mode: CursorGrabMode::Confined,
                        visible: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()))
        // Add our third-person game plugin
        .add_plugins((
            player::PlayerPlugin,
            camera::CameraPlugin,
            world::WorldPlugin,
            lighting::LightingPlugin,
            PhysicsPlugins::default()
        ))
        // Set a dark sky color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.15)))
        .run();
}