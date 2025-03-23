use bevy::prelude::*;
use bevy_skein::SkeinPlugin;
use std::f32::consts::*;
use avian3d::prelude::*;


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

#[derive(Component)]
struct MyCam;

fn main() {
    App::new()
        .register_type::<Player>()
        .add_plugins((
            DefaultPlugins,
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            SkeinPlugin::default()
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (file_drop, move_player, update_cam))
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
    )));

    // Sun light
    commands.spawn((
/*        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
    },*/
        PointLight {
            intensity: 1_000_000.0,
            range: 500.0,
            radius: 20.0,
            color: Color::linear_rgb(1.0,0.9, 0.9),
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(5.0, 10.0, 0.0),
           // rotation: Quat::from_rotation_x(-PI / 2.0 +0.4),
            ..default()
        },
    ));

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
        t.translation.y += (secs * 2.0).sin() * 0.01;
    }
}
