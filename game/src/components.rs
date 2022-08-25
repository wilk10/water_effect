use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::render::render_resource::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::camera::RenderTarget;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::reflect::TypeUuid;
use bevy::sprite::Material2d;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::render::extract_resource::ExtractResource;
use bevy::ecs::query::QueryItem;
use bevy::render::extract_component::ExtractComponent;
use bevy::ecs::system::lifetimeless::Read;

use crate::ripples_style::RipplesStyle;

#[derive(Clone)]
pub struct WaterEffectImages {
    pub rendered_water_sprites: Handle<Image>,
    pub rendered_ripples: Handle<Image>, // TODO: need to remove this
}

impl WaterEffectImages {
    const RENDER_LAYER: u8 = 1;

    pub fn render_layers() -> RenderLayers {
        RenderLayers::layer(Self::RENDER_LAYER)
    }

    fn image_size(window: &Window) -> Extent3d {
        let extra_margin = Vec2::ZERO;
        let adjusted_size = Vec2::new(
            window.width() + extra_margin.x,
            window.height() + extra_margin.y,
        );
        Extent3d {
            width: adjusted_size.as_uvec2().x,
            height: adjusted_size.as_uvec2().y,
            depth_or_array_layers: 1,
        }
    }

    fn rendered_water_sprites_image(window: &Window) -> Image {
        let size = Self::image_size(window);
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..Default::default()
        };
        // NOTE: fill image.data with zeroes
        image.resize(size);

        image
    }

    fn rendered_ripples_image(window: &Window) -> Image {
        let size = Self::image_size(window);
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..Default::default()
        };
        // NOTE: fill image.data with zeroes
        image.resize(size);

        image
    }
}

impl FromWorld for WaterEffectImages {
    fn from_world(world: &mut World) -> Self {
        let (water_sprites_image, ripples_image) = {
        // let image = {
            let window = world
                .resource::<Windows>()
                .get_primary()
                .expect("cannot get primary Window in Windows");
            let rendered_water_sprites_image = Self::rendered_water_sprites_image(window);
            let rendered_ripples_image = Self::rendered_ripples_image(window);
            (rendered_water_sprites_image, rendered_ripples_image)
            // image
        };

        let mut images = world.resource_mut::<Assets<Image>>();

        Self {
            rendered_water_sprites: images.add(water_sprites_image),
            rendered_ripples: images.add(ripples_image),
        }
    }
}

#[derive(Bundle)]
pub struct MainCameraBundle {
    main_camera: MainCamera,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    #[bundle]
    bundle: Camera2dBundle,
}

impl MainCameraBundle {
    pub fn z(&self) -> f32 {
        self.bundle.transform.translation.z
    }
}

impl Default for MainCameraBundle {
    fn default() -> Self {
        Self {
            main_camera: MainCamera,
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            bundle: Camera2dBundle::default(),
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Bundle)]
pub struct RipplesCameraBundle {
    tag: RipplesCamera,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    #[bundle]
    camera_bundle: Camera2dBundle,
}

impl Default for RipplesCameraBundle {
    fn default() -> Self {
        let mut camera_bundle = Camera2dBundle::default();
        camera_bundle.camera_2d = Camera2d {
            clear_color: ClearColorConfig::None, // NOTE: i think this is correct...? not sure
        };
        camera_bundle.camera = Camera {
            // render before the "main pass" camera, after WaterSpritesCameraBundle
            priority: -1,
            // TODO: this currently renders to the primary window, but if i want more control,
            // it can render to an Image, if i want. I don't know, let's see
            ..Default::default()
        };
        camera_bundle.transform = Transform::from_translation(Vec3::ZERO);

        Self {
            tag: RipplesCamera,
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            camera_bundle,
        }
    }
}

#[derive(Component)]
pub struct RipplesCamera;

// TODO: try to see if i can remove this
impl ExtractComponent for RipplesCamera {
    type Query = Read<RipplesCamera>;

    type Filter = ();

    fn extract_component(_: QueryItem<Self::Query>) -> Self {
        RipplesCamera
    }
}

#[derive(Bundle)]
pub struct WaterSpritesCameraBundle {
    tag: WaterSpritesCamera,
    render_layers: RenderLayers,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    #[bundle]
    camera_bundle: Camera2dBundle,
}

impl WaterSpritesCameraBundle {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(water_effect_images: &WaterEffectImages) -> Self {
        let image_handle = water_effect_images.rendered_water_sprites.clone();
        // let mut color = Color::PINK;
        // color.set_a(0.);

        let color = Color::from(Vec4::ZERO);

