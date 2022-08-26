use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_resource::*;
use bevy::render::RenderStage;
use bevy::render::render_phase::DrawFunctions;
use bevy::render::render_asset::RenderAssets;
use bevy::sprite::Mesh2dHandle;
use bevy::sprite::Mesh2dUniform;
use bevy::render::view::ExtractedView;
use bevy::render::view::VisibleEntities;
use bevy::render::render_phase::RenderPhase;
use bevy::sprite::Mesh2dPipelineKey;
use bevy::render::render_phase::AddRenderCommand;
use bevy::sprite::Material2dPlugin;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_asset::RenderAssetPlugin;
use bevy::render::render_phase::SetItemPipeline;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::renderer::RenderQueue;
use bevy::render::Extract;
use bevy::reflect::TypeUuid;
use bevy::asset::load_internal_asset;

use crate::components::WaterEffectImages;
use crate::components::WaterSpritesMaterial;
use crate::ripples_style::RipplesStyle;
use crate::mask::WaterMask;
use crate::mask::DrawWaterMask;
use crate::resources;
use crate::mask::WaterMaskPipeline;
use crate::jfa_init::JfaInitPipeline;
use crate::jfa::JfaPipeline;
use crate::ripples::RipplesPipeline;
use crate::graph;
use crate::components::WaterSpritesToTexture;
use crate::components::RipplesCamera;
use crate::components::ExtractedTime;
// use crate::components::RipplesMaterial;

const FULLSCREEN_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12099561278220359682);
const DIMENSIONS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11721531257850828867);

pub struct WaterEffectPlugin;

impl Plugin for WaterEffectPlugin {
    fn build(&self, app: &mut App) {

        load_internal_asset!(
            app,
            FULLSCREEN_SHADER_HANDLE,
            "internal_shaders/fullscreen.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            DIMENSIONS_SHADER_HANDLE,
            "internal_shaders/dimensions.wgsl",
            Shader::from_wgsl
        );

        app
            .add_plugin(ExtractComponentPlugin::<RipplesCamera>::default())
            .add_plugin(ExtractComponentPlugin::<WaterSpritesToTexture>::default()) // TODO: is this necessary?
            .add_plugin(ExtractResourcePlugin::<ExtractedTime>::default())
            .add_plugin(Material2dPlugin::<WaterSpritesMaterial>::default())
            // .add_plugin(Material2dPlugin::<RipplesMaterial>::default())
            .add_plugin(RenderAssetPlugin::<RipplesStyle>::default())
            .add_asset::<RipplesStyle>()
            .init_resource::<WaterEffectImages>();

    
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(r) => r,
            Err(_) => return,
        };

        let water_effect_driver_node = graph::WaterEffectDriverNode::new(&mut render_app.world);

        render_app
            .init_resource::<DrawFunctions<WaterMask>>()
            .add_render_command::<WaterMask, SetItemPipeline>()
            .add_render_command::<WaterMask, DrawWaterMask>()
            .init_resource::<resources::WaterEffectResources>()
            .init_resource::<WaterMaskPipeline>()
            .init_resource::<SpecializedMeshPipelines<WaterMaskPipeline>>()
            .init_resource::<JfaInitPipeline>()
            .init_resource::<JfaPipeline>()
            .init_resource::<RipplesPipeline>()
            .init_resource::<SpecializedRenderPipelines<RipplesPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_ripples_styles)
            .add_system_to_stage(RenderStage::Extract, extract_ripples_camera_and_add_water_mask_phase)
            .add_system_to_stage(RenderStage::Prepare, prepare_time)
            .add_system_to_stage(RenderStage::Prepare,resources::recreate)
            .add_system_to_stage(RenderStage::Queue, queue_water_mask);

        let water_effect_subgraph = graph::water_effect(render_app).unwrap();

        let mut root_graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_2d_graph = root_graph.get_sub_graph_mut(bevy::core_pipeline::core_2d::graph::NAME).unwrap();
        let draw_2d_input = draw_2d_graph.input_node().unwrap().id;

        draw_2d_graph.add_sub_graph(graph::water_effect::NAME, water_effect_subgraph);
        let water_effect_driver =
            draw_2d_graph.add_node(graph::WaterEffectDriverNode::NAME, water_effect_driver_node);
        draw_2d_graph
            .add_slot_edge(
                draw_2d_input,
                bevy::core_pipeline::core_2d::graph::input::VIEW_ENTITY,
                water_effect_driver,
                graph::WaterEffectDriverNode::INPUT_VIEW,
            )
            .unwrap();
        draw_2d_graph
            .add_node_edge(bevy::core_pipeline::core_2d::graph::node::MAIN_PASS, water_effect_driver)
            .unwrap();
    }
}

