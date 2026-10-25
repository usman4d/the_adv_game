use bevy::prelude::*;

pub struct GameManagerPlugin;

impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
pub struct WindowData {
    pub cursor_lock: bool,
}

fn setup(mut commands: Commands) {
    commands.spawn(WindowData { cursor_lock: true });
}
