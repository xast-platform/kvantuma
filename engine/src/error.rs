use glfw::InitError;
use thiserror::Error;

use crate::render::error::RenderError;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Cannot initialize game: {0}")]
    InitializeGame(#[from] InitError),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),
}