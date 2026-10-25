use crate::{gamemanager::WindowData, terrain_generation::TerrainSeersTag};
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

pub struct FreeCameraPlugin;
#[derive(Component)]
pub struct FreeCameraTag;
#[derive(Component)]
pub struct FreeCameraData;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (camera_look, camera_movement));
    }
}
fn setup(mut commands: Commands) {
    commands.spawn((
        FreeCameraTag,
        FreeCameraData,
        TerrainSeersTag,
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));
}
fn camera_movement(
    mut transform_query: Query<&mut Transform, With<FreeCameraTag>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in transform_query.iter_mut() {
        let mut move_vec = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            let forw_vec = transform.forward();
            move_vec += *forw_vec;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            let forw_vec = transform.forward();
            move_vec -= *forw_vec;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            let right_vec = transform.right();
            move_vec -= *right_vec;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            let right_vec = transform.right();
            move_vec += *right_vec;
        }
        transform.translation += move_vec * 30.0 * time.delta_secs();
    }
}
fn camera_look(
    mut cameratransform: Query<&mut Transform, With<FreeCameraTag>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    windowdata: Query<&WindowData>,
    time: Res<Time>,
) {
    let Ok(windowdata) = windowdata.single() else {
        return;
    };
    let cursorlock = windowdata.cursor_lock;

    if !cursorlock {
        return;
    }
    let mut delta = Vec2::ZERO;
    delta += mouse_motion.delta * time.delta_secs() * 0.4;
    for mut transform in cameratransform.iter_mut() {
        let pitch = Quat::from_rotation_x(-delta.y);
        transform.rotation = transform.rotation * pitch;

        let yaw = Quat::from_rotation_y(-delta.x);
        transform.rotation = yaw * transform.rotation; // rotate around global y axis
    }
}
