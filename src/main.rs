use bevy::{
    prelude::*,
    pbr::experimental::meshlet::MeshletPlugin,
    window::{CursorGrabMode, CursorOptions, Window, WindowResolution},
};

mod player;
mod camera;
mod world;
mod lighting;
mod animation;
mod ui;
mod menu;
mod fx;
mod physics;
mod shader;

fn main() {
    println!("Starting Third-Person Example...");
    println!("Controls:");
    println!("  - ESC: Exit game");
    
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Use vsync for better performance 
                    //mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
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
            MeshletPlugin{
                cluster_buffer_slots: 8192,
            },
            physics::AvPhysicsPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            world::WorldPlugin,
            // menu::MenuPlugin,// This one doesnt work yet
            lighting::LightingPlugin,
            animation::PlayerAnimationPlugin,
            // fx::FXPlugin, // Disable til this works.
            ui::UIPlugin, 
            shader::ShaderPlugin, 
        ))
        // Set a dark sky color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.15)))
        .run();
}