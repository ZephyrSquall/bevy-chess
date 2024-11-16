use super::input::CursorPos;
use super::setup::CursorDisplay;
use crate::{GamePiece, MAP_SIZE, MAP_TYPE, SCALED_GRID_SIZE};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct MouseoverHighlight();

#[derive(Resource, Default)]
pub struct SelectedPiece(Option<GamePiece>);

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

pub fn highlight_tile(
    mut tile_texture_q: Query<(&mut TileTextureIndex, Option<&MouseoverHighlight>, &TilePos)>,
) {
    for (mut tile_texture_index, mouseover_highlight, tile_pos) in &mut tile_texture_q {
        if mouseover_highlight.is_some() {
            *tile_texture_index = TileTextureIndex(2);
        } else {
            *tile_texture_index = TileTextureIndex((tile_pos.x + tile_pos.y) % 2);
        }
    }
}

pub fn pick_up_piece(
    mut selected_piece: ResMut<SelectedPiece>,
    mouse: Res<ButtonInput<MouseButton>>,
    // Must include "Without<CursorDisplay>" to create a disjoint query that doesn't mutably access
    // the "Visibility" component of an entity twice.
    mut tile_q: Query<
        (&mut Visibility, Option<&GamePiece>),
        (With<MouseoverHighlight>, Without<CursorDisplay>),
    >,
    mut cursor_q: Query<(&mut Handle<Image>, &mut Visibility), With<CursorDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (mut visibility, game_piece) in &mut tile_q {
            if let Some(game_piece) = game_piece {
                *selected_piece = SelectedPiece(Some(game_piece.clone()));
                *visibility = Visibility::Hidden;

                let (mut cursor_handle, mut cursor_visibility) = cursor_q.single_mut();
                *cursor_handle = asset_server.load(game_piece.get_asset_path().to_string());
                *cursor_visibility = Visibility::Visible;
            }
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
