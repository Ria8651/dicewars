use bevy::prelude::*;

mod board;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ui::UiPlugin)
        .add_plugin(board::BoardPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
