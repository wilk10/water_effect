mod components;
mod graph;
mod jfa;
mod jfa_init;
mod mask;
mod plugin;
// mod render;
mod resources;
mod ripples;
mod ripples_style;

use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::texture::BevyDefault;

use crate::ripples_style::RipplesStyle;
use crate::components::*;
use crate::plugin::WaterEffectPlugin;

// TODO: most likely i can just move it inside WaterEffectResources

// TODO: still don't understand this
const JFA_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rg16Snorm;
// // TODO: still don't understand this
const FULLSCREEN_PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
    topology: PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: FrontFace::Ccw,
    cull_mode: Some(Face::Back),
    unclipped_depth: false,
    polygon_mode: PolygonMode::Fill,
    conservative: false,
};

// const JFA_TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WaterEffectPlugin)
        .add_startup_system(setup)
        .add_system(rotate_sprites)
        .add_system(lets_panic)
        ;
    }
}

#[derive(Clone, Debug, Component)]
struct RotationSpeed(f32);

fn setup(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    water_effect_images: Res<WaterEffectImages>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut ripples_styles: ResMut<Assets<RipplesStyle>>,
    mut water_sprites_materials: ResMut<Assets<WaterSpritesMaterial>>,
    // mut ripples_materials: ResMut<Assets<RipplesMaterial>>,
    // mut ripples_styles: ResMut<Assets<RipplesStyle>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::PURPLE,
                custom_size: Some(Vec2::new(400., 900.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(50.0, 0.0, 0.0),
            ..Default::default()
        }); 

    let red_sprite_entity =  commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(200., 120.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-200.0, 0.0, 1.0),
            ..Default::default()
        })
        .insert(RotationSpeed(0.08))
        // .insert(WaterSprite)
        .insert(WaterEffectImages::water_sprites_render_layer())
        .id();

    

    let green_sprite_entity = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(130., 220.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        })
        .insert(RotationSpeed(0.06))
        // .insert(WaterSprite)
        .insert(WaterEffectImages::water_sprites_render_layer())
        .id();

    

    let blue_sprite_entity = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(320., 130.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(200.0, 0.0, 3.0),
            ..Default::default()
        })
        .insert(RotationSpeed(0.04))
        // .insert(WaterSprite)
        .insert(WaterEffectImages::water_sprites_render_layer())
        .id();

   

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::ORANGE,
                custom_size: Some(Vec2::new(150., 150.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, -100.0, 4.0),
            ..Default::default()
        }); 

    let main_camera = MainCameraBundle::default();
    // let camera_z = main_camera.z();

    let main_camera_entity = commands.spawn_bundle(main_camera).id();
    let water_sprites_camera_entity = commands.spawn_bundle(WaterSpritesCameraBundle::new(&water_effect_images)).id();
    let ripples_camera_entity = commands.spawn_bundle(RipplesCameraBundle::new(&mut ripples_styles, &water_effect_images)).id();
    // let ripples_camera_entity = commands.spawn_bundle(RipplesCameraBundle::new(
    //     &water_effect_images,
    //     &mut ripples_styles,
    // )).id();
    // let water_effect_entity = commands.spawn_bundle(WaterEffectBundle::new(&mut meshes, &images, &water_effect_images, camera_z)).id();
    let water_sprites_texture_entity = commands.spawn_bundle(WaterSpritesToTextureBundle::new(
        &mut meshes,
        &mut water_sprites_materials,
        &images,
        &water_effect_images,
        // camera_z,
        // &mut ripples_styles,
    )).id();
    let ripples_texture_entity = commands.spawn_bundle(RipplesTextureBundle::new(
        // &mut meshes,
        // &mut ripples_materials,
        &images,
        &water_effect_images
    )).id();

    println!("\n=============================================");
    dbg!(&red_sprite_entity);
    dbg!(&green_sprite_entity);
    dbg!(&blue_sprite_entity);
    
    dbg!(&main_camera_entity);
    dbg!(&water_sprites_camera_entity);
    dbg!(&ripples_camera_entity);
    dbg!(&water_sprites_texture_entity);
    dbg!(&ripples_texture_entity);
    // dbg!(camera_z);

    // commands.entity(main_camera_entity).push_children(&[water_sprites_camera_entity]);
    // commands.entity(water_sprites_camera_entity).push_children(&[ripples_camera_entity, water_sprites_texture_entity]);

}

fn rotate_sprites(time: Res<Time>, mut query: Query<(&mut Transform, &RotationSpeed)>) {
    let delta = time.delta_seconds();

    for (mut transform, rot) in query.iter_mut() {
        transform.rotate_z(rot.0 * delta);
    }
}

fn lets_panic(time: Res<Time>) {
    if time.seconds_since_startup() > 1. {
        panic!("lets panic");
    }
}