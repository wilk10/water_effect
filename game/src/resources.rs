use bevy::{
    prelude::*,
    render::{
        render_resource::{
            AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, DynamicUniformBuffer, Extent3d, FilterMode, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureDescriptor,
            TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
            TextureViewDimension, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::ExtractedWindows,
    },
    window::WindowId,
};

use crate::{jfa, water_effect, JFA_TEXTURE_FORMAT};

const JFA_FROM_PRIMARY: &str = "jfa_from_primary_output_bind_group";
const JFA_FROM_SECONDARY: &str = "jfa_from_secondary_output_bind_group";
const JFA_WATER_EFFECT_SRC: &str = "jfa_water_effect_src_bind_group";

pub struct WaterEffectResources {
    // Multisample target for initial mask pass.
    pub mask_multisample: CachedTexture,
    // Resolve target for the above.
    pub mask_output: CachedTexture,

    pub dimensions_bind_group_layout: BindGroupLayout,
    pub dimensions_buffer: UniformBuffer<jfa::Dimensions>,
    pub dimensions_bind_group: BindGroup,

    // Non-filtering sampler for all sampling operations.
    pub sampler: Sampler,

    // Bind group and layout for JFA init pass.
    pub jfa_init_bind_group_layout: BindGroupLayout,
    pub jfa_init_bind_group: BindGroup,

    // Bind group layout for JFA iteration passes.
    pub jfa_bind_group_layout: BindGroupLayout,
    // Dynamic uniform buffer containing power-of-two JFA distances from 1 to 32768.
    // TODO: use instance ID instead?
    pub jfa_distance_buffer: DynamicUniformBuffer<jfa::JumpDist>,
    pub jfa_distance_offsets: Vec<u32>,

    // Bind group for jump flood passes targeting the primary output.
    pub jfa_from_secondary_bind_group: BindGroup,
    // Primary jump flood output.
    pub jfa_primary_output: CachedTexture,

    // Bind group for jump flood passes targeting the secondary output.
    pub jfa_from_primary_bind_group: BindGroup,
    // Secondary jump flood output.
    pub jfa_secondary_output: CachedTexture,

    // Bind groups for the final jump flood pass.
    pub jfa_final_output: CachedTexture,

    // Bind group layout for sampling JFA results in the outline shader.
    pub water_effect_src_bind_group_layout: BindGroupLayout,
    // Bind group layout for outline style parameters.
    pub water_effect_params_bind_group_layout: BindGroupLayout,
    pub water_effect_src_bind_group: BindGroup,
}

impl WaterEffectResources {
    fn create_jfa_bind_group(
        &self,
        device: &RenderDevice,
        label: &str,
        input: &TextureView,
    ) -> BindGroup {
        create_jfa_bind_group(
            device,
            &self.jfa_bind_group_layout,
            label,
            self.jfa_distance_buffer.binding().unwrap(),
            input,
            &self.sampler,
        )
    }
}

fn create_jfa_bind_group(
    device: &RenderDevice,
    layout: &BindGroupLayout,
    label: &str,
    dist_buffer: BindingResource,
    input: &TextureView,
    sampler: &Sampler,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: dist_buffer,
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(input),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn create_water_effect_src_bind_group(
    device: &RenderDevice,
    layout: &BindGroupLayout,
    label: &str,
    src: &TextureView,
    mask: &TextureView,
    sampler: &Sampler,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(src),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(mask),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}

impl FromWorld for WaterEffectResources {
    fn from_world(world: &mut World) -> Self {
        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let device = world.get_resource::<RenderDevice>().unwrap().clone();
        let queue = world.get_resource::<RenderQueue>().unwrap().clone();
        let mut textures = world.get_resource_mut::<TextureCache>().unwrap();

        let mask_output_desc = tex_desc("outline_mask_output", size, TextureFormat::R8Unorm);
        let mask_multisample_desc = TextureDescriptor {
            label: Some("outline_mask_multisample"),
            sample_count: 4,
            ..mask_output_desc.clone()
        };
        let mask_multisample = textures.get(&device, mask_multisample_desc);
        let mask_output = textures.get(&device, mask_output_desc);

        let dims = jfa::Dimensions::new(size.width, size.height);
        let mut dimensions_buffer = UniformBuffer::from(dims);
        dimensions_buffer.write_buffer(&device, &queue);

        let dimensions_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("jfa_dimensions_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(jfa::Dimensions::min_size()),
                    },
                    count: None,
                }],
            });

        let dimensions_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("jfa_dimensions_bind_group"),
            layout: &dimensions_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: dimensions_buffer.binding().unwrap(),
            }],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("outline_jfa_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            compare: None,
            ..Default::default()
        });

        let jfa_init_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("outline_jfa_init_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let jfa_init_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("outline_jfa_init_bind_group"),
            layout: &jfa_init_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&mask_output.default_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let jfa_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("outline_jfa_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(jfa::JumpDist::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
        let mut jfa_distance_buffer = DynamicUniformBuffer::default();
        let mut jfa_distance_offsets = Vec::new();
        for exp in 0_u32..16 {
            // TODO: this should be a DynamicUniformBuffer
            let ofs = jfa_distance_buffer.push(jfa::JumpDist {
                dist: 2_u32.pow(exp),
            });

            jfa_distance_offsets.push(ofs);
        }
        jfa_distance_buffer.write_buffer(&device, &queue);

        let jfa_primary_output_desc =
            tex_desc("water_effect_jfa_primary_output", size, JFA_TEXTURE_FORMAT);
        let jfa_primary_output = textures.get(&device, jfa_primary_output_desc);
        let jfa_secondary_output_desc = tex_desc(
            "water_effect_jfa_secondary_output",
            size,
            JFA_TEXTURE_FORMAT,
        );
        let jfa_secondary_output = textures.get(&device, jfa_secondary_output_desc);
        let jfa_final_output_desc =
            tex_desc("water_effect_jfa_final_output", size, JFA_TEXTURE_FORMAT);
        let jfa_final_output = textures.get(&device, jfa_final_output_desc);

        let jfa_from_secondary_bind_group = create_jfa_bind_group(
            &device,
            &jfa_bind_group_layout,
            "water_effect_jfa_primary_bind_group",
            jfa_distance_buffer.binding().unwrap(),
            &jfa_secondary_output.default_view,
            &sampler,
        );
        let jfa_from_primary_bind_group = create_jfa_bind_group(
            &device,
            &jfa_bind_group_layout,
            "water_effect_jfa_secondary_bind_group",
            jfa_distance_buffer.binding().unwrap(),
            &jfa_primary_output.default_view,
            &sampler,
        );

        let mut water_effect_params_buffer =
            UniformBuffer::from(water_effect::WaterEffectParams::default());
        water_effect_params_buffer.write_buffer(&device, &queue);

        let water_effect_src_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("jfa_water_effect_bind_group_layout"),
                entries: &[
                    // JFA texture
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Mask
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        let water_effect_params_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("jfa_water_effect_params_bind_group_layout"),
                entries: &[
                    // OutlineParams
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(water_effect::WaterEffectParams::min_size()),
                        },
                        count: None,
                    },
                ],
            });

        let water_effect_src_bind_group = create_water_effect_src_bind_group(
            &device,
            &water_effect_src_bind_group_layout,
            "jfa_water_effect_src_bind_group",
            &jfa_final_output.default_view,
            &mask_output.default_view,
            &sampler,
        );

        WaterEffectResources {
            mask_multisample,
            mask_output,
            dimensions_bind_group_layout,
            dimensions_buffer,
            dimensions_bind_group,
            jfa_init_bind_group_layout,
            jfa_init_bind_group,
            jfa_bind_group_layout,
            sampler,
            jfa_distance_buffer,
            jfa_distance_offsets,
            jfa_primary_output,
            jfa_secondary_output,
            jfa_final_output,
            jfa_from_secondary_bind_group,
            jfa_from_primary_bind_group,
            water_effect_src_bind_group_layout,
            water_effect_params_bind_group_layout,
            water_effect_src_bind_group,
        }
    }
}

