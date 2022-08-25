use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::*;
use bevy::{
    ecs::{system::SystemParamItem},
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::resources;

#[derive(Clone, Debug, PartialEq, TypeUuid)]
#[uuid = "6805d65e-f637-4a49-869a-889c0abe8140"]
pub struct RipplesStyle {
    pub water_color: Color,
    pub ripples_color: Color,
    pub distance_from_coast: f32,
    pub time: f32,      // ??
    pub frequency: f32, // https://itscai.us/blog/post/jfa/
    pub speed: f32,
}

impl Default for RipplesStyle {
    fn default() -> Self {
        Self {
            water_color: Color::BLUE,
            ripples_color: Color::BLACK,
            distance_from_coast: 100.,
            time: 0.,
            frequency: 0.5,
            speed: 1.,
        }
    }
}

impl RenderAsset for RipplesStyle {
    type ExtractedAsset = RipplesParams;
    type PreparedAsset = GpuRipplesParams;
    type Param = (
        Res<'static, RenderDevice>,
        Res<'static, RenderQueue>,
        Res<'static, resources::WaterEffectResources>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        RipplesParams::new(
            self.water_color,
            self.ripples_color,
            self.distance_from_coast,
            self.time,
            self.frequency,
            self.speed,
        )
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (device, queue, water_effect_res): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let mut buffer = UniformBuffer::from(extracted_asset.clone());
        buffer.write_buffer(device, queue);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &water_effect_res.water_effect_params_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.buffer().unwrap().as_entire_binding(),
            }],
        });

        Ok(GpuRipplesParams {
            params: extracted_asset,
            _buffer: buffer,
            bind_group,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, ShaderType)]
pub struct RipplesParams {
    pub(crate) water_color: Vec4,
    pub(crate) ripples_color: Vec4,
    pub(crate) distance_from_coast: f32,
    pub(crate) time: f32,      // ??
    pub(crate) frequency: f32, // https://itscai.us/blog/post/jfa/
    pub(crate) speed: f32,
}

impl RipplesParams {
    pub fn new(
        water_color: Color,
        ripples_color: Color,
        distance_from_coast: f32,
        time: f32,
        frequency: f32,
        speed: f32,
    ) -> RipplesParams {
        let water_color: Vec4 = water_color.as_rgba_f32().into();
        let ripples_color: Vec4 = ripples_color.as_rgba_f32().into();

        RipplesParams {
            water_color,
            ripples_color,
            distance_from_coast,
            time,
            frequency,
            speed,
        }
    }
}

pub struct GpuRipplesParams {
    pub(crate) params: RipplesParams,
    pub(crate) _buffer: UniformBuffer<RipplesParams>,
    pub(crate) bind_group: BindGroup,
}
