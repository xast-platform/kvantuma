use flecs_ecs::prelude::World;
use xastge::{
    Transform, 
    render::{
        RenderDevice, 
        camera::{
            Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera, 
            build_orthographic_uniform, build_perspective_uniform,
        }, 
        registry::RenderRegistry,
    }
};

pub fn update_camera_buffer(
    world: &mut World,
    render_device: &RenderDevice,
    registry: &RenderRegistry,
) {    
    world.each::<(&Camera, &OrthographicCamera, &Transform, &CameraBuffer)>(|(cam, ort_cam, t, buf)| {
        let uniform = build_orthographic_uniform(cam, ort_cam, t);
        if let Some(buf) = registry.get_buffer(buf.handle()) {
            buf.fill_exact(render_device, 0, &[uniform]).unwrap_or_else(|e| {
                log::error!("{e}");
            });
        } else {
            log::error!("Camera buffer not found in registry");
        }
    });

    world.each::<(&Camera, &PerspectiveCamera, &Transform, &CameraBuffer)>(|(cam, persp_cam, t, buf)| {
        let uniform = build_perspective_uniform(cam, persp_cam, t);
        if let Some(buf) = registry.get_buffer(buf.handle()) {
            buf.fill_exact(render_device, 0, &[uniform]).unwrap_or_else(|e| {
                log::error!("{e}");
            });
        } else {
            log::error!("Camera buffer not found in registry");
        }
    });
}