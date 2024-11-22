use crate::components::{Color, CursorDisplay, GamePiece, LegalMoves, MouseoverHighlight, Piece};
use crate::resources::{
    ColorToMove, CursorPos, MustRecalculateLegalMoves, SelectedPiece, SelectedPieceOriginalTile,
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
    mut selected_piece_original_tile: ResMut<SelectedPieceOriginalTile>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    // Must include "Without<CursorDisplay>" to create a disjoint query that doesn't mutably access
    // the "Visibility" component of an entity twice.
    mut tile_q: Query<
        (Entity, &mut Visibility, Option<&GamePiece>),
        (With<MouseoverHighlight>, Without<CursorDisplay>),
    >,
    mut cursor_q: Query<(&mut Handle<Image>, &mut Visibility), With<CursorDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if mouse.just_pressed(MouseButton::Left) && selected_piece.0.is_none() {
        for (tile_id, mut visibility, game_piece) in &mut tile_q {
            if let Some(game_piece) = game_piece {
                // Get game piece from current tile, and hide it on the tile while it is carried.
                *selected_piece = SelectedPiece(Some(game_piece.clone()));
                *visibility = Visibility::Hidden;

                // Display the game piece on the cursor.
                let (mut cursor_handle, mut cursor_visibility) = cursor_q.single_mut();
                *cursor_handle = asset_server.load(game_piece.get_asset_path().to_string());
                *cursor_visibility = Visibility::Visible;

                // Track the piece's original position.
                *selected_piece_original_tile = SelectedPieceOriginalTile(Some(tile_id));

                mouse.clear_just_pressed(MouseButton::Left);
            }
        }
    }
}

pub fn put_down_piece(
    mut commands: Commands,
    mut selected_piece: ResMut<SelectedPiece>,
    mut selected_piece_original_tile: ResMut<SelectedPieceOriginalTile>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut color_to_move: ResMut<ColorToMove>,
    mut must_recalculate_legal_moves: ResMut<MustRecalculateLegalMoves>,
    mut tile_q: Query<
        (Entity, &mut Handle<Image>, &mut Visibility),
        (With<MouseoverHighlight>, Without<CursorDisplay>),
    >,
    mut cursor_q: Query<&mut Visibility, With<CursorDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        // Get the currently-selected piece, or skip if no piece is selected.
        if let Some(selected_piece_some) = &selected_piece.0 {
            // Get the original tile of the currently-selected piece (this shouldn't be None if
            // there is a selected piece).
            if let Some(selected_piece_original_tile_some) = &selected_piece_original_tile.0 {
                // Get the current mouseover tile, or skip if the cursor is not over a tile.
                if let Ok((tile_id, mut handle, mut visibility)) = tile_q.get_single_mut() {
                    // Remove the GamePiece on the original tile (do this before adding the GamePiece
                    // from the cursor in case it's on the same tile).
                    commands
                        .entity(*selected_piece_original_tile_some)
                        .remove::<GamePiece>();

                    // Add the GamePiece on the tile that the cursor is currently hovering over.
                    commands.entity(tile_id).insert(selected_piece_some.clone());
                    *handle = asset_server.load(selected_piece_some.get_asset_path().to_string());
                    *visibility = Visibility::Visible;

                    // Remove the game piece sprite from the cursor.
                    let mut cursor_visibility = cursor_q.single_mut();
                    *cursor_visibility = Visibility::Hidden;

                    // Reset the cursor.
                    *selected_piece = SelectedPiece(None);
                    *selected_piece_original_tile = SelectedPieceOriginalTile(None);

                    // Prepare to calculate the next legal moves.
                    color_to_move.switch();
                    *must_recalculate_legal_moves = MustRecalculateLegalMoves(true);
                }
            }
        }
    }
}

pub fn recalculate_legal_moves(
    mut commands: Commands,
    color_to_move: Res<ColorToMove>,
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

        // A closure that returns the color of the game piece on the specified square, or None if
        // there is no game piece on that square.
        let get_game_piece_color_at_position = |tile_pos: &TilePos| {
            let adjacent_tile_id = tilemap.get(tile_pos).expect("Tile should exist in tilemap");

            if let Ok((_, other_game_piece, _)) = &tile_game_piece_q.get(adjacent_tile_id) {
                Some(other_game_piece.color)
            } else {
                None
            }
        };

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
                            get_game_piece_color_at_position,
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
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                    }
                    Piece::Knight => {
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 2, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 2, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -2, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -2, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 2 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                    }
                    Piece::Bishop => {
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                    }
                    Piece::Queen => {
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 0 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 1 },
                            true,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                    }
                    Piece::King => {
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: 0 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 1, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: 0, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: -1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 0 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        find_legal_moves_in_direction(
                            Direction { x: -1, y: 1 },
                            false,
                            &mut legal_moves,
                            tile_pos,
                            &color_to_move.0,
                            get_game_piece_color_at_position,
                        );
                        // TODO: Handle castling.
                    }
                }

                // TODO: Remove legal moves that would leave one's own king under attack.

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

