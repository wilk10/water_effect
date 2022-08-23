use bevy::prelude::*;
use bevy::sprite::Mesh2dPipeline;
use bevy::render::render_resource::SpecializedMeshPipeline;
use bevy::sprite::Mesh2dPipelineKey;
use bevy::utils::Hashed;
use bevy::render::mesh::InnerMeshVertexBufferLayout;
use bevy::utils::FixedState;
use bevy::render::render_resource::*;
use bevy::render::render_phase::SetItemPipeline;
use bevy::sprite::SetMesh2dViewBindGroup;
use bevy::sprite::SetMesh2dBindGroup;
use bevy::sprite::DrawMesh2d;

pub struct WaterEffectPipeline {
    pub shader: Handle<Shader>,
    pub mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for WaterEffectPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/water_effect.wgsl");
        dbg!(&shader);
        Self {
            shader,
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}


impl SpecializedMeshPipeline for WaterEffectPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &Hashed<InnerMeshVertexBufferLayout, FixedState>,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut desc = self.mesh2d_pipeline.specialize(key, layout)?;

        // desc.layout = Some(vec![
        //     self.mesh2d_pipeline.view_layout.clone(),
        //     self.mesh2d_pipeline.mesh_layout.clone(),
        // ]);

        desc.vertex.shader = self.shader.clone();
        desc.fragment.as_mut().unwrap().shader = self.shader.clone();

        // desc.vertex = VertexState {
        //     shader: WATER_EFFECT_SHADER_HANDLE.typed::<Shader>(),
        //     shader_defs: vec![],
        //     entry_point: "vertex".into(),
        //     buffers: vec![],
        // };
        // desc.fragment = Some(FragmentState {
        //     shader: WATER_EFFECT_SHADER_HANDLE.typed::<Shader>(),
        //     shader_defs: vec![],
        //     entry_point: "fragment".into(),
        //     targets: vec![Some(ColorTargetState {
        //         format: TextureFormat::bevy_default(),
        //         blend: Some(BlendState::ALPHA_BLENDING),
        //         write_mask: ColorWrites::ALL,
        //     })],
        // });

        // desc.fragment = Some(FragmentState {
        //     shader: self.shader.clone(),
        //     shader_defs: vec![],
        //     entry_point: "fragment".into(),
        //     targets: vec![Some(ColorTargetState {
        //         format: TextureFormat::R8Unorm,
        //         blend: None,
        //         write_mask: ColorWrites::ALL,
        //     })],
        // });
        // desc.depth_stencil = None;

        desc.multisample = MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        // desc.label = Some("mesh_stencil_pipeline".into());
        Ok(desc)
    }
}

// This specifies how to render a colored 2d mesh
pub type DrawWaterEffect = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);
