use bevy::{prelude::*, reflect::{TypeRegistry, serde::*}, scene::SceneInstanceReady};
use bevy_skein::SkeinPlugin;
use std::f32::consts::*;
use avian3d::prelude::*;

use std::f32::consts::TAU;

#[derive(Debug, Event)]
pub struct DroppedFile {
    pub name: String,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Player {
    name: String,
    power: f32,
    test: i32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Spin {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Lamp {
	r: f32,
	g: f32,
	b: f32,
	light: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct LampCol {
    _col: Color,
    r: f32,
}

#[derive(Component)]
struct MyCam;

#[test]
fn color_support() {
    let value = LampCol {
        _col: Color::srgba(1.0, 1.0, 1.0, 1.0),
        r: 0.0
    };

    let mut type_registry = TypeRegistry::new();
    type_registry.register::<LampCol>();
    let serializer = ReflectSerializer::new(&value, &type_registry);
    let json_string = serde_json::ser::to_string(&serializer).unwrap();

    assert_eq!(json_string, "");
}

fn main() {
    App::new()
        .register_type::<Player>()
        .register_type::<Spin>()
        .register_type::<Lamp>()
        .register_type::<LampCol>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .add_plugins((
            DefaultPlugins,
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            SkeinPlugin::default()
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            file_drop,
            added_lamp,
            move_player,
            update_cam,
            update_spin
        ))
        .add_observer(on_dropped)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        MyCam,
        Camera3d::default(),
        Transform::from_xyz(17.0, 10.0, 30.0)
            .looking_at(Vec3::new(5.0, 0.0, 0.0), Dir3::Y),
));
    /*commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(Color::WHITE))),
        Transform::default().with_scale(Vec3::splat(128.)),
    ));*/

    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("test.glb"),
    ))).observe(on_scene_ready);


    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::linear_rgb(1.0,1.0, 1.0),
        brightness: 50.0,
    });

}

fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut commands: Commands
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            println!("Dropped file with path: {:?}, in window id: {:?}", path_buf, window);
            commands.trigger(DroppedFile{ name: path_buf.to_str().unwrap_or("").to_string()});
        }
    }
}

fn move_player(
    time: Res<Time>,
    mut players: Query<&mut Transform, With<Player>>
){
    for mut t in players.iter_mut() {
        t.translation.x += time.delta_secs();
    }
}

fn on_dropped(
    trigger: Trigger<DroppedFile>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ev = trigger.event();
    println!("yop {:?}", ev.name);
    /*
    let texture_handle = textures.add(Texture {
        data: frame.data().to_vec(),
        dimension: bevy::render::texture::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        size: Extent3d {
            width,
            height,
            depth: 1,
        },
        ..Default::default()
    });
    let mesh_handle = meshes.add(Rectangle::from_size(Vec2::splat(256.0)));
    */
}

fn update_cam(
    mut cam: Query<&mut Transform, With<MyCam>>,
    time: Res<Time>
) {
    let secs = time.elapsed_secs_wrapped();
    for mut t in cam.iter_mut() {
        t.translation.y += (secs * 0.5).sin() * 0.0025;
    }
}


fn update_spin(
    mut spin: Query<(&mut Transform, &Spin)>,
    time: Res<Time>
) {
    let dt = time.delta_secs();
    for (mut t, s) in spin.iter_mut() {
        t.rotate_x(s.x * TAU * dt);
        t.rotate_y(s.y * TAU * dt);
        t.rotate_z(s.z * TAU * dt);
    }
}

fn added_lamp(
    lamp: Query<(Entity, &Parent, &Lamp), Added<Lamp>>,
    all: Query<&Transform>,
    mut commands: Commands
) {
    for (e, parent, lamp) in lamp.iter() {
        if let Ok(t) = all.get(parent.get()) {
            commands.spawn((
                PointLight {
                    intensity: lamp.light,
                    color: Color::linear_rgb(lamp.r,lamp.g, lamp.b),
                    shadows_enabled: true,
                    ..default()
                },
                Transform {
                    translation: t.translation.clone(),
                    ..default()
                }
            ));
        }
        commands.entity(e).despawn();
    }
}


fn on_scene_ready(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    lamps_query: Query<(&Parent, &Lamp)>,
    deets: Query<&Transform>,
) {
    let root = trigger.entity();
    for entity in children.iter_descendants(root) {
        if let Ok((p, lamp)) = lamps_query.get(entity) {
            if let Ok(transform) = deets.get(p.get()) {
                info!("Light onread: {} {:?}", lamp.light, transform);
            }

        }
    }
}
