use crate::components::{
    Color, CursorDisplay, GamePiece, LegalMove, LegalMoves, MouseoverHighlight, Piece,
};
use crate::resources::{
    ColorToMove, CursorPos, MustRecalculateLegalMoves, RightToCastle, SelectedPiece,
    SelectedPieceOriginalPosition,
};
use crate::{MAP_SIZE, MAP_TYPE, SCALED_GRID_SIZE};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn find_mouseover_tile(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(&Transform, &TileStorage)>,
    tile_q: Query<Entity, With<MouseoverHighlight>>,
) {
    // Remove MouseoverHighlight component for any tile that has it. It will be re-added to tiles
    // that still need it later.
    for tile_id in &tile_q {
        commands.entity(tile_id).remove::<MouseoverHighlight>();
    }

    let mut cursor_with_offset = cursor_pos.0;
    for (transform, tile_storage) in &tilemap_q {
        // Apply the opposite translation that the board has experienced to the cursor position so
        // the cursor lines up with an untranslated board.
        cursor_with_offset.x -= transform.translation.x;
        cursor_with_offset.y -= transform.translation.y;

        // Check if there is a tile at that position
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_with_offset, &MAP_SIZE, &SCALED_GRID_SIZE, &MAP_TYPE)
        {
            if let Some(tile_id) = tile_storage.get(&tile_pos) {
                commands.entity(tile_id).insert(MouseoverHighlight());
            }
        }
    }
}

pub fn pick_up_piece(
    mut selected_piece: ResMut<SelectedPiece>,
    mut selected_piece_original_position: ResMut<SelectedPieceOriginalPosition>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    // Must include "Without<CursorDisplay>" to create a disjoint query that doesn't mutably access
    // the "Visibility" component of an entity twice.
    mut tile_q: Query<
        (
            &TilePos,
            &mut Visibility,
            Option<&GamePiece>,
            Option<&LegalMoves>,
        ),
        (With<MouseoverHighlight>, Without<CursorDisplay>),
    >,
    mut cursor_q: Query<(&mut Handle<Image>, &mut Visibility), With<CursorDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if mouse.just_pressed(MouseButton::Left) && selected_piece.0.is_none() {
        for (tile_pos, mut visibility, game_piece, legal_moves) in &mut tile_q {
            if let Some(game_piece) = game_piece {
                if let Some(legal_moves) = legal_moves {
                    // Do not pick up the piece if it has no legal moves.
                    if !legal_moves.0.is_empty() {
                        // Get game piece from current tile, and hide it on the tile while it is carried.
                        *selected_piece = SelectedPiece(Some(*game_piece));
                        *visibility = Visibility::Hidden;

                        // Display the game piece on the cursor.
                        let (mut cursor_handle, mut cursor_visibility) = cursor_q.single_mut();
                        *cursor_handle = asset_server.load(game_piece.get_asset_path().to_string());
                        *cursor_visibility = Visibility::Visible;

                        // Track the piece's original position.
                        *selected_piece_original_position =
                            SelectedPieceOriginalPosition(Some(*tile_pos));

                        mouse.clear_just_pressed(MouseButton::Left);
                    }
                }
            }
        }
    }
}

