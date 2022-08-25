use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_resource::*;
use bevy::render::RenderStage;
use bevy::render::Extract;
use bevy::render::render_phase::DrawFunctions;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::render::render_asset::RenderAssets;
use bevy::sprite::Mesh2dHandle;
use bevy::sprite::Mesh2dUniform;
use bevy::render::view::ExtractedView;
use bevy::render::view::VisibleEntities;
use bevy::render::render_phase::RenderPhase;
use bevy::utils::FloatOrd;
use bevy::sprite::Mesh2dPipelineKey;
use bevy::render::render_phase::AddRenderCommand;
use bevy::sprite::Material2dPlugin;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_asset::RenderAssetPlugin;
use bevy::render::render_phase::SetItemPipeline;
use bevy::render::extract_component::ExtractComponentPlugin;

use crate::components::WaterEffectImages;
use crate::components::WaterSpritesMaterial;
use crate::ripples_style::RipplesStyle;
// use crate::components::WaterEffect;
// use crate::render::WaterEffectPipeline;
// use crate::render::DrawWaterEffect;
// use crate::render::AdditionalDebugNode;
use crate::mask::WaterMask;
use crate::mask::DrawWaterMask;
use crate::resources;
use crate::mask::WaterMaskPipeline;
use crate::jfa_init::JfaInitPipeline;
use crate::jfa::JfaPipeline;
use crate::water_effect::WaterEffectPipeline;
use crate::graph;
use crate::components::WaterSpritesToTexture;
use crate::components::RipplesCamera;

pub struct WaterEffectPlugin;

impl Plugin for WaterEffectPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(ExtractComponentPlugin::<WaterEffect>::default()) // TODO: is this necessary?
            .add_plugin(Material2dPlugin::<WaterSpritesMaterial>::default())
            .add_plugin(RenderAssetPlugin::<RipplesStyle>::default())
            .add_asset::<RipplesStyle>()
            .init_resource::<WaterEffectImages>();

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(r) => r,
            Err(_) => return,
        };

        render_app
            .init_resource::<DrawFunctions<WaterMask>>()
            .add_render_command::<WaterMask, SetItemPipeline>()
            .add_render_command::<WaterMask, DrawWaterMask>()
            .init_resource::<resources::WaterEffectResources>()
            .init_resource::<WaterMaskPipeline>()
            .init_resource::<SpecializedMeshPipelines<WaterMaskPipeline>>()
            .init_resource::<JfaInitPipeline>()
            .init_resource::<JfaPipeline>()
            .init_resource::<WaterEffectPipeline>()
            .init_resource::<SpecializedRenderPipelines<WaterEffectPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_ripples_styles)
            .add_system_to_stage(RenderStage::Extract, extract_ripples_camera_and_add_water_mask_phase)
            .add_system_to_stage(RenderStage::Prepare,resources::recreate)
            .add_system_to_stage(RenderStage::Queue, queue_water_mask);

        let water_effect_subgraph = graph::water_effect(render_app).unwrap();

        let mut root_graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_2d_graph = root_graph.get_sub_graph_mut(bevy::core_pipeline::core_2d::graph::NAME).unwrap();
        let draw_2d_input = draw_2d_graph.input_node().unwrap().id;

        draw_2d_graph.add_sub_graph(graph::water_effect::NAME, water_effect_subgraph);
        let water_effect_driver =
            draw_2d_graph.add_node(graph::WaterEffectDriverNode::NAME, graph::WaterEffectDriverNode);
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

        // let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        // let draw_2d_graph = render_graph.get_sub_graph_mut(bevy::core_pipeline::core_2d::graph::NAME).unwrap();
        // let draw_2d_input = draw_2d_graph.input_node().unwrap().id;

        // let additional_debug_node = AdditionalDebugNode::default();
        // render_graph.add_node(AdditionalDebugNode::NAME, additional_debug_node);
        // render_graph
        //     .add_slot_edge(
        //         draw_2d_input,
        //         bevy::core_pipeline::core_2d::graph::input::VIEW_ENTITY,
        //         additional_debug_node,
        //         AdditionalDebugNode::INPUT_VIEW,
        //     )
        //     .unwrap();
        // render_graph
        //     .add_node_edge(
        //         bevy::core_pipeline::core_2d::graph::node::MAIN_PASS,
        //         AdditionalDebugNode::NAME,
        //     )
        //     .unwrap();

        // render_app
        //     .add_render_command::<Transparent2d, DrawWaterEffect>()
        //     .init_resource::<WaterEffectPipeline>()
        //     .init_resource::<SpecializedMeshPipelines<WaterEffectPipeline>>()
        //     .add_system_to_stage(RenderStage::Extract, extract_water_effect_mesh2d)
        //     .add_system_to_stage(RenderStage::Queue, queue_water_effect_mesh);
    }
}

fn extract_ripples_styles(
    mut commands: Commands,
    mut previous_ripples_styles_len: Local<usize>,
    water_texture: Query<(Entity, &Handle<RipplesStyle>), With<WaterSpritesToTexture>>,
) {
    let mut batches = Vec::with_capacity(*previous_ripples_styles_len);
    batches.extend(
        water_texture
            .iter()
            .map(|(entity, style)| (entity, (style.clone(),))),
    );
    *previous_ripples_styles_len = batches.len();
    commands.insert_or_spawn_batch(batches);
}

