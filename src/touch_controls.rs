use bevy::{input::InputSystems, prelude::*, window::PrimaryWindow};

/// On-screen controls for touch devices: dragging on the left half of the
/// screen acts as a virtual joystick for movement, dragging on the right half
/// looks around.
pub struct TouchControlsPlugin;

/// Movement and look input produced by touches, consumed by the player systems.
#[derive(Resource, Default)]
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
            .add_systems(Startup, spawn_joystick)
            .add_systems(PreUpdate, update_touch_controls.after(InputSystems))
            .add_systems(Update, update_joystick_ui);
    }
}

fn spawn_joystick(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(JOYSTICK_RADIUS * 2.0),
            height: Val::Px(JOYSTICK_RADIUS * 2.0),
            display: Display::None,
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
        JoystickBase,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(KNOB_SIZE),
            height: Val::Px(KNOB_SIZE),
            display: Display::None,
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
        JoystickKnob,
    ));
}

fn update_touch_controls(
    touches: Res<Touches>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut state: Local<TouchState>,
    mut controls: ResMut<TouchControls>,
) {
    let Ok(width) = window.single().map(Window::width) else {
        return;
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
            // Touch positions have a top-left origin, so dragging up (-y) is forward.
            Some(touch) => {
                let offset = touch.position() - start;
                controls.movement =
                    (Vec2::new(offset.x, -offset.y) / JOYSTICK_RADIUS).clamp_length_max(1.0)
            }
            None => state.move_touch = None,
        }
    }
    controls.joystick_origin = state.move_touch.map(|(_, start)| start);
    if let Some(id) = state.look_touch {
        match touches.get_pressed(id) {
            Some(touch) => {
                controls.look_delta = touch.delta();
            }
            None => state.look_touch = None,
        }
    }
}

fn update_joystick_ui(
    controls: Res<TouchControls>,
    mut base_query: Query<&mut Node, (With<JoystickBase>, Without<JoystickKnob>)>,
    mut knob_query: Query<&mut Node, (With<JoystickKnob>, Without<JoystickBase>)>,
) {
    let display = if controls.joystick_origin.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    let origin = controls.joystick_origin.unwrap_or_default();
    let knob = origin + Vec2::new(controls.movement.x, -controls.movement.y) * JOYSTICK_RADIUS;

    for mut node in base_query.iter_mut() {
        node.display = display;
        node.left = Val::Px(origin.x - JOYSTICK_RADIUS);
        node.top = Val::Px(origin.y - JOYSTICK_RADIUS);
    }
    for mut node in knob_query.iter_mut() {
        node.display = display;
        node.left = Val::Px(knob.x - KNOB_SIZE / 2.0);
        node.top = Val::Px(knob.y - KNOB_SIZE / 2.0);
    }
}
