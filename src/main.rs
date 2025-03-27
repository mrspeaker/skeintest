use bevy::{prelude::*, reflect::{TypeRegistry, serde::*}, scene::SceneInstanceReady};
use bevy_asset_loader::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_skein::SkeinPlugin;
use std::f32::consts::*;
use avian3d::prelude::*;

use std::f32::consts::TAU;

#[derive(Component)]
struct Playa;

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
	light: f32,
    col: Color
}

#[derive(Component)]
struct MyCam;


#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path="models/anim.glb#Scene0")]
    player: Handle<Scene>,
    #[asset(path="models/anim.glb#Animation1")]
    anim0: Handle<AnimationClip>,
}

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn main() {
    App::new()
        .register_type::<Player>()
        .register_type::<Spin>()
        .register_type::<Lamp>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .add_plugins((
            DefaultPlugins,
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            SkeinPlugin::default()
        ))
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Next)
                .load_collection::<PlayerAssets>(),
        )
        .add_systems(OnEnter(GameStates::Next), setup)
        .add_systems(Update, (
            file_drop,
            move_player,
            update_cam,
            update_spin,
            update_playa
        ))
        .add_observer(on_dropped)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Res<PlayerAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn((
        MyCam,
        Camera3d::default(),
        Transform::from_xyz(17.0, 10.0, 30.0)
            .looking_at(Vec3::new(5.0, 0.0, 0.0), Dir3::Y),
    ));


    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("test.glb"),
    ))).observe(on_scene_ready);


    // Anim for player
    let (graph, node_index) =
        AnimationGraph::from_clip(player.anim0.clone());
    let graph_handle = graphs.add(graph);

    commands.spawn((
        Name::new("APlayer"),
        SceneRoot(player.player.clone()),
        Transform::from_xyz(5.0, 0.0, 2.0),
        Playa,
        AnimationToPlay {
            graph_handle,
            index: node_index
        }
    )).observe(
        |trigger: Trigger<SceneInstanceReady>,
        mut cmds: Commands,
        children: Query<&Children>,
        animations_to_play: Query<&AnimationToPlay>,
        mut players: Query<&mut AnimationPlayer>,
        | {
            let Ok(animation_to_play) = animations_to_play.get(trigger.entity()) else {
                return;
            };

            for child in children.iter_descendants(trigger.entity()) {
                if let Ok(mut player) = players.get_mut(child) {
                    player.play(animation_to_play.index).repeat();
                    // Link graph to mesh
                    cmds
                        .entity(child)
                        .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
                }
            }
        });

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

fn on_scene_ready(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    lamps_query: Query<(&Parent, &Lamp)>,
    deets: Query<&Transform>,
    mut commands: Commands,
) {
    let root = trigger.entity();
    for child in children.iter_descendants(root) {
        if let Ok((p, lamp)) = lamps_query.get(child) {
            if let Ok(transform) = deets.get(p.get()) {
                info!("Light onread: {} {:?}", lamp.light, transform);
                commands.spawn((
                    PointLight {
                        intensity: lamp.light,
                        color: lamp.col,
                        shadows_enabled: true,
                        ..default()
                    },
                    Transform {
                        translation: transform.translation.clone(),
                        ..default()
                    }
                ));

            }
            commands.entity(child).despawn_recursive();

        }
    }
}

fn update_playa(
    mut player: Query<&mut Transform, With<Playa>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,

) {
    for mut t in player.iter_mut() {
        let power = 3.0;
        let mut v = Vec2::new(0.0, 0.0);
        if input.pressed(KeyCode::KeyW) {
            v.y -= power;
        }
        if input.pressed(KeyCode::KeyS) {
            v.y += power;
        }
        if input.pressed(KeyCode::KeyA) {
            v.x -= power;
        }
        if input.pressed(KeyCode::KeyD) {
            v.x += power;
        }

        t.translation.x += v.x * time.delta_secs();
        t.translation.z += v.y * time.delta_secs();
    }
}
