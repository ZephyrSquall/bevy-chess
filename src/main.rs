use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

const MAP_SIZE: TilemapSize = TilemapSize { x: 8, y: 8 };
const MAP_TYPE: TilemapType = TilemapType::Square;
const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 10.0, y: 10.0 };
const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 10.0, y: 10.0 };
const SCALE: f32 = 8.0;

// Used to properly space sprites on the grid after it is scaled up.
const SCALED_GRID_SIZE: TilemapGridSize = TilemapGridSize {
    x: GRID_SIZE.x * SCALE,
    y: GRID_SIZE.y * SCALE,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            // Prevent anti-aliasing
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(TilemapPlugin)
    .add_systems(Startup, (setup_board, setup_pieces).chain())
    .add_systems(Update, update_cursor)
    .run();
}

fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(MAP_SIZE);

    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    // Create a checkerboard pattern by selecting the light or dark tile depending
                    // on whether the sum of its coordinates is even or odd.
                    texture_index: TileTextureIndex((x + y) % 2),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: GRID_SIZE,
        map_type: MAP_TYPE,
        size: MAP_SIZE,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TILE_SIZE,
        transform: get_tilemap_center_transform(&MAP_SIZE, &SCALED_GRID_SIZE, &MAP_TYPE, 0.0)
            * Transform::from_scale(Vec3::splat(SCALE)),
        ..Default::default()
    });
}

fn setup_pieces(
    mut commands: Commands,
    tile_q: Query<(Entity, &TilePos)>,
    tilemap_transform_q: Query<&Transform, With<TileStorage>>,
    asset_server: Res<AssetServer>,
) {
    // Get the offset of the board from the center
    let mut tilemat_offset = Vec2 { x: 0.0, y: 0.0 };
    for transform in &tilemap_transform_q {
        tilemat_offset.x = transform.translation.x;
        tilemat_offset.y = transform.translation.y;
    }

    for (tile_id, tile_pos) in &tile_q {
        let center = tile_pos.center_in_world(&SCALED_GRID_SIZE, &MAP_TYPE) + tilemat_offset;

        commands.entity(tile_id).insert(SpriteBundle {
            texture: asset_server.load("pieces/king_white.png"),
            transform: Transform {
                translation: Vec3 {
                    x: center.x,
                    y: center.y,
                    z: 1.0,
                },
                scale: Vec3::splat(SCALE),
                ..default()
            },
            ..default()
        });
    }
}

fn update_cursor(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera_q.single();

        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            commands.spawn(SpriteBundle {
                texture: asset_server.load("pieces/king_white.png"),
                transform: Transform {
                    translation: Vec3 {
                        x: world_position.x,
                        y: world_position.y,
                        z: 0.0,
                    },
                    scale: Vec3::splat(SCALE),
                    ..default()
                },
                ..default()
            });
        }
    }
}