fn extract_ripples_styles(
    mut commands: Commands,
    mut previous_ripples_styles_len: Local<usize>,
    ripples_camera: Extract<Query<(Entity, &Handle<RipplesStyle>), With<RipplesCamera>>>,
) {
    let mut batches = Vec::with_capacity(*previous_ripples_styles_len);
    batches.extend(
        ripples_camera
            .iter()
            .map(|(entity, style)| (entity, (style.clone(),))),
    );
    *previous_ripples_styles_len = batches.len();

    dbg!(&batches);

    commands.insert_or_spawn_batch(batches);
}

fn extract_ripples_camera_and_add_water_mask_phase(
    mut commands: Commands,
    cameras: Extract<Query<Entity, With<RipplesCamera>>>,
) {
    for entity in cameras.iter() {

        dbg!(&entity);

        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<WaterMask>::default());
    }
}

fn prepare_time(
    time: Res<ExtractedTime>,
    water_effect_resources: Res<resources::WaterEffectResources>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &water_effect_resources.ripples_time_uniform_buffer,
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}

fn queue_water_mask(
    water_mask_draw_function: Res<DrawFunctions<WaterMask>>,
    mesh_mask_pipeline: Res<WaterMaskPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<WaterMaskPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    render_meshes: Res<RenderAssets<Mesh>>,
    water_sprites_mesh: Query<(Entity, &Mesh2dHandle, &Mesh2dUniform), With<WaterSpritesToTexture>>,
    mut views: Query<(
        &ExtractedView,
        &mut VisibleEntities,
        &mut RenderPhase<WaterMask>,
    )>,
) {
    let draw_water_mask = water_mask_draw_function
        .read()
        .get_id::<DrawWaterMask>()
        .unwrap();

    for (view, visible_entities, mut mesh_mask_phase) in views.iter_mut() {

        // TODO: this is 1600 and 900 and they are a bit weird honestly, why?
        dbg!(&view.width);
        dbg!(&view.height);

        // NOTE: ok, with render layers this works, it only sees the one texture it's supposed to see
        dbg!(&visible_entities);

        // let view_matrix = view.transform.compute_matrix();
        // let inv_view_row_2 = view_matrix.inverse().row(2);

        for visible_entity in visible_entities.entities.iter().copied() {

            let (entity, mesh2d_handle, mesh2d_uniform) = match water_sprites_mesh.get(visible_entity) {
                Ok(m) => m,
                Err(_) => continue,
            };

            dbg!(&visible_entity);
            dbg!(&mesh2d_handle);

            let mesh = match render_meshes.get(&mesh2d_handle.0) {
                Some(m) => m,
                None => continue,
            };

            let key = Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);

            dbg!(&key);

            let pipeline = pipelines
                .specialize(&mut pipeline_cache, &mesh_mask_pipeline, key, &mesh.layout)
                .unwrap();

            let mesh_z = mesh2d_uniform.transform.w_axis.z;

            dbg!(&mesh_z);

            mesh_mask_phase.add(WaterMask {
                entity,
                pipeline,
                draw_function: draw_water_mask,
                distance: mesh_z,
            });
        }
    }
}
