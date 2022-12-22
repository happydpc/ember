use crate::core::plugins::Plugin;
use libloading::Library as WinLib;
use libloading::Symbol as WinSymbol;
use libloading::Error as LibError;
use std::ffi::OsStr;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::CStr;

#[cfg(target_os = "windows")]
use libloading::os::windows::Library;
#[cfg(target_os = "windows")]
use libloading::os::windows::Symbol;

#[cfg(target_os = "macos")]
use libloading::os::unix::Library;
#[cfg(target_os = "macos")]
use libloading::os::unix::Symbol;

use errors::*;

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    pub fn startup(&mut self){
        log::info!("Starting plugin manager...");
        
    }

    pub unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<(), LibError> {
        type PluginCreate = unsafe fn() -> *mut dyn Plugin;

        let lib = Library::new(filename.as_ref())?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")?;
        let boxed_raw = constructor();

        let plugin = Box::from_raw(boxed_raw);
        log::debug!("Loaded plugin: {}", plugin.name());
        plugin.on_plugin_load();
        self.plugins.push(plugin);


        Ok(())
    }
    
    /// Unload all plugins and loaded plugin libraries, making sure to fire 
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        log::info!("Unloading plugins");

        for plugin in self.plugins.drain(..) {
            log::debug!("Firing on_plugin_unload for {:?}", plugin.name());
            plugin.on_plugin_unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_manager_new() -> *mut PluginManager {
    Box::into_raw(Box::new(PluginManager::new()))
}

/// Destroy a `PluginManager` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn plugin_manager_destroy(pm: *mut PluginManager) {
    if !pm.is_null() {
        let pm = Box::from_raw(pm);
        drop(pm);
    }
}

/// Unload all loaded plugins.
#[no_mangle]
pub unsafe extern "C" fn plugin_manager_unload(pm: *mut PluginManager) {
    let pm = &mut *pm;
    pm.unload();
}

#[no_mangle]
pub unsafe extern "C" fn plugin_manager_load_plugin(
    pm: *mut PluginManager,
    filename: *const c_char,
    // filename: *const CStr,
) -> c_int {
    let pm = &mut *pm;
    let filename = CStr::from_ptr(filename);
    let filename_as_str = match filename.to_str() {
        Ok(s) => s,
        Err(_) => {
            // TODO: proper error handling
            return -1;
        }
    };

    // TODO: proper error handling and catch_unwind
    match pm.load_plugin(filename_as_str) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}