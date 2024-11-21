use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use resources::{
    ColorToMove, CursorPos, MustRecalculateLegalMoves, SelectedPiece, SelectedPieceOriginalTile,
};
use system::input::update_cursor_pos;
use system::setup::{setup_board, setup_cursor, setup_pieces};
use system::update::{
    find_mouseover_tile, highlight_tile, pick_up_piece, put_down_piece, recalculate_legal_moves,
    update_cursor_display,
};

mod components;
mod resources;
mod system;

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
    .init_resource::<CursorPos>()
    .init_resource::<SelectedPiece>()
    .init_resource::<SelectedPieceOriginalTile>()
    .init_resource::<ColorToMove>()
    .init_resource::<MustRecalculateLegalMoves>()
    .add_systems(Startup, (setup_board, setup_pieces).chain())
    .add_systems(Startup, setup_cursor)
    .add_systems(First, update_cursor_pos)
    .add_systems(
        Update,
        (
            find_mouseover_tile,
            pick_up_piece,
            put_down_piece,
            recalculate_legal_moves,
            highlight_tile,
        )
            .chain(),
    )
    .add_systems(Update, update_cursor_display)
    .run();
}
