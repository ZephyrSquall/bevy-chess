use crate::components::{Color, GamePiece};
use bevy::prelude::*;

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
pub struct SelectedPieceOriginalTile(pub Option<Entity>);

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
