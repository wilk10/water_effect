use bevy::sprite::Mesh2dPipeline;
use bevy::sprite::Mesh2dPipelineKey;
use bevy::{
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
use bevy::render::render_phase::EntityRenderCommand;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::render::render_phase::RenderCommandResult;
use bevy::sprite::RenderMaterials2d;
use bevy::ecs::system::lifetimeless::SQuery;
use bevy::ecs::system::lifetimeless::Read;
use bevy::render::renderer::RenderDevice;

use crate::components::WaterSpritesMaterial;
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
    SetWaterTextureViewBindGroup<1>,
    SetMesh2dBindGroup<2>,
    DrawMesh2d,
);

pub struct SetWaterTextureViewBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetWaterTextureViewBindGroup<I> {
    type Param = (SRes<RenderMaterials2d<WaterSpritesMaterial>>, SQuery<Read<Handle<WaterSpritesMaterial>>>);

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (materials, query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let material2d_handle = query.get(item).unwrap();

        dbg!(&material2d_handle);

        let material2d = materials.into_inner().get(material2d_handle).unwrap();

        dbg!(&material2d.key);

        pass.set_bind_group(I, &material2d.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct WaterMaskPipeline {
    mesh_pipeline: Mesh2dPipeline,
    shader: Handle<Shader>,
    texture_view_bind_group_layout: BindGroupLayout,
}

impl FromWorld for WaterMaskPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.get_resource::<Mesh2dPipeline>().unwrap().clone();

        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/mask.wgsl");

        dbg!(&shader);

        let render_device = world.resource::<RenderDevice>();
        let texture_view_bind_group_layout = WaterSpritesMaterial::bind_group_layout(render_device);

        dbg!(&texture_view_bind_group_layout);

        WaterMaskPipeline { mesh_pipeline, shader, texture_view_bind_group_layout }
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
            self.texture_view_bind_group_layout.clone(),
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

        desc.label = Some("water_mask_stencil_pipeline".into());

        dbg!(&desc);

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

        dbg!(&view_entity);

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
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        dbg!(&pass_raw);

        let mut pass = TrackedRenderPass::new(pass_raw);

        let draw_functions = world
            .get_resource::<DrawFunctions<WaterMask>>()
            .unwrap();
        let mut draw_functions = draw_functions.write();

        for item in stencil_phase.items.iter() {

            dbg!(&item);

            let draw_function = draw_functions.get_mut(item.draw_function()).unwrap();
            draw_function.draw(world, &mut pass, view_entity, item);
        }

        Ok(())
    }
}
