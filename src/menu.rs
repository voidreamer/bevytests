use bevy::{
      prelude::*,
      sprite::Anchor
};
use bevy_lunex::*;

fn spawn_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
){
      commands.spawn((
            UiLayoutRoot::new_2d(),
            UiFetchFromCamera::<0>,
      )).with_children(|ui|{
            ui.spawn((
                  Name::new("menu"),
                  UiLayout::window()
                        .anchor(Anchor::Center)
                        .pos(Rl(500.0))
                        .size((200.0, 50.0))    // Set the size to [200.0, 50.0]
                        .pack(),
                  // Color the sprite with red color
                  UiColor::from(Color::srgb(1.0, 0.0, 0.0)),

                  // Attach sprite to the node
                  Sprite::from_image(asset_server.load("images/button.png")),

                  // When hovered, it will request the cursor icon to be changed
                  OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),

            // Interactivity is done through observers, you can query anything here
            )).observe(|_: Trigger<Pointer<Click>>, mut exit: EventWriter<AppExit>| {
            
                  // Close the app on click
                  exit.send(AppExit::Success);
            });
      });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_menu);
    }
}