use crate::{
    gamemanager::WindowData, terrain_generation::TerrainSeersTag, touch_controls::TouchControls,
};
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;
pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerData {
    pub cursor_lock: bool,
    pub velocity: Vec3,
    pub speed: f32,
    pub acceleration: f32,

    pub surface_normal: Vec3,
}
#[derive(Component)]
pub struct PlayerTag;
#[derive(Component)]
pub struct PlayerRootTag;
#[derive(Component)]
pub struct PlayerCameraTag;

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            cursor_lock: true,
            velocity: Vec3::ZERO,
            speed: 10f32,
            acceleration: 1f32,
            surface_normal: Vec3::ZERO,
        }
    }
}
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, player_look, player_freefall));
    }
}
fn spawn_player(mut commands: Commands) {
    /* commands.spawn().insert(PlayerTag).insert(Transform::from_xyz(-5.0,2.0,-5.0))
    .with_children(|parent| {
        // child cube
        parent.spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 2.0)/* .looking_at(Vec3::ZERO, Vec3::Y) */,
            ..default()
        });
    }); */
    commands
        .spawn((
            PlayerTag,
            PlayerData {cursor_lock : true, ..Default::default()},
            PlayerRootTag,
            TerrainSeersTag,
            Transform::from_xyz(-10.0, 0.0, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCameraTag,
                Camera3d::default(),
                Msaa::Sample4,
                Transform::from_xyz(0.0, 2.0, 0.0), /* .looking_at(Vec3::ZERO, Vec3::Y) */
            ));
        });
}
fn player_freefall(
    mut query: Query<(&mut Transform, &mut PlayerData), With<PlayerRootTag>>,
    rapier_context: ReadRapierContext,
    time: Res<Time>,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };
    for (mut transform, mut player_data) in query.iter_mut() {
        let ray_pos = transform.translation
            + Vec3 {
                x: 0f32,
                y: 2f32,
                z: 0f32,
            };
        let ray_dir = Vec3::new(0.0, -1.0, 0.0);
        let max_toi = 10.0;
        let solid = true;

        if let Some((entity, ray)) =
            rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, QueryFilter::default())
        {
            // The first collider hit has the entity `entity` and it hit after
            // the ray travelled a distance equal to `ray_dir * toi`.
            let toi = ray.time_of_impact;
            player_data.surface_normal = ray.normal;
            // println!("Entity {:?} hit at point {}", entity, hit_point);

            if toi > 2.0 {
                transform.translation += ray_dir * time.delta_secs() * 5.0;
            } else if toi < 1.8 {
                transform.translation -= ray_dir * time.delta_secs() * 5.0;
            }
        }
    }
}
fn player_movement(
    mut player_query: Query<(Entity, &mut PlayerData), With<PlayerRootTag>>,
    mut transform_query: Query<&mut Transform>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touch_controls: Res<TouchControls>,
    time: Res<Time>,
) {
    for (player, mut player_data) in player_query.iter_mut() {
        let mut move_vec = Vec3::ZERO;
        if let Ok(transform) = transform_query.get(player) {
            if keyboard_input.pressed(KeyCode::KeyW) {
                let forw_vec = transform.right().cross(player_data.surface_normal);
                move_vec -= forw_vec;
            } else {
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                let forw_vec = transform.right().cross(player_data.surface_normal);
                move_vec += forw_vec;
            }
            if keyboard_input.pressed(KeyCode::KeyA) {
                let right_vec = transform.forward().cross(player_data.surface_normal);
                move_vec -= right_vec;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                let right_vec = transform.forward().cross(player_data.surface_normal);
                move_vec += right_vec;
            }
            let touch_move = touch_controls.movement;
            if touch_move != Vec2::ZERO {
                let forw_vec = transform.right().cross(player_data.surface_normal);
                let right_vec = transform.forward().cross(player_data.surface_normal);
                move_vec -= forw_vec * touch_move.y;
                move_vec += right_vec * touch_move.x;
            }
        }

        if !(keyboard_input.pressed(KeyCode::KeyW)
            && keyboard_input.pressed(KeyCode::KeyS)
            && keyboard_input.pressed(KeyCode::KeyA)
            && keyboard_input.pressed(KeyCode::KeyD))
        {
            let vel = player_data.velocity.clone();
            player_data.velocity -= vel * time.delta_secs() * 5f32;
        }
        if let Ok(mut transform) = transform_query.get_mut(player) {
            let mut speed = 0.2f32;
            if keyboard_input.pressed(KeyCode::ShiftLeft) {
                speed = 0.4f32;
                if player_data.velocity.length() <= 0.4f32 {
                    player_data.velocity += move_vec * time.delta_secs() * speed;
                }
            } else {
                if player_data.velocity.length() <= 0.2f32 {
                    player_data.velocity += move_vec * time.delta_secs() * speed;
                }
            }
            transform.translation += player_data.velocity.clone();
        }
    }
}
fn player_look(
    mut set: ParamSet<(
        Query<&mut Transform, With<PlayerCameraTag>>,
        Query<&mut Transform, With<PlayerRootTag>>,
    )>,
    window_data: Query<&WindowData>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    touch_controls: Res<TouchControls>,
    time: Res<Time>,
) {
    let Ok(window_data) = window_data.single() else {
        return;
    };
    let cursor_lock: bool = window_data.cursor_lock;
    if !cursor_lock {
        return;
    }

    let mut delta = Vec2::ZERO;
    delta += mouse_motion.delta * time.delta_secs() * 0.4;
    delta += touch_controls.look_delta * time.delta_secs() * 0.4;
    for mut transform in set.p0().iter_mut() {
        let pitch = Quat::from_rotation_x(-delta.y);
        transform.rotation = transform.rotation * pitch;
    }
    for mut transform in set.p1().iter_mut() {
        let yaw = Quat::from_rotation_y(-delta.x);
        transform.rotation = yaw * transform.rotation; // rotate around global y axis
    }
}
// fn player_debug(
//     query: Quey<GlobalTransform, With<PlayerCameraTag>>,
// ){
//     for global_transform in query.iter() {
//         //global_transform.com
//     }
// }
