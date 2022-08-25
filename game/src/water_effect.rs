use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::TrackedRenderPass,
        render_resource::{
            BindGroup, BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, LoadOp,
            MultisampleState, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, ShaderType, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, TextureSampleType, TextureUsages,
            UniformBuffer, VertexState,
        },
        renderer::RenderContext,
        view::ExtractedWindows,
    },
};

use crate::{components::RipplesCamera};
use crate::ripples_style::RipplesStyle;
use crate::{
    resources::{self, WaterEffectResources},
    FULLSCREEN_PRIMITIVE_STATE,
};

#[derive(Clone, Debug, Default, PartialEq, ShaderType)]
pub struct WaterEffectParams {
    pub(crate) water_color: Vec4,
    pub(crate) ripples_color: Vec4,
    pub(crate) distance_from_coast: f32,
    pub(crate) time: f32,      // ??
    pub(crate) frequency: f32, // https://itscai.us/blog/post/jfa/
    pub(crate) speed: f32,
}

impl WaterEffectParams {
    pub fn new(
        water_color: Color,
        ripples_color: Color,
        distance_from_coast: f32,
        time: f32,
        frequency: f32,
        speed: f32,
    ) -> WaterEffectParams {
        let water_color: Vec4 = water_color.as_rgba_f32().into();
        let ripples_color: Vec4 = ripples_color.as_rgba_f32().into();

        WaterEffectParams {
            water_color,
            ripples_color,
            distance_from_coast,
            time,
            frequency,
            speed,
        }
    }
}

pub struct GpuWaterEffectParams {
    pub(crate) params: WaterEffectParams,
    pub(crate) _buffer: UniformBuffer<WaterEffectParams>,
    pub(crate) bind_group: BindGroup,
}

#[derive(Clone, Debug)]
pub struct WaterEffectPipeline {
    dimensions_layout: BindGroupLayout,
    input_layout: BindGroupLayout,
    params_layout: BindGroupLayout,
    shader: Handle<Shader>,
}

impl FromWorld for WaterEffectPipeline {
    fn from_world(world: &mut World) -> Self {
        let res = world
            .get_resource::<resources::WaterEffectResources>()
            .unwrap();

        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/water_effect.wgsl");

        let dimensions_layout = res.dimensions_bind_group_layout.clone();
        let input_layout = res.water_effect_src_bind_group_layout.clone();
        let params_layout = res.water_effect_params_bind_group_layout.clone();

        WaterEffectPipeline {
            dimensions_layout,
            input_layout,
            params_layout,
            shader,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WaterEffectPipelineKey {
    format: TextureFormat,
}

impl WaterEffectPipelineKey {
    pub fn new(format: TextureFormat) -> Option<WaterEffectPipelineKey> {
        let info = format.describe();

        if info.sample_type == TextureSampleType::Depth {
            // Can't use this format as a color attachment.
            return None;
        }

        if info
            .guaranteed_format_features
            .allowed_usages
            .contains(TextureUsages::RENDER_ATTACHMENT)
        {
            Some(WaterEffectPipelineKey { format })
        } else {
            None
        }
    }
}

impl SpecializedRenderPipeline for WaterEffectPipeline {
    type Key = WaterEffectPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let blend = BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::Zero,
                operation: BlendOperation::Add,
            },
        };

        RenderPipelineDescriptor {
            label: Some("jfa_water_effect_pipeline".into()),
            layout: Some(vec![
                self.dimensions_layout.clone(),
                self.input_layout.clone(),
                self.params_layout.clone(),
            ]),
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.format,
                    blend: Some(blend),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: FULLSCREEN_PRIMITIVE_STATE,
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}

pub struct WaterEffectNode {
    pipeline_id: CachedRenderPipelineId,
    camera_query: QueryState<(&'static ExtractedCamera, &'static RipplesCamera)>,
    ripples_query: QueryState<&'static Handle<RipplesStyle>>,
}

impl WaterEffectNode {
    pub const IN_VIEW: &'static str = "in_view";
    pub const IN_JFA: &'static str = "in_jfa";
    pub const OUT_VIEW: &'static str = "out_view";

    pub fn new(world: &mut World, target_format: TextureFormat) -> WaterEffectNode {
        let pipeline_id = world.resource_scope(|world, mut cache: Mut<PipelineCache>| {
            let base = world.get_resource::<WaterEffectPipeline>().unwrap().clone();
            let mut spec = world
                .get_resource_mut::<SpecializedRenderPipelines<WaterEffectPipeline>>()
                .unwrap();
            let key = WaterEffectPipelineKey::new(target_format)
                .expect("invalid format for WaterEffectNode");
            spec.specialize(&mut cache, &base, key)
        });

        let camera_query = QueryState::new(world);
        let ripples_query = QueryState::new(world);

        WaterEffectNode { pipeline_id, camera_query, ripples_query }
    }
}

impl Node for WaterEffectNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![
            SlotInfo {
                name: Self::IN_JFA.into(),
                slot_type: SlotType::TextureView,
            },
            SlotInfo {
                name: Self::IN_VIEW.into(),
                slot_type: SlotType::Entity,
            },
        ]
    }

