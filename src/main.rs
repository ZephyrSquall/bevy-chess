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

enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

enum Color {
    White,
    Black,
}

#[derive(Component)]
struct GamePiece {
    piece: Piece,
    color: Color,
}

impl GamePiece {
    fn get_asset_path(&self) -> &str {
        match (&self.piece, &self.color) {
            (Piece::Bishop, Color::White) => "pieces/bishop_white.png",
            (Piece::King, Color::White) => "pieces/king_white.png",
            (Piece::Knight, Color::White) => "pieces/knight_white.png",
            (Piece::Pawn, Color::White) => "pieces/pawn_white.png",
            (Piece::Queen, Color::White) => "pieces/queen_white.png",
            (Piece::Rook, Color::White) => "pieces/rook_white.png",
            (Piece::Bishop, Color::Black) => "pieces/bishop_black.png",
            (Piece::King, Color::Black) => "pieces/king_black.png",
            (Piece::Knight, Color::Black) => "pieces/knight_black.png",
            (Piece::Pawn, Color::Black) => "pieces/pawn_black.png",
            (Piece::Queen, Color::Black) => "pieces/queen_black.png",
            (Piece::Rook, Color::Black) => "pieces/rook_black.png",
        }
    }
}

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
    tilemap_q: Query<(&Transform, &TileStorage)>,
    asset_server: Res<AssetServer>,
) {
    let mut tilemat_offset = Vec2 { x: 0.0, y: 0.0 };
    for (transform, tile_storage) in &tilemap_q {
        // Get the offset of the board from the center
        tilemat_offset.x = transform.translation.x;
        tilemat_offset.y = transform.translation.y;

        // Place the starting pieces on the board.
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Rook,
                color: Color::White,
            },
            TilePos { x: 0, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Knight,
                color: Color::White,
            },
            TilePos { x: 1, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Bishop,
                color: Color::White,
            },
            TilePos { x: 2, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Queen,
                color: Color::White,
            },
            TilePos { x: 3, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::King,
                color: Color::White,
            },
            TilePos { x: 4, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Bishop,
                color: Color::White,
            },
            TilePos { x: 5, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Knight,
                color: Color::White,
            },
            TilePos { x: 6, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Rook,
                color: Color::White,
            },
            TilePos { x: 7, y: 0 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );

        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 0, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 1, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 2, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 3, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 4, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 5, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 6, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::White,
            },
            TilePos { x: 7, y: 1 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );

        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 0, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 1, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 2, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 3, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 4, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 5, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 6, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Pawn,
                color: Color::Black,
            },
            TilePos { x: 7, y: 6 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );

        insert_piece_at_position(
            GamePiece {
                piece: Piece::Rook,
                color: Color::Black,
            },
            TilePos { x: 0, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Knight,
                color: Color::Black,
            },
            TilePos { x: 1, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Bishop,
                color: Color::Black,
            },
            TilePos { x: 2, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Queen,
                color: Color::Black,
            },
            TilePos { x: 3, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::King,
                color: Color::Black,
            },
            TilePos { x: 4, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Bishop,
                color: Color::Black,
            },
            TilePos { x: 5, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Knight,
                color: Color::Black,
            },
            TilePos { x: 6, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
        insert_piece_at_position(
            GamePiece {
                piece: Piece::Rook,
                color: Color::Black,
            },
            TilePos { x: 7, y: 7 },
            &mut commands,
            tile_storage,
            tilemat_offset,
            &asset_server,
        );
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

fn insert_piece_at_position(
    game_piece: GamePiece,
    tile_pos: TilePos,
    commands: &mut Commands,
    tile_storage: &TileStorage,
    tilemat_offset: Vec2,
    asset_server: &Res<AssetServer>,
) {
    // Must call to_string() to create an owned copy of the string, as the &str lifetime doesn't
    // last long enough.
    let asset_path = game_piece.get_asset_path().to_string();
    let center = tile_pos.center_in_world(&SCALED_GRID_SIZE, &MAP_TYPE) + tilemat_offset;

    let tile_id = tile_storage
        .get(&tile_pos)
        .expect("All board positions should have a tile entity");

    commands.entity(tile_id).insert(SpriteBundle {
        texture: asset_server.load(asset_path),
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
