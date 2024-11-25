use crate::components::{Color, GamePiece};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Resource)]
pub struct ColorToMove(pub Color);
impl Default for ColorToMove {
    fn default() -> Self {
        ColorToMove(Color::White)
    }
}
impl ColorToMove {
    pub fn switch(&mut self) {
        match self {
            ColorToMove(Color::White) => self.0 = Color::Black,
            ColorToMove(Color::Black) => self.0 = Color::White,
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectedPiece(pub Option<GamePiece>);

#[derive(Resource, Default)]
pub struct SelectedPieceOriginalPosition(pub Option<TilePos>);

#[derive(Resource)]
pub struct MustRecalculateLegalMoves(pub bool);
impl Default for MustRecalculateLegalMoves {
    fn default() -> Self {
        MustRecalculateLegalMoves(true)
    }
}

#[derive(Resource)]
pub struct CursorPos(pub Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

#[derive(Resource)]
pub struct RightToCastle {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}
impl Default for RightToCastle {
    fn default() -> Self {
        RightToCastle {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}
