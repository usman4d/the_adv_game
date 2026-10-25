use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};
#[hot_lib_reloader::hot_module(dylib = "hotlib")]
mod hot_lib {
    hot_functions_from_file!("hotlib/src/lib.rs");
}
pub struct WorldGenUIPlugin;

impl Plugin for WorldGenUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NoiseMapImage { ..default() })
            .add_systems(Startup, setup)
            .add_systems(Update, noiseimage_reload)
            .add_systems(EguiPrimaryContextPass, ui_example);
    }
}
#[derive(Resource, Default)]
struct NoiseMapImage {
    is_initialized: bool,
    image: Handle<Image>,
}
fn setup(asset_server: Res<AssetServer>, mut noisemap: ResMut<NoiseMapImage>) {
    hot_lib::calc_noise();
    noisemap.image = asset_server.load("../example_images/texture_wood_planar.png");
    noisemap.is_initialized = false;
}

fn noiseimage_reload(
    asset_server: Res<AssetServer>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::KeyG) {
        hot_lib::calc_noise();
        asset_server.reload("../example_images/texture_wood_planar.png");
    }
}
fn ui_example(
    mut contexts: EguiContexts,
    mut rendered_texture_id: Local<egui::TextureId>,
    mut noisemap: ResMut<NoiseMapImage>,
) -> Result {
    if !noisemap.is_initialized {
        noisemap.is_initialized = true;
        *rendered_texture_id = contexts.add_image(EguiTextureHandle::Weak(noisemap.image.id()));
    }

    egui::Window::new("WorldGenTool").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
            *rendered_texture_id,
            [256.0, 256.0],
        )));
        ui.label("Noisemap");
    });
    Ok(())
}
