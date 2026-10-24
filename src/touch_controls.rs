use bevy::prelude::*;

/// On-screen controls for touch devices: dragging on the left half of the
/// screen acts as a virtual joystick for movement, dragging on the right half
/// looks around.
pub struct TouchControlsPlugin;

/// Movement and look input produced by touches, consumed by the player systems.
#[derive(Default)]
pub struct TouchControls {
    /// x = strafe right, y = forward; length is at most 1.
    pub movement: Vec2,
    /// Look delta in logical pixels, using the same convention as `MouseMotion`
    /// (+y = down).
    pub look_delta: Vec2,
    /// Where the active movement touch started, if any.
    pub joystick_origin: Option<Vec2>,
}

#[derive(Default)]
struct TouchState {
    move_touch: Option<(u64, Vec2)>,
    look_touch: Option<u64>,
}

#[derive(Component)]
struct JoystickBase;
#[derive(Component)]
struct JoystickKnob;

const JOYSTICK_RADIUS: f32 = 70.0;
const KNOB_SIZE: f32 = 50.0;

impl Plugin for TouchControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchControls>()
            .add_startup_system(spawn_joystick)
            .add_system_to_stage(CoreStage::PreUpdate, update_touch_controls)
            .add_system(update_joystick_ui);
    }
}

fn spawn_joystick(mut commands: Commands) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(
                    Val::Px(JOYSTICK_RADIUS * 2.0),
                    Val::Px(JOYSTICK_RADIUS * 2.0),
                ),
                display: Display::None,
                ..default()
            },
            color: Color::rgba(1.0, 1.0, 1.0, 0.15).into(),
            ..default()
        })
        .insert(JoystickBase);
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Px(KNOB_SIZE), Val::Px(KNOB_SIZE)),
                display: Display::None,
                ..default()
            },
            color: Color::rgba(1.0, 1.0, 1.0, 0.4).into(),
            ..default()
        })
        .insert(JoystickKnob);
}

fn update_touch_controls(
    touches: Res<Touches>,
    windows: Res<Windows>,
    mut state: Local<TouchState>,
    mut controls: ResMut<TouchControls>,
) {
    let width = match windows.get_primary() {
        Some(window) => window.width(),
        None => return,
    };

    for touch in touches.iter_just_pressed() {
        if touch.position().x < width / 2.0 {
            if state.move_touch.is_none() {
                state.move_touch = Some((touch.id(), touch.position()));
            }
        } else if state.look_touch.is_none() {
            state.look_touch = Some(touch.id());
        }
    }

    controls.movement = Vec2::ZERO;
    controls.look_delta = Vec2::ZERO;

    if let Some((id, start)) = state.move_touch {
        match touches.get_pressed(id) {
            // Touch positions have a bottom-left origin, so +y is forward.
            Some(touch) => {
                controls.movement =
                    ((touch.position() - start) / JOYSTICK_RADIUS).clamp_length_max(1.0)
            }
            None => state.move_touch = None,
        }
    }
    controls.joystick_origin = state.move_touch.map(|(_, start)| start);
    if let Some(id) = state.look_touch {
        match touches.get_pressed(id) {
            Some(touch) => {
                let delta = touch.delta();
                controls.look_delta = Vec2::new(delta.x, -delta.y);
            }
            None => state.look_touch = None,
        }
    }
}

fn update_joystick_ui(
    controls: Res<TouchControls>,
    mut base_query: Query<&mut Style, (With<JoystickBase>, Without<JoystickKnob>)>,
    mut knob_query: Query<&mut Style, (With<JoystickKnob>, Without<JoystickBase>)>,
) {
    let display = if controls.joystick_origin.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    let origin = controls.joystick_origin.unwrap_or_default();
    let knob = origin + controls.movement * JOYSTICK_RADIUS;

    for mut style in base_query.iter_mut() {
        style.display = display;
        style.position.left = Val::Px(origin.x - JOYSTICK_RADIUS);
        style.position.bottom = Val::Px(origin.y - JOYSTICK_RADIUS);
    }
    for mut style in knob_query.iter_mut() {
        style.display = display;
        style.position.left = Val::Px(knob.x - KNOB_SIZE / 2.0);
        style.position.bottom = Val::Px(knob.y - KNOB_SIZE / 2.0);
    }
}