    fn output(&self) -> Vec<SlotInfo> {
        vec![SlotInfo {
            name: Self::OUT_VIEW.into(),
            slot_type: SlotType::Entity,
        }]
    }

    fn update(&mut self, world: &mut World) {
        self.camera_query.update_archetypes(world);
        self.ripples_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        

        let view_ent = graph.get_input_entity(Self::IN_VIEW)?;
        graph.set_output(Self::OUT_VIEW, view_ent)?;

        dbg!(&view_ent);

        let (extracted_camera, _ripples_camera_tag) = &self.camera_query.get_manual(world, view_ent).unwrap();

        // let style = match self
        //     .ripples_query
        //     .get_manual(world, graph.get_input_entity(Self::IN_VIEW)?) // the input entity is the one with the handle riiiiight?
        // {
        //     Ok(ripples_style) => {
        //         dims.width.max(dims.height).min(
        //            styles
        //                 .get(&ripples_style)
        //                 .unwrap()
        //                 .params
        //                 .distance_from_coast
        //                 .ceil(),
        //         )
        //     }
        //     Err(_) => return Ok(()),
        // };

        let style = match self
            .ripples_query
            .get_manual(world, graph.get_input_entity(Self::IN_VIEW)?) {
                Ok(s) => s,
                Err(_) => return Ok(()),
            };

        dbg!(&extracted_camera);
        // dbg!(&water_camera);

        let windows = world.resource::<ExtractedWindows>();
        let images = world.resource::<RenderAssets<Image>>();
        let target_view = match extracted_camera.target.get_texture_view(windows, images) {
            Some(v) => v,
            None => return Ok(()),
        };

        dbg!(&target_view);

        let styles = world.resource::<RenderAssets<RipplesStyle>>();
        let style = styles.get(&style).unwrap();

        dbg!(&style.params);

        let res = world.get_resource::<WaterEffectResources>().unwrap();

        let pipelines = world.get_resource::<PipelineCache>().unwrap();
        let pipeline = match pipelines.get_render_pipeline(self.pipeline_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        let render_pass = render_context
            .command_encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("jfa_water_effect"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                // TODO: support outlines being occluded by world geometry
                depth_stencil_attachment: None,
            });

        let mut tracked_pass = TrackedRenderPass::new(render_pass);
        tracked_pass.set_render_pipeline(pipeline);
        tracked_pass.set_bind_group(0, &res.dimensions_bind_group, &[]);
        tracked_pass.set_bind_group(1, &res.water_effect_src_bind_group, &[]);
        tracked_pass.set_bind_group(2, &style.bind_group, &[]);
        tracked_pass.draw(0..3, 0..1);

        Ok(())
    }
}
