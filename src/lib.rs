mod dual_contouring;
mod freecamera;
mod gamemanager;
mod ground;
mod house;
mod player;
mod skybox;
mod terrain_generation;
mod terrain_material;
mod touch_controls;
#[cfg(not(target_os = "android"))]
mod worldgen_ui;
use bevy::{
    asset::UnapprovedPathMode,
    prelude::*,
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::{CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection, WindowMode},
};
#[cfg(not(target_os = "android"))]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;
use gamemanager::GameManagerPlugin;
use gamemanager::WindowData;
use ground::GroundPlugin;
use house::HousePlugin;
use player::PlayerPlugin;
use terrain_generation::TerrainGenerationPlugin;
use touch_controls::TouchControlsPlugin;
#[cfg(not(target_os = "android"))]
use worldgen_ui::WorldGenUIPlugin;

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "kairos".to_string(),
                    resizable: false,
                    present_mode: PresentMode::Fifo,
                    // Android windows are always fullscreen; requesting exclusive
                    // fullscreen there fails because no video mode is reported.
                    mode: if cfg!(target_os = "android") {
                        WindowMode::Windowed
                    } else {
                        WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current)
                    },
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::from(WgpuSettings {
                    // Line polygon mode (wireframes) is not supported by mobile GPUs.
                    features: if cfg!(target_os = "android") {
                        WgpuFeatures::empty()
                    } else {
                        WgpuFeatures::POLYGON_MODE_LINE
                    },
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // The world gen tool loads its noise image from outside the assets folder.
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            })
            .set(ImagePlugin::default_linear()),
    )
    .insert_resource(ClearColor(Color::srgb(
        70.0 / 255.0,
        57.0 / 255.0,
        40.0 / 255.0,
    )))
    //.add_plugins(WireframePlugin::default())
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    //.add_plugins(RapierDebugRenderPlugin::default())
    .add_plugins((GameManagerPlugin, TouchControlsPlugin, GroundPlugin, HousePlugin))
    .add_systems(Startup, setup)
    .add_systems(Update, (regennoise, cursor_grab_system))
    .add_plugins(PlayerPlugin)
    // .add_plugins(freecamera::FreeCameraPlugin)
    .add_plugins(TerrainGenerationPlugin);
    // .add_plugins(skybox::SkyboxPlugin)
    #[cfg(not(target_os = "android"))]
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(WorldGenUIPlugin);
    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for za in 0..2 {
        for ya in 0..2 {
            for xa in 0..2 {
                // cube
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::from_length(1.0))),
                    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                    Transform::from_xyz((xa * 4) as f32, (ya * 4) as f32, (za * 4) as f32),
                ));
            }
        }
    }

    let mut sun_tranform = Transform::from_xyz(0.0, 0.0, 0.0);
    sun_tranform.rotate_x(4.7);
    // directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        sun_tranform,
    ));
    // light
    //commands.spawn((
    //    PointLight {
    //        intensity: 1500.0,
    //        shadow_maps_enabled: true,
    //        ..default()
    //    },
    //    Transform::from_xyz(0.0, 5.0, 0.0),
    //));
    #[cfg(not(target_os = "android"))]
    if let Ok(mut cursor) = cursor_options.single_mut() {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}
fn regennoise(key: Res<ButtonInput<KeyCode>>) {
    if key.just_pressed(KeyCode::KeyG) {
        // hot_lib::calc_noise();
    }
}
fn cursor_grab_system(
    mut windowdata_query: Query<&mut WindowData>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if cfg!(target_os = "android") {
        return;
    }
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };
    let Ok(mut windowsdata) = windowdata_query.single_mut() else {
        return;
    };

    if key.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;

        windowsdata.cursor_lock = true;
    }

    if key.just_pressed(KeyCode::KeyQ) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;

        windowsdata.cursor_lock = false;
    }
}
