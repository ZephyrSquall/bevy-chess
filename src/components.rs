use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Clone)]
pub enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Component, Clone)]
pub struct GamePiece {
    pub piece: Piece,
    pub color: Color,
}
impl GamePiece {
    pub fn get_asset_path(&self) -> &str {
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

#[derive(Component)]
pub struct MouseoverHighlight();

#[derive(Component)]
pub struct LegalMoves(pub Vec<TilePos>);

#[derive(Component)]
pub struct CursorDisplay;
