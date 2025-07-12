use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AtlasMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 2760892297209218923, VertexFormat::Uint32);

impl Material for AtlasMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/atlas.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/atlas.wgsl".into()
    }
    fn specialize(
        _: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            ATTRIBUTE_TEXTURE_INDEX.at_shader_location(2),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = Vec::from([vertex_layout]);
        Ok(())
    }
}
