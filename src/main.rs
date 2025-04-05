use bevy::{
    prelude::*,
    scene::SceneInstanceReady,
    core_pipeline::prepass::{ DepthPrepass, MotionVectorPrepass, NormalPrepass },
    render::{
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
        camera::ScalingMode
    },
};
use bevy_asset_loader::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_skein::SkeinPlugin;
use std::f32::consts::*;
use avian3d::prelude::*;

use std::f32::consts::TAU;

const PREPASS_SHADER_ASSET_PATH: &str = "shaders/show_prepass.wgsl";
const MATERIAL_SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

static mut VIEW:bool = false;

#[derive(Resource)]
struct GoalsReached {
    main_goal: bool,
    bonus: u32,
}

#[derive(Resource)]
struct Config {
    view: bool,
}

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

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct MyCam;


#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path="models/character.glb#Scene0")]
    player: Handle<Scene>,
    #[asset(path="models/building.glb#Scene0")]
    building: Handle<Scene>,
    #[asset(path="models/character.glb#Animation0")]
    anim0: Handle<AnimationClip>,
    #[asset(path="models/character.glb#Animation1")]
    anim1: Handle<AnimationClip>,
}

#[derive(Component)]
struct PlayerPlayer;

#[derive(Component)]
struct AnimationsToPlay {
    graph: Handle<AnimationGraph>,
    indices: Vec<AnimationNodeIndex>,
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

/// Not shown in this example, but if you need to specialize your material, the specialize
/// function will also be used by the prepass
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        MATERIAL_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    // You can override the default shaders used in the prepass if your material does
    // anything not supported by the default prepass
    // fn prepass_fragment_shader() -> ShaderRef {
    //     "shaders/custom_material.wgsl".into()
    // }
}

#[derive(Debug, Clone, Default, ShaderType)]
struct ShowPrepassSettings {
    show_depth: u32,
    show_normals: u32,
    show_motion_vectors: u32,
    padding_1: u32,
    padding_2: u32,
}

// This shader loads the prepass texture and outputs it directly
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct PrepassOutputMaterial {
    #[uniform(0)]
    settings: ShowPrepassSettings,
}

impl Material for PrepassOutputMaterial {
    fn fragment_shader() -> ShaderRef {
        PREPASS_SHADER_ASSET_PATH.into()
    }

    // This needs to be transparent in order to show the scene behind the mesh
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

fn main() {
    App::new()
        .register_type::<Player>()
        .register_type::<Spin>()
        .register_type::<Lamp>()
        .register_type::<MyCam>()
//        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            SkeinPlugin::default(),
            MaterialPlugin::<CustomMaterial>::default(),
            MaterialPlugin::<PrepassOutputMaterial> {
                // This material only needs to read the prepass textures,
                // but the meshes using it should not contribute to the prepass render, so we can disable it.
                prepass_enabled: false,
                ..default()
            },
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
            update_cam,
            update_spin,
            update_playa,
            toggle_prepass_view
        ).run_if(in_state(GameStates::Next)))
        .add_observer(on_dropped)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Res<PlayerAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut depth_materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    commands.insert_resource(Config { view: false });
    commands.spawn((
        MyCam,
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.0, // world units per pixel of window height.
            },
            ..OrthographicProjection::default_3d()
        }),
        //Bloom::default(),
        DepthPrepass,
        NormalPrepass,
        MotionVectorPrepass,
//        Transform::from_xyz(2.0, 5.0, 7.0)
//            .looking_at(Vec3::new(-2.0, 2.0, 0.0), Dir3::Y),
        Transform::from_xyz(7.0, 5.0, 7.0)
            .looking_at(Vec3::new(0.0, 3.0, 0.0), Dir3::Y),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("hdrs/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("hdrs/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 700.0,
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(5.0, 5.0))),
        MeshMaterial3d(depth_materials.add(PrepassOutputMaterial {
            settings: ShowPrepassSettings::default(),
        })),
        Transform::from_xyz(-3.0, 2.0, 0.0)
            //.looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    //    NotShadowCaster,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.0 -0.4),
            ..default()
        },
    ));


    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("test.glb"),
    ))).observe(on_scene_ready);

    commands.spawn((
        Name::new("Building"),
        SceneRoot(player.building.clone()),
        Transform::from_xyz(-12.25, 0.0, -2.1),
    ));

    commands.spawn((
        Name::new("Building"),
        SceneRoot(player.building.clone()),
        Transform::from_xyz(-12.25, 0.0, -6.1)
            .with_rotation(Quat::from_rotation_y(-PI*1.5)),
    ));

    commands.spawn((
        Name::new("Building"),
        SceneRoot(player.building.clone()),
        Transform::from_xyz(-12.25, 0.0, -12.1)
            .with_rotation(Quat::from_rotation_y(-PI/2.0)),
    ));


    // Anim for player
    let (graph, indices) =
        AnimationGraph::from_clips([
            player.anim0.clone(),
            player.anim1.clone(),
        ]);
    let graph_handle = graphs.add(graph);

    commands.spawn((
        Name::new("APlayer"),
        SceneRoot(player.player.clone()),
        Transform::from_xyz(0.0, -0.1, 0.0),
        Playa,
        AnimationsToPlay {
            graph: graph_handle,
            indices
        }
    )).observe(
        |trigger: Trigger<SceneInstanceReady>,
        mut cmds: Commands,
        children: Query<&Children>,
        animations_to_play: Query<&AnimationsToPlay>,
        mut players: Query<(Entity, &mut AnimationPlayer)>,
        | {
            let Ok(animations) = animations_to_play.get(trigger.target()) else {
                info!("no anims in player");
                return;
            };

            for child in children.iter_descendants(trigger.target()) {
                if let Ok((pe, mut player)) = players.get_mut(child) {
                    player.play(animations.indices[1]).repeat();
                    cmds.entity(pe).insert(PlayerPlayer);
                    // Link graph to mesh
                    cmds
                        .entity(child)
                        .insert(AnimationGraphHandle(animations.graph.clone()));
                }
            }
        });

    // Ambient light
