use super::input::CursorPos;
use crate::{
    Color, GamePiece, Piece, GRID_SIZE, MAP_SIZE, MAP_TYPE, SCALE, SCALED_GRID_SIZE, TILE_SIZE,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct CursorDisplay;

pub fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(MAP_SIZE);
    let tilemap_transform =
        get_tilemap_center_transform(&MAP_SIZE, &SCALED_GRID_SIZE, &MAP_TYPE, 0.0)
            * Transform::from_scale(Vec3::splat(SCALE));
    // Get the offset of the board from the center
    let tilemat_offset = Vec2 {
        x: tilemap_transform.translation.x,
        y: tilemap_transform.translation.y,
    };

    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_center =
                tile_pos.center_in_world(&SCALED_GRID_SIZE, &MAP_TYPE) + tilemat_offset;
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        // Create a checkerboard pattern by selecting the light or dark tile depending
                        // on whether the sum of its coordinates is even or odd.
                        texture_index: TileTextureIndex((x + y) % 2),
                        tilemap_id: TilemapId(tilemap_entity),
                        ..Default::default()
                    },
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3 {
                                x: tile_center.x,
                                y: tile_center.y,
                                z: 1.0,
                            },
                            scale: Vec3::splat(SCALE),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                ))
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
        transform: tilemap_transform,
        ..Default::default()
    });
}

pub fn setup_cursor(mut commands: Commands, cursor_pos: Res<CursorPos>) {
    commands.spawn((
        CursorDisplay,
        SpriteBundle {
            transform: Transform {
                translation: Vec3 {
                    x: cursor_pos.0.x,
                    y: cursor_pos.0.y,
                    z: 1.0,
                },
                scale: Vec3::splat(SCALE),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

fn insert_piece(
    game_piece: GamePiece,
    commands: &mut Commands,
    tile_id: Entity,
    handle: &mut Handle<Image>,
    visibility: &mut Visibility,
    asset_server: &Res<AssetServer>,
) {
    // Must call to_string() to create an owned copy of the string, as the &str lifetime doesn't
    // last long enough.
    let asset_path = game_piece.get_asset_path().to_string();
    commands.entity(tile_id).insert(GamePiece {
        piece: game_piece.piece,
        color: game_piece.color,
    });
    *handle = asset_server.load(asset_path);
    *visibility = Visibility::Visible;
}

pub fn setup_pieces(
    mut commands: Commands,
    mut tile_q: Query<(Entity, &TilePos, &mut Handle<Image>, &mut Visibility)>,
    asset_server: Res<AssetServer>,
) {
    // Place the starting pieces on the board.
    for (tile_id, tile_pos, mut handle, mut visibility) in &mut tile_q {
        match tile_pos {
            TilePos { x: 0 | 7, y: 0 } => insert_piece(
                GamePiece {
                    piece: Piece::Rook,
                    color: Color::White,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 1 | 6, y: 0 } => insert_piece(
                GamePiece {
                    piece: Piece::Knight,
                    color: Color::White,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 2 | 5, y: 0 } => insert_piece(
                GamePiece {
                    piece: Piece::Bishop,
                    color: Color::White,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 3, y: 0 } => insert_piece(
                GamePiece {
                    piece: Piece::Queen,
                    color: Color::White,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 4, y: 0 } => insert_piece(
                GamePiece {
                    piece: Piece::King,
                    color: Color::White,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 0..8, y: 1 } => insert_piece(
                GamePiece {
                    piece: Piece::Pawn,
                    color: Color::White,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 0..8, y: 6 } => insert_piece(
                GamePiece {
                    piece: Piece::Pawn,
                    color: Color::Black,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 0 | 7, y: 7 } => insert_piece(
                GamePiece {
                    piece: Piece::Rook,
                    color: Color::Black,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 1 | 6, y: 7 } => insert_piece(
                GamePiece {
                    piece: Piece::Knight,
                    color: Color::Black,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 2 | 5, y: 7 } => insert_piece(
                GamePiece {
                    piece: Piece::Bishop,
                    color: Color::Black,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 3, y: 7 } => insert_piece(
                GamePiece {
                    piece: Piece::Queen,
                    color: Color::Black,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            TilePos { x: 4, y: 7 } => insert_piece(
                GamePiece {
                    piece: Piece::King,
                    color: Color::Black,
                },
                &mut commands,
                tile_id,
                &mut handle,
                &mut visibility,
                &asset_server,
            ),
            _ => (),
        }
    }
}
