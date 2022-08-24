use bevy::sprite::Mesh2dPipeline;
use bevy::sprite::Mesh2dPipelineKey;
use bevy::{
    // pbr::{MeshPipeline, MeshPipelineKey},
    prelude::*,
    render::{
        mesh::InnerMeshVertexBufferLayout,
        render_graph::{Node, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{DrawFunctions, PhaseItem, RenderPhase, TrackedRenderPass},
        render_resource::*,
        renderer::RenderContext,
    },
    utils::{FixedState, Hashed},
};
use bevy::render::render_phase::DrawFunctionId;
use bevy::utils::FloatOrd;
use bevy::render::render_phase::EntityPhaseItem;
use bevy::render::render_phase::CachedRenderPipelinePhaseItem;
use bevy::render::render_phase::SetItemPipeline;
use bevy::sprite::SetMesh2dViewBindGroup;
use bevy::sprite::SetMesh2dBindGroup;
use bevy::sprite::DrawMesh2d;

use crate::{resources::WaterEffectResources};

#[derive(Debug)]
pub struct WaterMask {
    pub distance: f32,
    pub pipeline: CachedRenderPipelineId,
    pub entity: Entity,
    pub draw_function: DrawFunctionId,
}

impl PhaseItem for WaterMask {
    type SortKey = FloatOrd;

    fn sort_key(&self) -> Self::SortKey {
        dbg!(self.distance);
        FloatOrd(self.distance)
    }

    fn draw_function(&self) -> DrawFunctionId {
        dbg!(&self.draw_function);
        self.draw_function
    }
}

impl EntityPhaseItem for WaterMask {
    fn entity(&self) -> Entity {
        dbg!(&self.entity);
        self.entity
    }
}

impl CachedRenderPipelinePhaseItem for WaterMask {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        dbg!(&self.pipeline);
        self.pipeline
    }
}

pub type DrawWaterMask = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    DrawMesh2d,
);

pub struct WaterMaskPipeline {
    mesh_pipeline: Mesh2dPipeline,
    shader: Handle<Shader>,
}

impl FromWorld for WaterMaskPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.get_resource::<Mesh2dPipeline>().unwrap().clone();

        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/mask.wgsl");

        WaterMaskPipeline { mesh_pipeline, shader }
    }
}

impl SpecializedMeshPipeline for WaterMaskPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &Hashed<InnerMeshVertexBufferLayout, FixedState>,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut desc = self.mesh_pipeline.specialize(key, layout)?;

        desc.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
        ]);

        desc.vertex.shader = self.shader.clone();

        desc.fragment = Some(FragmentState {
            shader: self.shader.clone(),
            shader_defs: vec![],
            entry_point: "fragment".into(),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::R8Unorm,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        });
        desc.depth_stencil = None;

        desc.multisample = MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        desc.label = Some("mesh_stencil_pipeline".into());
        Ok(desc)
    }
}

/// Render graph node for producing stencils from meshes.
pub struct WaterMaskNode {
    query: QueryState<&'static RenderPhase<WaterMask>>,
}

impl WaterMaskNode {
    pub const IN_VIEW: &'static str = "view";

    /// The produced stencil buffer.
    ///
    /// This has format `TextureFormat::Depth24PlusStencil8`. Fragments covered
    /// by a mesh are assigned a value of 255. All other fragments are assigned
    /// a value of 0. The depth aspect is unused.
    pub const OUT_MASK: &'static str = "stencil";

    pub fn new(world: &mut World) -> WaterMaskNode {
        WaterMaskNode {
            query: QueryState::new(world),
        }
    }
}

impl Node for WaterMaskNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn output(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::OUT_MASK, SlotType::TextureView)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let res = world.get_resource::<WaterEffectResources>().unwrap();

        graph
            .set_output(Self::OUT_MASK, res.mask_multisample.default_view.clone())
            .unwrap();

        let view_entity = graph.get_input_entity(Self::IN_VIEW).unwrap();
        let stencil_phase = match self.query.get_manual(world, view_entity) {
            Ok(q) => q,
            Err(_) => return Ok(()),
        };

        let pass_raw = render_context
            .command_encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("water_effect_stencil_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &res.mask_multisample.default_view,
                    resolve_target: Some(&res.mask_output.default_view),
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK.into()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        let mut pass = TrackedRenderPass::new(pass_raw);

        let draw_functions = world
            .get_resource::<DrawFunctions<WaterMask>>()
            .unwrap();
        let mut draw_functions = draw_functions.write();
        for item in stencil_phase.items.iter() {
            let draw_function = draw_functions.get_mut(item.draw_function()).unwrap();
            draw_function.draw(world, &mut pass, view_entity, item);
        }

        Ok(())
    }
}
