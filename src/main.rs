use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use system::input::{update_cursor_pos, CursorPos};
use system::setup::{setup_board, setup_cursor, setup_pieces};
use system::update::{
    find_mouseover_tile, highlight_tile, pick_up_piece, update_cursor_display, SelectedPiece,
};

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

#[derive(Clone)]
enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

#[derive(Clone)]
enum Color {
    White,
    Black,
}

#[derive(Component, Clone)]
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
    .init_resource::<CursorPos>()
    .init_resource::<SelectedPiece>()
    .add_systems(Startup, (setup_board, setup_pieces).chain())
    .add_systems(Startup, setup_cursor)
    .add_systems(First, update_cursor_pos)
    .add_systems(
        Update,
        (find_mouseover_tile, highlight_tile, pick_up_piece).chain(),
    )
    .add_systems(Update, update_cursor_display)
    .run();
}
