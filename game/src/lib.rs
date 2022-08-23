mod components;
mod plugin;
mod render;

use bevy::prelude::*;

use crate::components::*;
use crate::plugin::WaterEffectPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WaterEffectPlugin)
        .add_startup_system(setup)
        .add_system(rotate_sprites)
        // .add_system(lets_panic)
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
        .insert(RotationSpeed(0.8))
        // .insert(WaterSprite)
        .insert(WaterEffectImages::render_layers())
        .id();

    dbg!(&red_sprite_entity);

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
        .insert(RotationSpeed(0.6))
        // .insert(WaterSprite)
        .insert(WaterEffectImages::render_layers())
        .id();

    dbg!(&green_sprite_entity);

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
        .insert(RotationSpeed(0.4))
        // .insert(WaterSprite)
        .insert(WaterEffectImages::render_layers())
        .id();

    dbg!(&blue_sprite_entity);

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
    let camera_z = main_camera.z();

    dbg!(camera_z);

    let main_camera_entity = commands.spawn_bundle(main_camera).id();
    let water_camera_entity = commands.spawn_bundle(WaterCameraBundle::new(&water_effect_images)).id();
    // let ripples_camera_entity = commands.spawn_bundle(RipplesCameraBundle::new(
    //     &water_effect_images,
    //     &mut ripples_styles,
    // )).id();
    let water_effect_entity = commands.spawn_bundle(WaterEffectBundle::new(&mut meshes, &images, &water_effect_images, camera_z)).id();

    dbg!(&main_camera_entity);
    dbg!(&water_camera_entity);
    // dbg!(&ripples_camera_entity);
    dbg!(&water_effect_entity);

    commands.entity(main_camera_entity).push_children(&[water_camera_entity, water_effect_entity]);

    // commands.entity(main_camera_entity).push_children(&[water_camera_entity]);
    // commands.entity(water_camera_entity).push_children(&[ripples_camera_entity]);
    // commands.entity(main_camera_entity).push_children(&[ripples_camera_entity]);
    // commands.entity(ripples_camera_entity).push_children(&[water_effect_entity]);

    // commands
    //     .entity(main_camera_entity)
    //     .with_children(|parent| {
    //         parent
    //             .spawn_bundle(WaterCameraBundle::new(&water_effect_images))
    //             .with_children(|parent| {
    //                 parent
    //                     .spawn_bundle(RipplesCameraBundle::new(
    //                         &water_effect_images,
    //                         &mut ripples_styles,
    //                     ))
    //                     .with_children(|parent| {
    //                         parent.spawn_bundle(WaterEffectBundle::default());
    //                         // .spawn_bundle(MaterialMesh2dBundle {
    //                         //     mesh: meshes.add(Mesh::from(quad)).into(),
    //                         //     material: materials.add(water_effect),
    //                         //     transform: Transform::from_translation(mesh_translation),
    //                         //     ..Default::default()
    //                         // })
    //                         // .insert_bundle(WaterEffectBundle::default());
    //                     });
    //             });
    //     });
}

fn rotate_sprites(time: Res<Time>, mut query: Query<(&mut Transform, &RotationSpeed)>) {
    let delta = time.delta_seconds();

    for (mut transform, rot) in query.iter_mut() {
        transform.rotate_z(rot.0 * delta);
    }
}

// fn lets_panic(time: Res<Time>) {
//     if time.seconds_since_startup() > 0.2 {
//         panic!("lets panic");
//     }
// }