/*    commands.insert_resource(AmbientLight {
        color: Color::linear_rgb(1.0,1.0, 1.0),
        brightness: 50.0,
    }); */

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

fn on_dropped(
    trigger: Trigger<DroppedFile>,
    mut _commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<ColorMaterial>>,
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
    keycode: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<Config>,
    time: Res<Time>
) {
    let secs = time.elapsed_secs_wrapped();
    for mut t in cam.iter_mut() {
        t.translation.y += (secs * 0.5).sin() * 0.0025;
        if keycode.just_pressed(KeyCode::Digit1) {
            config.view = false;
            *t = Transform::from_xyz(-7.0, 1.0, 7.0)
                .looking_at(Vec3::new(-7.0, 1.0, 0.0), Dir3::Y);
        }
        if keycode.just_pressed(KeyCode::Digit2) {
            config.view = true;
            *t = Transform::from_xyz(7.0, 1.0, -6.5)
                .looking_at(Vec3::new(0.0, 1.0, -6.5), Dir3::Y);
        }
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
    lamps_query: Query<(&ChildOf, &Lamp)>,
    // camera_query: Query<(&Parent, &MyCam)>,
    deets: Query<&Transform>,
    mut commands: Commands,
) {
    let root = trigger.target();
    for child in children.iter_descendants(root) {
        if let Ok((p, lamp)) = lamps_query.get(child) {
            if let Ok(transform) = deets.get(p.parent) {
                info!("Light onread: {} {:?}", lamp.light, transform);
                commands.spawn((
                    PointLight {
                        intensity: lamp.light * 2.0,
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
            commands.entity(child).despawn();

        }
    }
}

fn update_playa(
    mut player: Query<(Entity, &mut Transform), With<Playa>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    config: Res<Config>,
    animations_to_play: Query<&AnimationsToPlay>,
    mut players: Query<&mut AnimationPlayer, With<PlayerPlayer>>,
) {

    for (e, mut t) in player.iter_mut() {
        let Ok(animations) = animations_to_play.get(e) else {
            info!("no anim");
            continue;
        };
        let Ok(mut anim_player) = players.single_mut() else {
            info!("no player");
            continue;
        };

        let power = 2.0;
        let anim_speed = 1.5;
        let mut v = Vec2::new(0.0, 0.0);
        if config.view {
            if input.pressed(KeyCode::KeyW) {
                v.x -= power;
                t.rotation = Quat::from_rotation_y(-PI * 0.5);
            }
            if input.pressed(KeyCode::KeyS) {
                v.x += power;
                t.rotation = Quat::from_rotation_y(PI * 0.5);
            }
            if input.pressed(KeyCode::KeyA) {
                v.y += power;
                t.rotation = Quat::from_rotation_y(0.0);
            }
            if input.pressed(KeyCode::KeyD) {
                v.y -= power;
                t.rotation = Quat::from_rotation_y(PI);
            }
        } else {
            if input.pressed(KeyCode::KeyW) {
                v.y -= power;
                t.rotation = Quat::from_rotation_y(-PI);
            }
            if input.pressed(KeyCode::KeyS) {
                v.y += power;
                t.rotation = Quat::from_rotation_y(0.0);
            }
            if input.pressed(KeyCode::KeyA) {
                v.x -= power;
                t.rotation = Quat::from_rotation_y( -PI / 2.0);
            }
            if input.pressed(KeyCode::KeyD) {
                v.x += power;
                t.rotation = Quat::from_rotation_y(PI / 2.0);
            }
        }
        if v.length() == 0.0 {
            anim_player.stop(animations.indices[0]);
            anim_player.play(animations.indices[1]).repeat().set_speed(anim_speed);
        } else {
            anim_player.stop(animations.indices[1]);
            anim_player.play(animations.indices[0]).repeat().set_speed(anim_speed);
        }

        t.translation.x += v.x * time.delta_secs();
        if t.translation.x > 1.0 {
            t.translation.x = 1.0;
        }
        t.translation.z += v.y * time.delta_secs();
        if t.translation.z > 1.0 {
            t.translation.z = 1.0;
        }
    }
}

fn toggle_prepass_view(
    mut prepass_view: Local<u32>,
    keycode: Res<ButtonInput<KeyCode>>,
    material_handle: Single<&MeshMaterial3d<PrepassOutputMaterial>>,
    mut materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    if keycode.just_pressed(KeyCode::Space) {
        *prepass_view = (*prepass_view + 1) % 4;
        let mat = materials.get_mut(*material_handle).unwrap();
        mat.settings.show_depth = (*prepass_view == 1) as u32;
        mat.settings.show_normals = (*prepass_view == 2) as u32;
        mat.settings.show_motion_vectors = (*prepass_view == 3) as u32;
    }
}
