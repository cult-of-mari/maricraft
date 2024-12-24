use self::physics::{CharacterControllerBundle, CharacterControllerPlugin};
use avian3d::math::*;
use avian3d::prelude::*;
use bevy::{
    asset::LoadedFolder,
    image::ImageSampler,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use leafwing_input_manager::prelude::*;
use std::collections::HashMap;

mod mesh;
mod physics;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum GameState {
    #[default]
    Setup,
    InGame,
}

#[derive(Debug, Resource)]
struct State {
    block: Handle<Mesh>,
    texture_map: HashMap<String, u32>,
}

#[derive(Resource, Default)]
struct TextureFolder(Handle<LoadedFolder>);

#[derive(Component)]
struct Block;

#[derive(Component)]
struct PlayerBody;

#[derive(Component)]
struct PlayerEye;

#[derive(Component)]
struct Hud;

#[derive(Component, Deref, DerefMut)]
pub struct WishDir(Vec2);

#[derive(Actionlike, Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
enum Action {
    #[actionlike(DualAxis)]
    Look,
    #[actionlike(DualAxis)]
    Move,
    Jump,
    Sneak,
    Sprint,
    Attack,
    Pick,
    Use,
}

impl Action {
    pub fn input_map() -> InputMap<Self> {
        InputMap::default()
            .with_dual_axis(Action::Look, MouseMove::default())
            .with_dual_axis(Action::Move, VirtualDPad::wasd())
            .with(Action::Jump, KeyCode::Space)
            .with(Action::Sneak, KeyCode::KeyC)
            .with(Action::Sprint, MouseButton::Other(4))
            .with(Action::Attack, MouseButton::Left)
            .with(Action::Pick, MouseButton::Middle)
            .with(Action::Use, MouseButton::Right)
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            InputManagerPlugin::<Action>::default(),
            MeshPickingPlugin,
            PhysicsPlugins::default(),
            //PhysicsDebugPlugin::default(),
            CharacterControllerPlugin,
            WireframePlugin,
        ))
        /*.insert_gizmo_config(
            PhysicsGizmos {
                aabb_color: Some(Color::WHITE),
                ..default()
            },
            GizmoConfig::default(),
        )*/
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        })
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Setup), setup)
        .add_systems(Update, loading.run_if(in_state(GameState::Setup)))
        .add_systems(OnExit(GameState::Setup), finalize)
        .add_systems(Update, update_hud.run_if(in_state(GameState::InGame)))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TextureFolder(asset_server.load_folder("textures")));
}

fn loading(
    mut next_state: ResMut<NextState<GameState>>,
    texture_folder: Res<TextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(GameState::InGame);
        }
    }
}

fn finalize(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    texture_folder: Res<TextureFolder>,
) {
    let texture_folder = loaded_folders.get(&texture_folder.0).unwrap();
    let mut builder = TextureAtlasBuilder::default();
    let mut texture_map = HashMap::new();
    let mut index = 0;

    for handle in texture_folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let path = handle
            .path()
            .unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy();

        let texture = images.get(id).unwrap();

        builder.add_texture(Some(id), texture);
        texture_map.insert(path.to_string(), index);

        index += 1;

        info!("loaded texture {path} into atlas at {index}");
    }

    let (_layout, _sources, mut image) = builder.build().unwrap();

    image.sampler = ImageSampler::nearest();
    let image_handle = images.add(image);

    let mesh = mesh::new_block(
        texture_map["grass_side.png"], // Front
        texture_map["grass_side.png"], // Back
        texture_map["grass_side.png"], // Right
        texture_map["grass_side.png"], // Left
        texture_map["grass_top.png"],  // Top
        texture_map["dirt.png"],       // Bottom
    );

    let state = State {
        block: meshes.add(mesh),
        texture_map,
    };

    for x in 0..=16 {
        for y in 0..=16 {
            commands
                .spawn((
                    Block,
                    Mesh3d(state.block.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(image_handle.clone()),
                        perceptual_roughness: 0.97,
                        reflectance: 0.1,
                        ..default()
                    })),
                    Transform::from_xyz(x as f32, -10.0, y as f32),
                    RigidBody::Static,
                    Collider::cuboid(1.0, 1.0, 1.0),
                ))
                .observe(|trigger: Trigger<Pointer<Over>>, mut commands: Commands| {
                    let entity = trigger.entity();

                    commands.entity(entity).insert(Wireframe);
                })
                .observe(|trigger: Trigger<Pointer<Out>>, mut commands: Commands| {
                    let entity = trigger.entity();

                    commands.entity(entity).remove::<Wireframe>();
                });
        }
    }

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
    ));

    commands
        .spawn((
            PlayerBody,
            WishDir(Vec2::ZERO),
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(1.0, 2.0, 1.0)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            })),
            InputManagerBundle::with_map(Action::input_map()),
            CharacterControllerBundle::new(Collider::capsule(0.5, 1.0), Vector::NEG_Y * 9.81 * 2.0)
                .with_movement(30.0, 0.92, 7.0, (30.0 as Scalar).to_radians()),
            Transform::default(),
        ))
        .with_children(|builder| {
            builder
                .spawn((PlayerEye, Visibility::default(), Transform::default()))
                .with_child((Camera3d::default(), Transform::from_xyz(0.0, 0.0, 10.0)));
        });

    let font = asset_server.load("fonts/RobotoMono-Regular.ttf");

    commands.spawn(Node::default()).with_children(|builder| {
        builder.spawn((
            Hud,
            Text::new("ok"),
            TextColor(Color::WHITE),
            TextFont { font, ..default() },
        ));
    });
}

fn update_hud(
    velocity: Single<&LinearVelocity, With<PlayerBody>>,
    player_body: Single<(&Transform, &WishDir), With<PlayerBody>>,
    mut text: Single<&mut Text, With<Hud>>,
) {
    let (transform, wish_dir) = player_body.into_inner();
    let (x, y, z) = transform.translation.into();
    let (vx, vy, vz) = (***velocity).into();
    let (yaw, pitch) = wish_dir.map(f32::to_degrees).into();

    ***text = format!("XYZ: {x:0.2}, {y:0.2}, {z:0.2}\nVEL: {vx:0.2}, {vy:0.2}, {vz:0.2}\n YP: {yaw:0.2}, {pitch:0.2}");
}
