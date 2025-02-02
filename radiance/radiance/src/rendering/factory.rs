use std::rc::Rc;

use imgui::TextureId;

use super::{
    texture::TextureDef, Material, MaterialDef, RenderObject, RenderingComponent, Shader,
    ShaderProgram, Texture, VertexBuffer, VideoPlayer,
};

pub trait ComponentFactory {
    fn create_texture(&self, texture_def: &TextureDef) -> Box<dyn Texture>;
    fn create_imgui_texture(
        &self,
        buffer: &[u8],
        row_length: u32,
        width: u32,
        height: u32,
        texture_id: Option<TextureId>,
    ) -> (Box<dyn Texture>, TextureId);
    fn create_material(&self, material_def: &MaterialDef) -> Box<dyn Material>;
    fn create_render_object(
        &self,
        vertices: VertexBuffer,
        indices: Vec<u32>,
        material_def: &MaterialDef,
        host_dynamic: bool,
    ) -> Box<dyn RenderObject>;
    fn create_rendering_component(&self, objects: Vec<Box<dyn RenderObject>>)
        -> RenderingComponent;
    fn create_video_player(&self) -> Box<VideoPlayer>;
}