fn find_legal_pawn_moves<F: Fn(&TilePos) -> Option<Color>>(
    legal_moves: &mut Vec<TilePos>,
    position: &TilePos,
    color_to_move: &Color,
    get_game_piece_color_at_position: F,
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
            && get_game_piece_color_at_position(&TilePos {
                x: position.x,
                y: y_next,
            })
            .is_none()
        {
            legal_moves.push(TilePos {
                x: position.x,
                y: y_next,
            });
            // If the pawn could move forward one square, check if it can also move two squares
            // (it's on its starting rank and the next square is also free).
            if (*color_to_move == Color::White && position.y == 1)
                || (*color_to_move == Color::Black && position.y == 6)
            {
                // If a pawn is making a double move from its starting square, it's not possible for
                // its end position to be off the board, so checking for this isn't necessary.
                let y_next_next = y_next.wrapping_add_signed(y_direction);
                if get_game_piece_color_at_position(&TilePos {
                    x: position.x,
                    y: y_next_next,
                })
                .is_none()
                {
                    legal_moves.push(TilePos {
                        x: position.x,
                        y: y_next_next,
                    });
                }
            }
        }
    }

    // Check captures to both diagonals.
    // TODO: Once promotion is implemented, checking that the pawn isn't on the final rank will be
    // unnecessary.
    if let Some(x_next) = position.x.checked_add_signed(-1) {
        if let Some(y_next) = position.y.checked_add_signed(y_direction) {
            if y_next < MAP_SIZE.y
                && get_game_piece_color_at_position(&TilePos {
                    x: x_next,
                    y: y_next,
                })
                .is_some_and(|color| color != *color_to_move)
            {
                legal_moves.push(TilePos {
                    x: x_next,
                    y: y_next,
                });
            }
        }
    }
    // TODO: Once promotion is implemented, checking that the pawn isn't on the final rank will be
    // unnecessary.
    let x_next = position.x + 1;
    if let Some(y_next) = position.y.checked_add_signed(y_direction) {
        if x_next < MAP_SIZE.x
            && y_next < MAP_SIZE.y
            && get_game_piece_color_at_position(&TilePos {
                x: x_next,
                y: y_next,
            })
            .is_some_and(|color| color != *color_to_move)
        {
            legal_moves.push(TilePos {
                x: x_next,
                y: y_next,
            });
        }
    }
}

fn find_legal_moves_in_direction<F: Fn(&TilePos) -> Option<Color>>(
    direction: Direction,
    keep_going: bool,
    legal_moves: &mut Vec<TilePos>,
    position: &TilePos,
    color_to_move: &Color,
    get_game_piece_color_at_position: F,
) {
    // Use checked_add_signed() to make sure overflow doesn't occur from going below position 0
    if let Some(x_next) = position.x.checked_add_signed(direction.x) {
        if let Some(y_next) = position.y.checked_add_signed(direction.y) {
            // If code reaches this point, no overflow occurred, so only need to check if the new
            // position isn't past the eighth row or column.
            if x_next < MAP_SIZE.x && y_next < MAP_SIZE.y {
                match get_game_piece_color_at_position(&TilePos {
                    x: x_next,
                    y: y_next,
                }) {
                    // If there is no piece at the new position, add the position to the list of
                    // legal moves. If keep_going is true, repeat the search from the new position.
                    None => {
                        legal_moves.push(TilePos {
                            x: x_next,
                            y: y_next,
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
                                get_game_piece_color_at_position,
                            );
                        }
                    }
                    // If there is a piece at the new position, add the position to the list of
                    // legal moves only if the piece is of the opposite color (representing a
                    // capture). Either way, do not continue the search since pieces block movement.
                    Some(color) => {
                        if color != *color_to_move {
                            legal_moves.push(TilePos {
                                x: x_next,
                                y: y_next,
                            });
                        }
                    }
                }
            }
        }
    }
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
            legal_move_tiles = legal_moves_some.0.clone();
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
