use thiserror::Error;
use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError, SurfaceError};

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("The swap chain has been lost and needs to be recreated")]
    Lost,
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
    #[error("Error acquiring current texture: {0}")]
    SurfaceError(#[from] SurfaceError),
    #[error("Cannot create surface: {0}")]
    CreateSurface(#[from] CreateSurfaceError),
    #[error("Cannot request adapter: {0}")]
    RequestAdapter(#[from] RequestAdapterError),
    #[error("Cannot request device: {0}")]
    RequestDevice(#[from] RequestDeviceError),
    #[error("Window handle error: {0}")]
    HandleError(String),
    #[error("Buffer overflow: {0}")]
    BufferOverflow(usize),
    
}