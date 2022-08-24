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

use crate::components::WaterEffectImages;
use crate::components::WaterEffect;
use crate::render::WaterEffectPipeline;
use crate::render::DrawWaterEffect;

pub struct WaterEffectPlugin;

impl Plugin for WaterEffectPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_plugin(ExtractComponentPlugin::<WaterEffect>::default()) // TODO: is this necessary?
            .add_plugin(Material2dPlugin::<WaterEffect>::default())
            .init_resource::<WaterEffectImages>();

        // let render_app = match app.get_sub_app_mut(RenderApp) {
        //     Ok(r) => r,
        //     Err(_) => return,
        // };

        // render_app
        //     .add_render_command::<Transparent2d, DrawWaterEffect>()
        //     .init_resource::<WaterEffectPipeline>()
        //     .init_resource::<SpecializedMeshPipelines<WaterEffectPipeline>>()
        //     .add_system_to_stage(RenderStage::Extract, extract_water_effect_mesh2d)
        //     .add_system_to_stage(RenderStage::Queue, queue_water_effect_mesh);
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