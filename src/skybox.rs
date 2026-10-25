use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub struct SkyboxPlugin;
#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    image_handle: Handle<Image>,
}
impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, asset_loaded);
    }
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let skybox_handle = asset_server.load("textures/skybox/Ryfjallet_cubemap.png");

    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle,
    });
}
fn asset_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    if cubemap.is_loaded || !asset_server.is_loaded_with_dependencies(&cubemap.image_handle) {
        return;
    }
    let Some(mut image) = images.get_mut(&cubemap.image_handle) else {
        return;
    };
    // The cubemap is stored as six square faces stacked vertically.
    if image.texture_descriptor.array_layer_count() == 1 {
        let layers = image.height() / image.width();
        if image.reinterpret_stacked_2d_as_array(layers).is_err() {
            error!("skybox image is not a vertical stack of square faces");
            cubemap.is_loaded = true;
            return;
        }
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    for camera in cameras.iter() {
        commands.entity(camera).insert(Skybox {
            image: Some(cubemap.image_handle.clone()),
            brightness: 1000.0,
            ..default()
        });
    }
    cubemap.is_loaded = true;
}
