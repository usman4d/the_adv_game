use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Places an entity on the terrain surface below (or above) it once the
/// terrain there has a collider, then removes itself.
pub struct GroundPlugin;

#[derive(Component)]
pub struct SnapToGround {
    /// Height above the ground hit point to place the entity at.
    pub height: f32,
}

/// Rays start this high so terrain above the entity is found too.
const RAY_START_HEIGHT: f32 = 500.0;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, snap_to_ground);
    }
}

fn snap_to_ground(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &SnapToGround)>,
    rapier_context: ReadRapierContext,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };
    for (entity, mut transform, snap) in query.iter_mut() {
        let origin = Vec3::new(transform.translation.x, RAY_START_HEIGHT, transform.translation.z);
        let filter = QueryFilter::default().exclude_collider(entity);
        if let Some((_, toi)) =
            rapier_context.cast_ray(origin, Vec3::NEG_Y, RAY_START_HEIGHT * 2.0, true, filter)
        {
            transform.translation.y = origin.y - toi + snap.height;
            commands.entity(entity).remove::<SnapToGround>();
        }
    }
}
