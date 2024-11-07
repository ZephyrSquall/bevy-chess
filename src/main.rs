use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

const SCALE: f32 = 5.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            // Prevent anti-aliasing
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(TilemapPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, update_cursor)
    .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize { x: 8, y: 8 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
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

    let tile_size = TilemapTileSize { x: 10.0, y: 10.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0)
            .with_scale(Vec3::splat(SCALE)),
        ..Default::default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/king_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 100.0,
                y: 100.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/king_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 100.0,
                y: 0.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/pawn_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 200.0,
                y: 100.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/pawn_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 250.0,
                y: 100.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/knight_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 250.0,
                y: 260.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/knight_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 250.0,
                y: 200.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/rook_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: -250.0,
                y: -200.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/rook_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: -200.0,
                y: -200.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/bishop_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 200.0,
                y: -200.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/bishop_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 200.0,
                y: -250.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/queen_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: -200.0,
                y: 200.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/queen_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: -200.0,
                y: 250.0,
                z: 0.0,
            },
            scale: Vec3::splat(SCALE),
            ..default()
        },
        ..default()
    });
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