fn extract_ripples_camera_and_add_water_mask_phase(
    mut commands: Commands,
    cameras: Query<Entity, With<RipplesCamera>>,
) {
    for entity in cameras.iter() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<WaterMask>::default());
    }
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

    for (_view, visible_entities, mut mesh_mask_phase) in views.iter_mut() {

        // dbg!(&view.width);
        // dbg!(&view.height);
        // dbg!(&visible_entities);

        // let view_matrix = view.transform.compute_matrix();
        // let inv_view_row_2 = view_matrix.inverse().row(2);

        for visible_entity in visible_entities.entities.iter().copied() {
            let (entity, mesh2d_handle, mesh2d_uniform) = match water_sprites_mesh.get(visible_entity) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let mesh = match render_meshes.get(&mesh2d_handle.0) {
                Some(m) => m,
                None => continue,
            };

            let key = Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);

            let pipeline = pipelines
                .specialize(&mut pipeline_cache, &mesh_mask_pipeline, key, &mesh.layout)
                .unwrap();

            let mesh_z = mesh2d_uniform.transform.w_axis.z;

            mesh_mask_phase.add(WaterMask {
                entity,
                pipeline,
                draw_function: draw_water_mask,
                distance: mesh_z,
            });
        }
    }
}














// pub fn extract_water_effect_mesh2d(
//     mut commands: Commands,
//     mut previous_len: Local<usize>,
//     query: Extract<Query<(Entity, &ComputedVisibility), With<WaterEffect>>>,
// ) {
//     let mut values = Vec::with_capacity(*previous_len);
//     for (entity, _computed_visibility) in query.iter() {
//         // if !computed_visibility.is_visible() {
//         //     continue;
//         // }
//         values.push((entity, (WaterEffect,)));
//     }

//     dbg!(&values);

//     *previous_len = values.len();
//     commands.insert_or_spawn_batch(values);
// }

// fn queue_water_effect_mesh(
//     transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
//     water_effect_pipeline: Res<WaterEffectPipeline>,
//     msaa: Res<Msaa>,
//     mut pipelines: ResMut<SpecializedMeshPipelines<WaterEffectPipeline>>,
//     mut pipeline_cache: ResMut<PipelineCache>,
//     render_meshes: Res<RenderAssets<Mesh>>,
//     water_effect_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<WaterEffect>>,
//     mut views: Query<(&ExtractedView, &VisibleEntities, &mut RenderPhase<Transparent2d>)>,
// ) {
//     if water_effect_mesh2d.is_empty() {
//         warn!("water_effect_mesh2d is empty");
//         return;
//     }

//     dbg!(water_effect_mesh2d.iter().count());

//     // Iterate each view (a camera is a view)
//     for (_view, visible_entities, mut transparent_phase) in &mut views {

//         // dbg!(&view.width);
//         // dbg!(&view.height);
//         // dbg!(&visible_entities);

//         let draw_water_effect = transparent_2d_draw_functions
//             .read()
//             .get_id::<DrawWaterEffect>()
//             .unwrap();

//         // dbg!(&draw_water_effect);

//         // let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

//         // Queue all entities visible to that view
//         for visible_entity in &visible_entities.entities {

//             if let Ok((mesh2d_handle, mesh2d_uniform)) = water_effect_mesh2d.get(*visible_entity) {
//                 dbg!(&visible_entity);
//                 dbg!(&mesh2d_handle.0);

//                 let mesh2d_key = match render_meshes.get(&mesh2d_handle.0) {
//                     Some(mesh) =>{
//                         // dbg!("mesh found");
//                         Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology)
//                     },
//                     None =>  {
//                         // dbg!("mesh not found");
//                         Mesh2dPipelineKey::from_msaa_samples(msaa.samples)
//                     },
//                 };

//                 // dbg!(&mesh2d_key);

//                 let mesh2d_layout = match render_meshes.get(&mesh2d_handle.0) {
//                     Some(mesh) => &mesh.layout,
//                     None => {
//                         warn!("no mesh found with handle {:?}", &mesh2d_handle.0);
//                         continue
//                     },
//                 };

//                 // dbg!(&mesh2d_layout);

//                 let pipeline_id =
//                     pipelines.specialize(&mut pipeline_cache, &water_effect_pipeline, mesh2d_key, mesh2d_layout)
//                     .unwrap();

//                 dbg!(pipeline_id);

//                 // let pipeline = pipeline_cache.get_render_pipeline(pipeline_id).unwrap();

//                 // dbg!(pipeline);

//                 let mesh_z = mesh2d_uniform.transform.w_axis.z;

//                 dbg!(&mesh_z);

//                 transparent_phase.add(Transparent2d {
//                     entity: *visible_entity,
//                     draw_function: draw_water_effect,
//                     pipeline: pipeline_id,
//                     // The 2d render items are sorted according to their z value before rendering,
//                     // in order to get correct transparency
//                     sort_key: FloatOrd(mesh_z),
//                     // This material is not batched
//                     batch_range: None,
//                 });
//             }
//         }
//     }
// }