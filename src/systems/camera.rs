use hecs::World;
use xastge::{Transform, render::{RenderDevice, camera::{Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera, build_orthographic_uniform, build_perspective_uniform}, registry::RenderRegistry}};

pub fn update_camera_buffer(
    world: &mut World,
    render_device: &RenderDevice,
    registry: &RenderRegistry,
) {
    for (_, (cam, ort_cam, t, buf)) in &mut world.query::<(&Camera, &OrthographicCamera, &Transform, &CameraBuffer)>() {
        let uniform = build_orthographic_uniform(cam, ort_cam, t);
        if let Some(buf) = registry.get_buffer(buf.handle()) {
            buf.fill_exact(render_device, 0, &[uniform]).unwrap_or_else(|e| {
                log::error!("{e}");
            });
        } else {
            log::error!("Camera buffer not found in registry");
        }
    }

    for (_, (cam, persp_cam, t, buf)) in &mut world.query::<(&Camera, &PerspectiveCamera, &Transform, &CameraBuffer)>() {
        let uniform = build_perspective_uniform(cam, persp_cam, t);
        if let Some(buf) = registry.get_buffer(buf.handle()) {
            buf.fill_exact(render_device, 0, &[uniform]).unwrap_or_else(|e| {
                log::error!("{e}");
            });
        } else {
            log::error!("Camera buffer not found in registry");
        }
    }
}