pub fn recreate(
    mut water_effect: ResMut<WaterEffectResources>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut textures: ResMut<TextureCache>,
    windows: Res<ExtractedWindows>,
) {
    let primary = match windows.get(&WindowId::primary()) {
        Some(w) => w,
        None => return,
    };

    let size = Extent3d {
        width: primary.physical_width,
        height: primary.physical_height,
        depth_or_array_layers: 1,
    };

    let jfa_size = size;

    let new_dims = jfa::Dimensions::new(size.width, size.height);
    let dims = water_effect.dimensions_buffer.get_mut();
    if *dims != new_dims {
        *dims = new_dims;
        water_effect.dimensions_buffer.write_buffer(&device, &queue);
    }

    let old_mask = water_effect.mask_multisample.texture.id();
    let mask_output_desc = tex_desc("water_effect_mask_output", size, TextureFormat::R8Unorm);
    let mask_multisample_desc = TextureDescriptor {
        label: Some("water_effect_mask_multisample"),
        sample_count: 4,
        ..mask_output_desc.clone()
    };

    // Recreate mask output targets.
    water_effect.mask_output = textures.get(&device, mask_output_desc);
    water_effect.mask_multisample = textures.get(&device, mask_multisample_desc);

    if water_effect.mask_output.texture.id() != old_mask {
        // Recreate JFA init pass bind group
        water_effect.jfa_init_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("water_effect_jfa_init_bind_group"),
            layout: &water_effect.jfa_init_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&water_effect.mask_output.default_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&water_effect.sampler),
                },
            ],
        });
    }

    let old_jfa_primary = water_effect.jfa_primary_output.texture.id();
    let jfa_primary_desc = tex_desc(
        "water_effect_jfa_primary_output",
        jfa_size,
        JFA_TEXTURE_FORMAT,
    );
    let jfa_primary_output = textures.get(&device, jfa_primary_desc);
    if jfa_primary_output.texture.id() != old_jfa_primary {
        water_effect.jfa_primary_output = jfa_primary_output;
        water_effect.jfa_from_primary_bind_group = water_effect.create_jfa_bind_group(
            &device,
            JFA_FROM_PRIMARY,
            &water_effect.jfa_primary_output.default_view,
        );
    }

    let old_jfa_secondary = water_effect.jfa_secondary_output.texture.id();
    let jfa_secondary_desc = tex_desc(
        "water_effect_jfa_secondary_output",
        jfa_size,
        JFA_TEXTURE_FORMAT,
    );
    let jfa_secondary_output = textures.get(&device, jfa_secondary_desc);
    if jfa_secondary_output.texture.id() != old_jfa_secondary {
        water_effect.jfa_secondary_output = jfa_secondary_output;
        water_effect.jfa_from_secondary_bind_group = water_effect.create_jfa_bind_group(
            &device,
            JFA_FROM_SECONDARY,
            &water_effect.jfa_secondary_output.default_view,
        );
    }

    let old_jfa_final = water_effect.jfa_final_output.texture.id();
    let jfa_final_desc = tex_desc("water_effect_jfa_final_output", size, JFA_TEXTURE_FORMAT);
    let jfa_final_output = textures.get(&device, jfa_final_desc);
    if jfa_final_output.texture.id() != old_jfa_final {
        water_effect.jfa_final_output = jfa_final_output;
        water_effect.water_effect_src_bind_group = create_water_effect_src_bind_group(
            &device,
            &water_effect.water_effect_src_bind_group_layout,
            JFA_WATER_EFFECT_SRC,
            &water_effect.jfa_final_output.default_view,
            &water_effect.mask_output.default_view,
            &water_effect.sampler,
        );
    }
}

fn tex_desc(label: &'static str, size: Extent3d, format: TextureFormat) -> TextureDescriptor {
    TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    }
}