        let mut camera_bundle = Camera2dBundle::default();
        camera_bundle.camera_2d = Camera2d {
            clear_color: ClearColorConfig::Custom(color),
        };
        camera_bundle.camera = Camera {
            // render before the RipplesCamera
            priority: -2,
            target: RenderTarget::Image(image_handle),
            ..Default::default()
        };
        camera_bundle.transform = Transform::from_translation(Vec3::ZERO);
        Self {
            tag: WaterSpritesCamera,
            render_layers: WaterEffectImages::render_layers(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            camera_bundle,
        }
    }
}

#[derive(Component)]
pub struct WaterSpritesCamera;

// #[derive(Bundle)]
// pub struct WaterEffectBundle {
//     water_effect: WaterEffect,
//     handle: Mesh2dHandle,
//     texture: Handle<Image>,
//     #[bundle]
//     spatial_bundle: SpatialBundle,
//     // #[bundle]
//     // sprite_bundle: SpriteBundle,
// }

// impl WaterEffectBundle {
//     pub fn new(meshes: &mut Assets<Mesh>, images: &Assets<Image>, water_effect_images: &WaterEffectImages, camera_z: f32) -> Self {
//         let image = images.get(&water_effect_images.rendered_water_sprites).unwrap();
//         let mesh_size = UVec2::new(
//             image.texture_descriptor.size.width,
//             image.texture_descriptor.size.height,
//         );
//         let quad = shape::Quad::new(mesh_size.as_vec2());

//         let translation = Vec3::new(0., 0., -camera_z + 0.01); // NOTE 0.01 only for debugging
//         Self {
//             water_effect: WaterEffect,
//             handle: meshes.add(Mesh::from(quad)).into(),
//             texture: water_effect_images.rendered_water_sprites.clone(),
//             spatial_bundle: SpatialBundle {
//                 transform: Transform::from_translation(translation),
//                 ..Default::default()
//             }
//             // sprite_bundle: SpriteBundle {
//             //     sprite: Sprite {
//             //         custom_size: Some(mesh_size.as_vec2()),
//             //         ..Default::default()
//             //      },
//             //     texture: water_effect_images.rendered_water_sprites.clone(),
//             //     transform: Transform::from_translation(translation),
//             //     ..Default::default()
//             // }
//         }
//     }
// }


#[derive(Default, Bundle)]
pub struct WaterSpritesToTextureBundle {
    tag: WaterSpritesToTexture,
    ripples_style: Handle<RipplesStyle>,
    // #[bundle]
    // sprite_bundle: SpriteBundle,
    #[bundle]
    pub material_2d_bundle: MaterialMesh2dBundle<WaterSpritesMaterial>,
}

impl WaterSpritesToTextureBundle {
    const Z: f32 = 1.0;

    pub fn new(
        meshes: &mut Assets<Mesh>, 
        materials: &mut Assets<WaterSpritesMaterial>, 
        images: &Assets<Image>, 
        water_effect_images: &WaterEffectImages,
        camera_z: f32,
        ripples_styles: &mut Assets<RipplesStyle>,
    ) -> Self {
        let water_sprites_material = WaterSpritesMaterial::new(&water_effect_images.rendered_water_sprites);

        let image = images.get(&water_effect_images.rendered_water_sprites).unwrap();
        let mesh_size = UVec2::new(
            image.texture_descriptor.size.width,
            image.texture_descriptor.size.height,
        );
        let quad = shape::Quad::new(mesh_size.as_vec2());

        let translation = Vec3::new(0., 0., -camera_z + Self::Z);

        Self {
            tag: WaterSpritesToTexture,
            ripples_style: ripples_styles.add(RipplesStyle::default().into()),
            material_2d_bundle: MaterialMesh2dBundle { 
                mesh: meshes.add(Mesh::from(quad)).into(), 
                material: materials.add(water_sprites_material), 
                transform: Transform::from_translation(translation), 
                ..Default::default()
            }
            // sprite_bundle: SpriteBundle {
            //     sprite: Sprite {
            //         custom_size: Some(mesh_size.as_vec2()),
            //         ..Default::default()
            //      },
            //     texture: water_effect_images.rendered_water_sprites.clone(),
            //     transform: Transform::from_translation(translation),
            //     ..Default::default()
            // }
        }
    }
}

#[derive(Debug, Default, Component)]
pub struct WaterSpritesToTexture;


/// WATER EFFECT

#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "d8f3e2a1-ee4e-425c-90c1-125fb82eac1f"]
pub struct WaterSpritesMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image_handle: Handle<Image>,
}

impl WaterSpritesMaterial {
    pub fn new(image_handle: &Handle<Image>) -> Self {
        Self {
            image_handle: image_handle.clone(),
        }
    }
}

impl Material2d for WaterSpritesMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_sprites.wgsl".into()
    }
}

#[derive(Default)]
pub struct ExtractedTime {
    pub seconds_since_startup: f32,
}

impl ExtractResource for ExtractedTime {
    type Source = Time;

    fn extract_resource(time: &Self::Source) -> Self {
        ExtractedTime {
            seconds_since_startup: time.seconds_since_startup() as f32,
        }
    }
}

// TODO: probably i can remove this, and in `prepare_time` system i can query WaterEffectResources instead
pub struct TimeBuffer {
    pub buffer: Buffer,
}

impl TimeBuffer {
    pub fn new(buffer: &Buffer) -> Self {
        Self {
            buffer: buffer.clone(),
        }
    }
}

