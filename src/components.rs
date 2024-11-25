use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone)]
pub struct LegalMove {
    pub destination: TilePos,
    // Castling, en passant, and pawn double moves require special handling (castling moves a rook
    // in addition to the king, en passant captures a piece on a square other than the destination
    // square, and a pawn double move gives en passant rights to the opponent's pawns).
    pub is_castling: bool,
}

#[derive(Component, Clone)]
pub struct LegalMoves(pub Vec<LegalMove>);

#[derive(Component)]
pub struct CursorDisplay;
