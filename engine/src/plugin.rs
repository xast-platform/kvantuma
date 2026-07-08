use std::{
    fs, 
    path::PathBuf, 
    sync::OnceLock,
};
use flecs_ecs::{
    core::{World, utility::traits::IdOperations},
    sys::{ecs_entity_t, ecs_world_t},
};
use libloading::Library;
use parking_lot::Mutex;
use thiserror::Error;

use crate::{RandomNumber, Render, Update};

static PLUGIN_REGISTRY: OnceLock<Mutex<PluginRegistry>> = OnceLock::new();

#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Library>,
}

impl PluginRegistry {
    pub fn push(&mut self, plugin: Library) {
        self.plugins.push(plugin);
    }
}

#[derive(Debug, Error)]
pub enum LoadPluginError {
    #[error("Plugin file cannot be found: {0}")]
    FileNotFound(PathBuf),
    #[error("Error opening plugin shared library: {0}")]
    SharedLibraryError(libloading::Error),
}

pub trait LoadPluginExt {
    fn load_plugin(&self, name: &str) -> Result<(), LoadPluginError>;
}

impl LoadPluginExt for World {
    fn load_plugin(&self, name: &str) -> Result<(), LoadPluginError> {
        let filename = libloading::library_filename(name);
        let mut path = PathBuf::new();
        path.push("assets");
        path.push("plugins");
        path.push(&filename);

        if !fs::exists(&path).unwrap_or(false) {
            return Err(LoadPluginError::FileNotFound(path));
        }

        let plugin = unsafe {
            Library::new(path)
                .map_err(LoadPluginError::SharedLibraryError)?
        };

        unsafe {
            let register_systems = plugin.get::<RegisterSystems>(b"register_systems")
                .map_err(LoadPluginError::SharedLibraryError)?;

            let api = PluginApi {
                world: self.ptr_mut(),
                update_label: *self.component::<Update>().id(),
                render_label: *self.component::<Render>().id(),
                random_number: *self.component::<RandomNumber>().id(),
            };

            register_systems(&api);
        }

        let mut registry = PLUGIN_REGISTRY
            .get_or_init(|| Mutex::new(PluginRegistry::default()))
            .lock();

        registry.push(plugin);

        Ok(())
    }
}

pub type RegisterSystems = unsafe extern "C" fn(*const PluginApi);

#[repr(C)]
pub struct PluginApi {
    // World
    pub world: *mut ecs_world_t,
    // Labels
    pub update_label: ecs_entity_t,
    pub render_label: ecs_entity_t,
    // Components
    pub random_number: ecs_entity_t,
}