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
use bevy::render::render_graph::Node;
use bevy::render::render_graph::SlotInfo;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_graph::RenderGraphContext;
use bevy::render::renderer::RenderContext;
use bevy::render::render_graph::SlotType;
use bevy::render::render_phase::TrackedRenderPass;

// pub struct AdditionalDebugNode;

// impl AdditionalDebugNode {
//     pub const NAME: &'static str = "additional_node";
//     pub const INPUT_VIEW: &'static str = "view_entity";
//     pub const OUTPUT_VIEW: &'static str = "view_entity";
// }

// impl Default for AdditionalDebugNode {
//     fn default() -> Self {
//         AdditionalDebugNode
//     }
// }

// impl Node for AdditionalDebugNode {
//     fn input(&self) -> Vec<SlotInfo> {
//         vec![
//             SlotInfo {
//                 name: Self::INPUT_VIEW.into(),
//                 slot_type: SlotType::Entity,
//             }
//         ]
//     }

//     fn output(&self) -> Vec<SlotInfo> {
//         vec![SlotInfo {
//             name: Self::OUTPUT_VIEW.into(),
//             slot_type: SlotType::Entity,
//         }]
//     }
    
//     fn update(&mut self, _world: &mut World) {
//         // do this only if during `run` i need to access data in the render world via a query
//     }

//     fn run(
//         &self,
//         graph: &mut RenderGraphContext,
//         render_context: &mut RenderContext,
//         world: &World,
//     ) -> Result<(), NodeRunError> {
        
//         let view_entity = graph.get_input_entity(Self::INPUT_VIEW)?;
//         graph.set_output(Self::OUTPUT_VIEW, view_entity)?;

//         let pipelines = world.get_resource::<PipelineCache>().unwrap();
//         let pipeline = match pipelines.get_render_pipeline(self.pipeline_id) {
//             Some(p) => p,
//             None => return Ok(()),
//         };

//         let render_pass = render_context
//             .command_encoder
//             .begin_render_pass(&RenderPassDescriptor {
//                 label: Some("jfa_water_effect"),
//                 color_attachments: &[Some(RenderPassColorAttachment {
//                     view: target_view,
//                     resolve_target: None,
//                     ops: Operations {
//                         load: LoadOp::Load,
//                         store: true,
//                     },
//                 })],
//                 // TODO: support outlines being occluded by world geometry
//                 depth_stencil_attachment: None,
//             });

//         let mut tracked_pass = TrackedRenderPass::new(render_pass);
//         tracked_pass.set_render_pipeline(pipeline);
//         tracked_pass.set_bind_group(0, &res.dimensions_bind_group, &[]);
//         tracked_pass.set_bind_group(1, &res.water_effect_src_bind_group, &[]);
//         tracked_pass.set_bind_group(2, &style.bind_group, &[]);
//         tracked_pass.draw(0..3, 0..1);

//         Ok(())
        
//     }
// }

// pub struct WaterEffectPipeline {
//     pub shader: Handle<Shader>,
//     pub mesh2d_pipeline: Mesh2dPipeline,
// }

// impl FromWorld for WaterEffectPipeline {
//     fn from_world(world: &mut World) -> Self {
//         let asset_server = world.resource::<AssetServer>();
//         let shader = asset_server.load("shaders/water_effect.wgsl");
//         dbg!(&shader);
//         Self {
//             shader,
//             mesh2d_pipeline: Mesh2dPipeline::from_world(world),
//         }
//     }
// }

// impl SpecializedMeshPipeline for WaterEffectPipeline {
//     type Key = Mesh2dPipelineKey;

//     fn specialize(
//         &self,
//         key: Self::Key,
//         layout: &Hashed<InnerMeshVertexBufferLayout, FixedState>,
//     ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
//         let mut desc = self.mesh2d_pipeline.specialize(key, layout)?;

//         // desc.layout = Some(vec![
//         //     self.mesh2d_pipeline.view_layout.clone(),
//         //     self.mesh2d_pipeline.mesh_layout.clone(),
//         // ]);

//         desc.vertex.shader = self.shader.clone();
//         desc.fragment.as_mut().unwrap().shader = self.shader.clone();

//         // desc.vertex = VertexState {
//         //     shader: WATER_EFFECT_SHADER_HANDLE.typed::<Shader>(),
//         //     shader_defs: vec![],
//         //     entry_point: "vertex".into(),
//         //     buffers: vec![],
//         // };
//         // desc.fragment = Some(FragmentState {
//         //     shader: WATER_EFFECT_SHADER_HANDLE.typed::<Shader>(),
//         //     shader_defs: vec![],
//         //     entry_point: "fragment".into(),
//         //     targets: vec![Some(ColorTargetState {
//         //         format: TextureFormat::bevy_default(),
//         //         blend: Some(BlendState::ALPHA_BLENDING),
//         //         write_mask: ColorWrites::ALL,
//         //     })],
//         // });

//         // desc.fragment = Some(FragmentState {
//         //     shader: self.shader.clone(),
//         //     shader_defs: vec![],
//         //     entry_point: "fragment".into(),
//         //     targets: vec![Some(ColorTargetState {
//         //         format: TextureFormat::R8Unorm,
//         //         blend: None,
//         //         write_mask: ColorWrites::ALL,
//         //     })],
//         // });
//         // desc.depth_stencil = None;

//         desc.multisample = MultisampleState {
//             count: 4,
//             mask: !0,
//             alpha_to_coverage_enabled: false,
//         };

//         // desc.label = Some("mesh_stencil_pipeline".into());
//         Ok(desc)
//     }
// }

// // This specifies how to render a colored 2d mesh
// pub type DrawWaterEffect = (
//     // Set the pipeline
//     SetItemPipeline,
//     // Set the view uniform as bind group 0
//     SetMesh2dViewBindGroup<0>,
//     // Set the mesh uniform as bind group 1
//     SetMesh2dBindGroup<1>,
//     // Draw the mesh
//     DrawMesh2d,
// );