pub fn put_down_piece(
    mut commands: Commands,
    mut selected_piece: ResMut<SelectedPiece>,
    mut selected_piece_original_position: ResMut<SelectedPieceOriginalPosition>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut color_to_move: ResMut<ColorToMove>,
    mut must_recalculate_legal_moves: ResMut<MustRecalculateLegalMoves>,
    mut right_to_castle: ResMut<RightToCastle>,
    mut tile_q: Query<
        (Entity, &TilePos, &mut Handle<Image>, &mut Visibility),
        (With<MouseoverHighlight>, Without<CursorDisplay>),
    >,
    tile_legal_moves_q: Query<&LegalMoves>,
    mut tile_sprite_q: Query<
        (&mut Handle<Image>, &mut Visibility),
        (Without<MouseoverHighlight>, Without<CursorDisplay>),
    >,
    tilemap_q: Query<&TileStorage>,
    mut cursor_q: Query<&mut Visibility, With<CursorDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        // Get the currently-selected piece, or skip if no piece is selected.
        if let Some(selected_piece_some) = &selected_piece.0 {
            // Get the position of the original tile of the currently-selected piece (this shouldn't
            // be None if there is a selected piece).
            if let Some(selected_piece_original_position_some) = &selected_piece_original_position.0
            {
                // Get the entity id of the original tile of the currently-selected piece (this
                // shouldn't be None if there is a selected piece).
                let tilemap = tilemap_q.single();
                if let Some(selected_piece_original_tile) =
                    tilemap.get(selected_piece_original_position_some)
                {
                    // Get the current mouseover tile, or skip if the cursor is not over a tile.
                    if let Ok((tile_id, tile_pos, mut handle, mut visibility)) =
                        tile_q.get_single_mut()
                    {
                        // Get legal moves from the original tile, or skip if the original tile had
                        // no legal moves.
                        if let Ok(legal_moves) =
                            tile_legal_moves_q.get(selected_piece_original_tile)
                        {
                            // Check that the current mouseover tile is a legal move from the
                            // original tile, otherwise skip.
                            if let Some(legal_move) = legal_moves
                                .0
                                .iter()
                                .find(|legal_move| legal_move.destination == *tile_pos)
                            {
                                // All checks have been made to make sure this is a valid move. All
                                // code that follows is handling this move.

                                // Remove the GamePiece on the original tile (do this before adding
                                // the GamePiece from the cursor in case it's on the same tile).
                                commands
                                    .entity(selected_piece_original_tile)
                                    .remove::<GamePiece>();

                                // Add the GamePiece on the tile that the cursor is currently hovering over.
                                commands.entity(tile_id).insert(*selected_piece_some);
                                *handle = asset_server
                                    .load(selected_piece_some.get_asset_path().to_string());
                                *visibility = Visibility::Visible;

                                // If the move was castling, move the rook too.
                                if legal_move.is_castling {
                                    match tile_pos {
                                        TilePos { x: 2, y: 0 } => {
                                            let rook_starting_tile_id = tilemap
                                                .get(&TilePos { x: 0, y: 0 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            let rook_ending_tile_id = tilemap
                                                .get(&TilePos { x: 3, y: 0 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            commands
                                                .entity(rook_starting_tile_id)
                                                .remove::<GamePiece>();
                                            commands.entity(rook_ending_tile_id).insert(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::White,
                                                },
                                            );

                                            let (_, mut rook_starting_tile_visibility) =
                                                tile_sprite_q
                                                    .get_mut(rook_starting_tile_id)
                                                    .expect(
                                                        "Tile literal should exist on 8x8 board",
                                                    );
                                            *rook_starting_tile_visibility = Visibility::Hidden;

                                            let (
                                                mut rook_ending_tile_handle,
                                                mut rook_ending_tile_visibility,
                                            ) = tile_sprite_q
                                                .get_mut(rook_ending_tile_id)
                                                .expect("Tile literal should exist on 8x8 board");
                                            *rook_ending_tile_handle = asset_server.load(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::White,
                                                }
                                                .get_asset_path()
                                                .to_string(),
                                            );
                                            *rook_ending_tile_visibility = Visibility::Visible;
                                        }
                                        TilePos { x: 6, y: 0 } => {
                                            let rook_starting_tile_id = tilemap
                                                .get(&TilePos { x: 7, y: 0 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            let rook_ending_tile_id = tilemap
                                                .get(&TilePos { x: 5, y: 0 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            commands
                                                .entity(rook_starting_tile_id)
                                                .remove::<GamePiece>();
                                            commands.entity(rook_ending_tile_id).insert(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::White,
                                                },
                                            );

                                            let (_, mut rook_starting_tile_visibility) =
                                                tile_sprite_q
                                                    .get_mut(rook_starting_tile_id)
                                                    .expect(
                                                        "Tile literal should exist on 8x8 board",
                                                    );
                                            *rook_starting_tile_visibility = Visibility::Hidden;

                                            let (
                                                mut rook_ending_tile_handle,
                                                mut rook_ending_tile_visibility,
                                            ) = tile_sprite_q
                                                .get_mut(rook_ending_tile_id)
                                                .expect("Tile literal should exist on 8x8 board");
                                            *rook_ending_tile_handle = asset_server.load(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::White,
                                                }
                                                .get_asset_path()
                                                .to_string(),
                                            );
                                            *rook_ending_tile_visibility = Visibility::Visible;
                                        }
                                        TilePos { x: 2, y: 7 } => {
                                            let rook_starting_tile_id = tilemap
                                                .get(&TilePos { x: 0, y: 7 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            let rook_ending_tile_id = tilemap
                                                .get(&TilePos { x: 3, y: 7 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            commands
                                                .entity(rook_starting_tile_id)
                                                .remove::<GamePiece>();
                                            commands.entity(rook_ending_tile_id).insert(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::Black,
                                                },
                                            );

                                            let (_, mut rook_starting_tile_visibility) =
                                                tile_sprite_q
                                                    .get_mut(rook_starting_tile_id)
                                                    .expect(
                                                        "Tile literal should exist on 8x8 board",
                                                    );
                                            *rook_starting_tile_visibility = Visibility::Hidden;

                                            let (
                                                mut rook_ending_tile_handle,
                                                mut rook_ending_tile_visibility,
                                            ) = tile_sprite_q
                                                .get_mut(rook_ending_tile_id)
                                                .expect("Tile literal should exist on 8x8 board");
                                            *rook_ending_tile_handle = asset_server.load(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::Black,
                                                }
                                                .get_asset_path()
                                                .to_string(),
                                            );
                                            *rook_ending_tile_visibility = Visibility::Visible;
                                        }
                                        TilePos { x: 6, y: 7 } => {
                                            let rook_starting_tile_id = tilemap
                                                .get(&TilePos { x: 7, y: 7 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            let rook_ending_tile_id = tilemap
                                                .get(&TilePos { x: 5, y: 7 })
                                                .expect("Tile literal should exist on 8x8 board");
                                            commands
                                                .entity(rook_starting_tile_id)
                                                .remove::<GamePiece>();
                                            commands.entity(rook_ending_tile_id).insert(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::Black,
                                                },
                                            );

                                            let (_, mut rook_starting_tile_visibility) =
                                                tile_sprite_q
                                                    .get_mut(rook_starting_tile_id)
                                                    .expect(
                                                        "Tile literal should exist on 8x8 board",
                                                    );
                                            *rook_starting_tile_visibility = Visibility::Hidden;

                                            let (
                                                mut rook_ending_tile_handle,
                                                mut rook_ending_tile_visibility,
                                            ) = tile_sprite_q
                                                .get_mut(rook_ending_tile_id)
                                                .expect("Tile literal should exist on 8x8 board");
                                            *rook_ending_tile_handle = asset_server.load(
                                                GamePiece {
                                                    piece: Piece::Rook,
                                                    color: Color::Black,
                                                }
                                                .get_asset_path()
                                                .to_string(),
                                            );
                                            *rook_ending_tile_visibility = Visibility::Visible;
                                        }
                                        _ => panic!("Invalid castling move"),
                                    }
                                }

                                // If the original tile was a rook's or king's starting tile, remove the corresponding right to castle.
                                match selected_piece_original_position_some {
                                    TilePos { x: 0, y: 0 } => {
                                        right_to_castle.white_queenside = false
                                    }
                                    TilePos { x: 7, y: 0 } => {
                                        right_to_castle.white_kingside = false
                                    }
                                    TilePos { x: 4, y: 0 } => {
                                        right_to_castle.white_kingside = false;
                                        right_to_castle.white_queenside = false;
                                    }
                                    TilePos { x: 0, y: 7 } => {
                                        right_to_castle.black_queenside = false
                                    }
                                    TilePos { x: 7, y: 7 } => {
                                        right_to_castle.black_kingside = false
                                    }
                                    TilePos { x: 4, y: 7 } => {
                                        right_to_castle.black_kingside = false;
                                        right_to_castle.black_queenside = false;
                                    }
                                    _ => {}
                                }

                                // Remove the game piece sprite from the cursor.
                                let mut cursor_visibility = cursor_q.single_mut();
                                *cursor_visibility = Visibility::Hidden;

                                // Reset the cursor.
                                *selected_piece = SelectedPiece(None);
                                *selected_piece_original_position =
                                    SelectedPieceOriginalPosition(None);

                                // Prepare to calculate the next legal moves.
                                color_to_move.switch();
                                *must_recalculate_legal_moves = MustRecalculateLegalMoves(true);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn recalculate_legal_moves(
    mut commands: Commands,
    color_to_move: Res<ColorToMove>,
    right_to_castle: Res<RightToCastle>,
    mut must_recalculate_legal_moves: ResMut<MustRecalculateLegalMoves>,
    tile_legal_moves_q: Query<Entity, With<LegalMoves>>,
    tile_game_piece_q: Query<(Entity, &GamePiece, &TilePos)>,
    tilemap_q: Query<&TileStorage>,
) {
    if must_recalculate_legal_moves.0 {
        // Remove legal moves from all existing tiles.
        for tile_id in &tile_legal_moves_q {
            commands.entity(tile_id).remove::<LegalMoves>();
        }

        let tilemap = tilemap_q.single();

        // A closure that returns the game piece on the specified square, or None if there is no
        // game piece on that square.
        let get_game_piece_at_position = |tile_pos: &TilePos| {
            let adjacent_tile_id = tilemap.get(tile_pos).expect("Tile should exist in tilemap");

            if let Ok((_, other_game_piece, _)) = tile_game_piece_q.get(adjacent_tile_id) {
                Some(*other_game_piece)
            } else {
                None
            }
        };

        // Get the tile the king is currently on.
        let mut king_tile = None;
        for (_, game_piece, tile_pos) in &tile_game_piece_q {
            if *game_piece
                == (GamePiece {
                    piece: Piece::King,
                    color: color_to_move.0,
                })
            {
                king_tile = Some(tile_pos);
                break;
            }
        }

        for (tile_id, game_piece, tile_pos) in &tile_game_piece_q {
            if game_piece.color == color_to_move.0 {
                // The maximum number of legal moves a single piece can have is 27. This occurs when
                // a queen is placed in the center of the board and is unobstructed in all
                // directions, where it can move 7 spaces vertically, 7 spaces horizontally, 7
                // spaces along the long diagonal, and 6 spaces along the short diagonal.
                let mut legal_moves = Vec::with_capacity(27);

                match game_piece.piece {
                    Piece::Pawn => {
                        find_legal_pawn_moves(
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        // TODO: Handle promotion.
                        // TODO: Handle en passant.
                    }
                    Piece::Rook => {
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                    }
                    Piece::Knight => {
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 2, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 2, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -2, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -2, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                    }
                    Piece::Bishop => {
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                    }
                    Piece::Queen => {
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                    }
                    Piece::King => {
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 0 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 0 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );

                        // Handle castling.
                        find_legal_castling_moves(
                            &mut legal_moves,
                            &right_to_castle,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        );
                    }
                }

                // Remove legal moves that would leave one's own king under attack. As a failsafe,
                // skip this process if the king wasn't found on the board.
                if let Some(king_tile) = king_tile {
                    legal_moves.retain(|legal_move| {
                        !is_king_threatened_after_move(
                            tile_pos,
                            &legal_move.destination,
                            king_tile,
                            &color_to_move.0,
                            get_game_piece_at_position,
                        )
                    });
                }

                commands.entity(tile_id).insert(LegalMoves(legal_moves));
            }
        }

        *must_recalculate_legal_moves = MustRecalculateLegalMoves(false);
    }
}

struct Direction {
    x: i32,
    y: i32,
}

fn find_legal_pawn_moves<F: Fn(&TilePos) -> Option<GamePiece>>(
    legal_moves: &mut Vec<LegalMove>,
    position: &TilePos,
    color_to_move: &Color,
    get_game_piece_at_position: F,
) {
    let y_direction = if *color_to_move == Color::White {
        1
    } else {
        -1
    };

    // Check the square immediately in front.
    // Use checked_add_signed() to make sure overflow doesn't occur from going below position 0
    // TODO: Once promotion is implemented, checking that the pawn isn't on the final rank will be
    // unnecessary.
    if let Some(y_next) = position.y.checked_add_signed(y_direction) {
        if y_next < MAP_SIZE.y
            && get_game_piece_at_position(&TilePos {
                x: position.x,
                y: y_next,
            })
            .is_none()
        {
            legal_moves.push(LegalMove {
                destination: TilePos {
                    x: position.x,
                    y: y_next,
                },
                is_castling: false,
            });
            // If the pawn could move forward one square, check if it can also move two squares
            // (it's on its starting rank and the next square is also free).
            if (*color_to_move == Color::White && position.y == 1)
                || (*color_to_move == Color::Black && position.y == 6)
            {
                // If a pawn is making a double move from its starting square, it's not possible for
                // its end position to be off the board, so checking for this isn't necessary.
                let y_next_next = y_next.wrapping_add_signed(y_direction);
                if get_game_piece_at_position(&TilePos {
                    x: position.x,
                    y: y_next_next,
                })
                .is_none()
                {
                    legal_moves.push(LegalMove {
                        destination: TilePos {
                            x: position.x,
                            y: y_next_next,
                        },
                        is_castling: false,
                    });
                }
            }
        }
    }

    // Check captures to both diagonals.
    // TODO: Once promotion is implemented, checking that the pawn isn't on the final rank will be
    // unnecessary.
    if let Some((x_next, y_next)) = position
        .x
        .checked_add_signed(-1)
        .zip(position.y.checked_add_signed(y_direction))
    {
        if y_next < MAP_SIZE.y
            && get_game_piece_at_position(&TilePos {
                x: x_next,
                y: y_next,
            })
            .is_some_and(|game_piece| game_piece.color != *color_to_move)
        {
            legal_moves.push(LegalMove {
                destination: TilePos {
                    x: x_next,
                    y: y_next,
                },
                is_castling: false,
            });
        }
    }
    // TODO: Once promotion is implemented, checking that the pawn isn't on the final rank will be
    // unnecessary.
    let x_next = position.x + 1;
    if let Some(y_next) = position.y.checked_add_signed(y_direction) {
        if x_next < MAP_SIZE.x
            && y_next < MAP_SIZE.y
            && get_game_piece_at_position(&TilePos {
                x: x_next,
                y: y_next,
            })
            .is_some_and(|game_piece| game_piece.color != *color_to_move)
        {
            legal_moves.push(LegalMove {
                destination: TilePos {
                    x: x_next,
                    y: y_next,
                },
                is_castling: false,
            });
        }
    }
}

fn find_legal_moves_in_direction<F: Fn(&TilePos) -> Option<GamePiece>>(
    direction: Direction,
    keep_going: bool,
    legal_moves: &mut Vec<LegalMove>,
    position: &TilePos,
    color_to_move: &Color,
    get_game_piece_at_position: F,
) {
    // Use checked_add_signed() to make sure overflow doesn't occur from going below position 0
    if let Some((x_next, y_next)) = position
        .x
        .checked_add_signed(direction.x)
        .zip(position.y.checked_add_signed(direction.y))
    {
        // If code reaches this point, no overflow occurred, so only need to check if the new
        // position isn't past the eighth row or column.
        if x_next < MAP_SIZE.x && y_next < MAP_SIZE.y {
            match get_game_piece_at_position(&TilePos {
                x: x_next,
                y: y_next,
            }) {
                // If there is no piece at the new position, add the position to the list of
                // legal moves. If keep_going is true, repeat the search from the new position.
                None => {
                    legal_moves.push(LegalMove {
                        destination: TilePos {
                            x: x_next,
                            y: y_next,
                        },
                        is_castling: false,
                    });
                    if keep_going {
                        find_legal_moves_in_direction(
                            direction,
                            true,
                            legal_moves,
                            &TilePos {
                                x: x_next,
                                y: y_next,
                            },
                            color_to_move,
                            get_game_piece_at_position,
                        );
                    }
                }
                // If there is a piece at the new position, add the position to the list of
                // legal moves only if the piece is of the opposite color (representing a
                // capture). Either way, do not continue the search since pieces block movement.
                Some(game_piece) => {
                    if game_piece.color != *color_to_move {
                        legal_moves.push(LegalMove {
                            destination: TilePos {
                                x: x_next,
                                y: y_next,
                            },
                            is_castling: false,
                        });
                    }
                }
            }
        }
    }
}

fn find_legal_castling_moves<F: Fn(&TilePos) -> Option<GamePiece>>(
    legal_moves: &mut Vec<LegalMove>,
    right_to_castle: &RightToCastle,
    color_to_move: &Color,
    get_game_piece_at_position: F,
) {
    let king_row = if *color_to_move == Color::White {
        0
    } else {
        MAP_SIZE.y - 1
    };

    // It can be assumed the king and rook are still in their starting positions, because if they
    // have ever moved, the associated right to castle would have been lost.

    // Note it is not necessary to check if the king's final position places it under attack, as
    // every legal move is checked to see if the final arrangement of pieces places the king under
    // attack after the vector of legal moves is constructed.

    // Column 4 is the king column. Columns 5 and 6 have the kingside bishop and knight.
    if ((*color_to_move == Color::White && right_to_castle.white_kingside)
        || (*color_to_move == Color::Black && right_to_castle.black_kingside))
        && get_game_piece_at_position(&TilePos { x: 5, y: king_row }).is_none()
        && get_game_piece_at_position(&TilePos { x: 6, y: king_row }).is_none()
    {
        legal_moves.push(LegalMove {
            destination: TilePos { x: 6, y: king_row },
            is_castling: true,
        });
    }

    // Column 4 is the king column. Columns 1, 2, and 3 have the queenside knight, bishop, and
    // queen.
    if ((*color_to_move == Color::White && right_to_castle.white_queenside)
        || (*color_to_move == Color::Black && right_to_castle.black_queenside))
        && get_game_piece_at_position(&TilePos { x: 1, y: king_row }).is_none()
        && get_game_piece_at_position(&TilePos { x: 2, y: king_row }).is_none()
        && get_game_piece_at_position(&TilePos { x: 3, y: king_row }).is_none()
    {
        legal_moves.push(LegalMove {
            destination: TilePos { x: 2, y: king_row },
            is_castling: true,
        });
    }
}

// Checks if, after moving the piece from the starting tile to the end tile, if the king would be
// under attack by an enemy piece.
fn is_king_threatened_after_move<'a, F: Fn(&TilePos) -> Option<GamePiece>>(
    starting_tile: &TilePos,
    end_tile: &'a TilePos,
    mut king_tile: &'a TilePos,
    color_to_move: &Color,
    get_game_piece_at_position: F,
) -> bool {
    // If the starting tile and king tile match, that means the king was moved, so update the king
    // position before proceeding further.
    if starting_tile == king_tile {
        king_tile = end_tile;
    }

    let opposite_color = if *color_to_move == Color::White {
        Color::Black
    } else {
        Color::White
    };

    #[derive(PartialEq, Eq)]
    struct GamePieceInDirectionResult {
        game_piece: GamePiece,
        is_adjacent: bool,
    }

    // Determines where the next game piece in the given direction is, taking into account that
    // after the legal move is played, a piece of the same color as the king has been moved from the
    // starting tile to the end tile. Returns an option with None if no piece was found in that
    // direction (or if the end tile was found in that direction which can never contain an
    // attacking piece), or a tuple containing the game piece in that direction and a boolean
    // indicating if it's immediately adjacent.
    let get_game_piece_in_direction = |direction: Direction| {
        let mut x = king_tile.x;
        let mut y = king_tile.y;
        let mut is_adjacent = true;

        while let Some((x_next, y_next)) = x
            .checked_add_signed(direction.x)
            .zip(y.checked_add_signed(direction.y))
        {
            if x_next < MAP_SIZE.x && y_next < MAP_SIZE.y {
                // The next position is on the board, so check what's at that position.
                let next_tile = TilePos {
                    x: x_next,
                    y: y_next,
                };

                // If the position is the end tile, this is the position another piece of the same
                // color was moved to, which will block whatever's in this direction. Though the
                // piece is unknown, all that matters is that it can't possibly be threatening the
                // king, so return None to indicate this.
                if next_tile == *end_tile {
                    return None;
                }

                // If the position is the start tile, this is the position another piece of the same
                // color was moved from. This piece should be ignored for the sake of determining if
                // the king is threatened from this direction.
                if next_tile != *starting_tile {
                    // This position isn't the starting or ending position, so check if there's a
                    // piece here, and if so, return it.
                    if let Some(game_piece_at_position) = get_game_piece_at_position(&next_tile) {
                        return Some(GamePieceInDirectionResult {
                            game_piece: game_piece_at_position,
                            is_adjacent,
                        });
                    }
                }

                // At this point, nothing was found on this tile, so update the position and proceed
                // to the next loop iteration.
                x = x_next;
                y = y_next;
                is_adjacent = false;
            } else {
                // x_next or y_next were greater than the board size, so stop looping because the
                // edge of the board has been reached.
                return None;
            }
        }
        // If the loop is broken before hitting a return statement, that means an overflow occurred
        // with one of the checked additions, which means the position tried to go past the first
        // row or column. As this is the edge of the board, this means there are no pieces in this
        // direction, so return None.
        None
    };

    // Check horizontally and vertically for a rook, queen, or immediately adjacent king of the
    // opposite color. Stop searching in a direction if any other piece is found, the end tile is
    // reached (as this means a piece of the same color is being moved there which will block attack
    // from that direction), or the end of the board is reached.
    let is_threatened_horizontally_or_vertically =
        |game_piece_in_direction_result: GamePieceInDirectionResult| {
            game_piece_in_direction_result.game_piece
                == GamePiece {
                    piece: Piece::Rook,
                    color: opposite_color,
                }
                || game_piece_in_direction_result.game_piece
                    == GamePiece {
                        piece: Piece::Queen,
                        color: opposite_color,
                    }
                || game_piece_in_direction_result
                    == GamePieceInDirectionResult {
                        game_piece: GamePiece {
                            piece: Piece::King,
                            color: opposite_color,
                        },
                        is_adjacent: true,
                    }
        };

    // Check diagonally for a bishop, queen, or immediately adjacent king. If white, also check
    // upwards diagonals for an immediately adjacent pawn.
    let is_threatened_diagonally_upwards =
        |game_piece_in_direction_result: GamePieceInDirectionResult| {
            game_piece_in_direction_result.game_piece
                == GamePiece {
                    piece: Piece::Bishop,
                    color: opposite_color,
                }
                || game_piece_in_direction_result.game_piece
                    == GamePiece {
                        piece: Piece::Queen,
                        color: opposite_color,
                    }
                || game_piece_in_direction_result
                    == GamePieceInDirectionResult {
                        game_piece: GamePiece {
                            piece: Piece::King,
                            color: opposite_color,
                        },
                        is_adjacent: true,
                    }
                // Only black pawns attack downwards, so only white's king is vulnerable to pawn
                // attacks from above.
                || (*color_to_move == Color::White
                    && game_piece_in_direction_result
                        == GamePieceInDirectionResult {
                            game_piece: GamePiece {
                                piece: Piece::Pawn,
                                color: opposite_color,
                            },
                            is_adjacent: true,
                        })
        };
    // Same diagonal check, except for black checking downwards diagonals for an immediately
    // adjacent pawn.
    let is_threatened_diagonally_downwards =
        |game_piece_in_direction_result: GamePieceInDirectionResult| {
            game_piece_in_direction_result.game_piece
                == GamePiece {
                    piece: Piece::Bishop,
                    color: opposite_color,
                }
                || game_piece_in_direction_result.game_piece
                    == GamePiece {
                        piece: Piece::Queen,
                        color: opposite_color,
                    }
                || game_piece_in_direction_result
                    == GamePieceInDirectionResult {
                        game_piece: GamePiece {
                            piece: Piece::King,
                            color: opposite_color,
                        },
                        is_adjacent: true,
                    }
                // Only white pawns attack upwards, so only black's king is vulnerable to pawn
                // attacks from below.
                || (*color_to_move == Color::Black
                    && game_piece_in_direction_result
                        == GamePieceInDirectionResult {
                            game_piece: GamePiece {
                                piece: Piece::Pawn,
                                color: opposite_color,
                            },
                            is_adjacent: true,
                        })
        };

    if get_game_piece_in_direction(Direction { x: 0, y: 1 })
        .is_some_and(is_threatened_horizontally_or_vertically)
    {
        return true;
    }
    if get_game_piece_in_direction(Direction { x: 1, y: 0 })
        .is_some_and(is_threatened_horizontally_or_vertically)
    {
        return true;
    }
    if get_game_piece_in_direction(Direction { x: 0, y: -1 })
        .is_some_and(is_threatened_horizontally_or_vertically)
    {
        return true;
    }
    if get_game_piece_in_direction(Direction { x: -1, y: 0 })
        .is_some_and(is_threatened_horizontally_or_vertically)
    {
        return true;
    }

    if get_game_piece_in_direction(Direction { x: 1, y: 1 })
        .is_some_and(is_threatened_diagonally_upwards)
    {
        return true;
    }
    if get_game_piece_in_direction(Direction { x: -1, y: 1 })
        .is_some_and(is_threatened_diagonally_upwards)
    {
        return true;
    }

    if get_game_piece_in_direction(Direction { x: 1, y: -1 })
        .is_some_and(is_threatened_diagonally_downwards)
    {
        return true;
    }
    if get_game_piece_in_direction(Direction { x: -1, y: -1 })
        .is_some_and(is_threatened_diagonally_downwards)
    {
        return true;
    }

    // At this point, every possible attack coming horizontally, vertically, and diagonally have
    // been checked. Knights are the only piece that don't move in these ways, so they must be
    // checked separately.
    let is_enemy_knight_in_direction = |direction: Direction| {
        if let Some((x_next, y_next)) = king_tile
            .x
            .checked_add_signed(direction.x)
            .zip(king_tile.y.checked_add_signed(direction.y))
        {
            if x_next < MAP_SIZE.x && y_next < MAP_SIZE.y {
                let tile_pos = TilePos {
                    x: x_next,
                    y: y_next,
                };
                // If the end tile is at this position, then that means any knight that could be at
                // that position was captured, leaving the king safe from attack from that position.
                if tile_pos == *end_tile {
                    return false;
                }
                if get_game_piece_at_position(&tile_pos)
                    == Some(GamePiece {
                        piece: Piece::Knight,
                        color: opposite_color,
                    })
                {
                    return true;
                }
            }
        }
        false
    };

    if is_enemy_knight_in_direction(Direction { x: 1, y: 2 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: 2, y: 1 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: 2, y: -1 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: 1, y: -2 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: -1, y: -2 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: -2, y: -1 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: -2, y: 1 }) {
        return true;
    }
    if is_enemy_knight_in_direction(Direction { x: -1, y: 2 }) {
        return true;
    }

    // This function checks every possible direction an attack could be coming from, so if the end
    // of this function is reached, the king is not under attack.
    false
}

pub fn highlight_tile(
    tile_highlight_q: Query<(Option<&LegalMoves>, &TilePos), With<MouseoverHighlight>>,
    mut tile_texture_q: Query<(&mut TileTextureIndex, &TilePos)>,
) {
    let mut mouseover_tile = None;
    // This vector needs no capacity. It will either remain empty or be overwritten by a new vector.
    let mut legal_move_tiles = Vec::with_capacity(0);

    if let Ok((legal_moves, tile_pos)) = tile_highlight_q.get_single() {
        mouseover_tile = Some(tile_pos);
        if let Some(legal_moves_some) = legal_moves {
            // Highlighting tiles is only concerned with destination tiles, so map the legal moves
            // to just a vector of destination tiles.
            legal_move_tiles = legal_moves_some
                .0
                .iter()
                .map(|legal_move| legal_move.destination)
                .collect();
        }
    }

    for (mut tile_texture_index, tile_pos) in &mut tile_texture_q {
        if mouseover_tile == Some(tile_pos) {
            *tile_texture_index = TileTextureIndex((tile_pos.x + tile_pos.y) % 2 + 2);
        } else if legal_move_tiles.contains(tile_pos) {
            *tile_texture_index = TileTextureIndex((tile_pos.x + tile_pos.y) % 2 + 4);
        } else {
            *tile_texture_index = TileTextureIndex((tile_pos.x + tile_pos.y) % 2);
        }
    }
}

pub fn update_cursor_display(
    cursor_pos: Res<CursorPos>,
    mut cursor_q: Query<&mut Transform, With<CursorDisplay>>,
) {
    let mut transform = cursor_q.single_mut();
    *transform = transform.with_translation(Vec3 {
        x: cursor_pos.0.x,
        y: cursor_pos.0.y,
        z: 2.0,
    });
}
