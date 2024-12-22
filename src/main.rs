use bevy::{
    asset::{LoadedFolder, RenderAssetUsages},
    image::ImageSampler,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum GameState {
    #[default]
    Setup,
    InGame,
}

#[derive(Debug, Resource)]
struct State {
    block: Handle<Mesh>,
}

#[derive(Resource, Default)]
struct TextureFolder(Handle<LoadedFolder>);

#[derive(Component)]
struct Block;

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
            MeshPickingPlugin,
            WireframePlugin,
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        })
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Setup), setup)
        .add_systems(Update, loading.run_if(in_state(GameState::Setup)))
        .add_systems(OnExit(GameState::Setup), finalize)
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
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    texture_folder: Res<TextureFolder>,
) {
    let texture_folder = loaded_folders.get(&texture_folder.0).unwrap();
    let mut builder = TextureAtlasBuilder::default();

    for handle in texture_folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let texture = images.get(id).unwrap();

        info!("{:?}", handle.path());

        builder.add_texture(Some(id), texture);
    }

    let (_layout, _sources, mut image) = builder.build().unwrap();

    image.sampler = ImageSampler::nearest();
    let image_handle = images.add(image);

    let mesh = new_block_mesh(
        1, // Front
        1, // Back
        1, // Right
        1, // Left
        2, // Top
        0, // Bottom
    );

    let state = State {
        block: meshes.add(mesh),
    };

    for x in -5..5 {
        for y in -5..5 {
            commands
                .spawn((
                    Block,
                    Name::new("block"),
                    Mesh3d(state.block.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(image_handle.clone()),
                        perceptual_roughness: 0.97,
                        reflectance: 0.1,
                        ..default()
                    })),
                    Transform::from_xyz((x as f32) * 5.0, -7.0, (y as f32) * 5.0)
                        .with_scale(Vec3::splat(5.0)),
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

    commands.spawn((
        Name::new("camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 50.0),
    ));
}

fn new_block_uv(index: u32) -> [Vec2; 2] {
    let position = Vec2::new(index as f32, 0.0);

    let min = position / Vec2::splat(16.0);
    let max = (position + Vec2::ONE) / Vec2::splat(16.0);

    [min, max]
}

fn new_block_mesh(front: u32, back: u32, right: u32, left: u32, top: u32, bottom: u32) -> Mesh {
    let [front_min, front_max] = new_block_uv(front);
    let [back_min, back_max] = new_block_uv(back);
    let [right_min, right_max] = new_block_uv(right);
    let [left_min, left_max] = new_block_uv(left);
    let [top_min, top_max] = new_block_uv(top);
    let [bottom_min, bottom_max] = new_block_uv(bottom);

    let min = -0.5;
    let max = 0.5;

    let vertices = &[
        // Front
        ([min, min, max], [0.0, 0.0, 1.0], [front_min.x, front_max.y]),
        ([max, min, max], [0.0, 0.0, 1.0], [front_max.x, front_max.y]),
        ([max, max, max], [0.0, 0.0, 1.0], [front_max.x, front_min.y]),
        ([min, max, max], [0.0, 0.0, 1.0], [front_min.x, front_min.y]),
        // Back
        ([min, max, min], [0.0, 0.0, -1.0], [back_min.x, back_max.y]),
        ([max, max, min], [0.0, 0.0, -1.0], [back_max.x, back_max.y]),
        ([max, min, min], [0.0, 0.0, -1.0], [back_max.x, back_min.y]),
        ([min, min, min], [0.0, 0.0, -1.0], [back_min.x, back_min.y]),
        // Right
        ([max, min, min], [1.0, 0.0, 0.0], [right_max.x, right_max.y]),
        ([max, max, min], [1.0, 0.0, 0.0], [right_max.x, right_min.y]),
        ([max, max, max], [1.0, 0.0, 0.0], [right_min.x, right_min.y]),
        ([max, min, max], [1.0, 0.0, 0.0], [right_min.x, right_max.y]),
        // Left
        ([min, min, max], [-1.0, 0.0, 0.0], [left_max.x, left_max.y]),
        ([min, max, max], [-1.0, 0.0, 0.0], [left_max.x, left_min.y]),
        ([min, max, min], [-1.0, 0.0, 0.0], [left_min.x, left_min.y]),
        ([min, min, min], [-1.0, 0.0, 0.0], [left_min.x, left_max.y]),
        // Top
        ([max, max, min], [0.0, 1.0, 0.0], [top_max.x, top_min.y]),
        ([min, max, min], [0.0, 1.0, 0.0], [top_min.x, top_min.y]),
        ([min, max, max], [0.0, 1.0, 0.0], [top_min.x, top_max.y]),
        ([max, max, max], [0.0, 1.0, 0.0], [top_max.x, top_max.y]),
        // Bottom
        (
            [max, min, max],
            [0.0, -1.0, 0.0],
            [bottom_max.x, bottom_max.y],
        ),
        (
            [min, min, max],
            [0.0, -1.0, 0.0],
            [bottom_max.x, bottom_min.y],
        ),
        (
            [min, min, min],
            [0.0, -1.0, 0.0],
            [bottom_min.x, bottom_min.y],
        ),
        (
            [max, min, min],
            [0.0, -1.0, 0.0],
            [bottom_min.x, bottom_max.y],
        ),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Right
        12, 13, 14, 14, 15, 12, // Left
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20, // Bottom
    ]);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(indices)
}
