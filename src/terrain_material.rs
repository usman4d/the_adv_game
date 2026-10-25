use bevy::{
    pbr::Material,
    prelude::{Asset, Handle, Image, LinearRgba, TypePath},
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_tex_handle: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub normal_map_texture: Handle<Image>,
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wesl".into()
    }
}
