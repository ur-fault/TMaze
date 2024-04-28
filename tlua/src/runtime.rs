use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;
use tokio::task::LocalSet;

use crate::{lua_modules::LuaModule, util::static_ref};

/// The runtime options for the Lua runtime
///
/// # Safety
///
/// Both of these options are unsafe because they can be used to load unsafe
/// modules into the Lua runtime. FFI module is enabled by default.
/// Same cannot be said for the debug module, which is disabled by default.
pub struct RuntimeOption {
    pub debug: bool,
    pub ffi: bool,
    pub rt_name: &'static str,
    pub default_mods: bool,
    _private: (), // so that it can't be constructed outside of this module
}

impl Default for RuntimeOption {
    fn default() -> Self {
        Self {
            debug: false,
            ffi: true,
            rt_name: "tlua",
            default_mods: true,
            _private: (),
        }
    }
}

impl RuntimeOption {
    /// Create a new runtime option with default values
    pub fn new() -> Self {
        Self::default()
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn ffi(mut self, ffi: bool) -> Self {
        self.ffi = ffi;
        self
    }

    pub fn rt_name(mut self, rt_name: &'static str) -> Self {
        self.rt_name = rt_name;
        self
    }

    pub fn with_default_mods(mut self, with_default_mods: bool) -> Self {
        self.default_mods = with_default_mods;
        self
    }

    pub fn to_stdlib(&self) -> LuaStdLib {
        let mut lib = LuaStdLib::ALL_SAFE;
        if self.debug {
            lib |= LuaStdLib::DEBUG;
        }
        if self.ffi {
            lib |= LuaStdLib::FFI;
        }
        lib
    }
}

#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RefCell<RuntimeInner<'static>>>,
}

pub struct RuntimeInner<'l> {
    lua: &'static Lua,
    // callback_queue: Vec<LuaFunction<'l>>,
    // `FnOnce` would be better, but it requires `unsized locals`
    // which are:
    //      1. unstable,
    //      2. slow af
    queue: Vec<Box<dyn Fn(&'static Lua) -> LuaResult<()>>>,
    rs_obj: LuaTable<'l>,
    local_set: LocalSet,
}

impl Runtime {
    // ** Creating a new instances of the runtime.
    // ** Althought, there should be only single instance,
    // ** since we leak the Lua state, to make it 'static.
    pub fn new(rs_mod_name: &str) -> Self {
        Self::from_lua(Lua::new(), rs_mod_name)
            .with_default_mods()
            .unwrap()
    }

    pub fn with_default_mods(self) -> LuaResult<Self> {
        self.load_modules([crate::lua_modules::prelude::FsModule])
            .map(|_| self)
    }

    /// Create a new runtime with options
    ///
    /// # Example
    ///
    /// ```
    /// # use tlua::runtime::{Runtime, RuntimeOption};
    /// let rt = unsafe { Runtime::new_with_options(RuntimeOption::new().debug(true)) };
    /// ```
    ///
    /// # Safety
    /// This function is unsafe because it creates a new Lua state with the specified options
    pub unsafe fn new_with_options(options: RuntimeOption) -> Self {
        // SAFETY: idc, like really, I want speed and ffi so
        let lua = static_ref(Lua::unsafe_new_with(
            options.to_stdlib(),
            LuaOptions::default(),
        ));

        let rs_obj = lua.create_table().unwrap();
        lua.globals().set(options.rt_name, rs_obj.clone()).unwrap();

        Self {
            inner: Rc::new(RefCell::new(RuntimeInner {
                lua,
                // callback_queue: Vec::new(),
                queue: Vec::new(),
                rs_obj,
                local_set: LocalSet::new(),
            })),
        }
    }

    pub fn from_lua(lua: Lua, rs_mod_name: &str) -> Self {
        let lua = static_ref(lua);
        let rs_obj = lua.create_table().unwrap();
        lua.globals().set(rs_mod_name, rs_obj.clone()).unwrap();

        Self {
            inner: Rc::new(RefCell::new(RuntimeInner {
                lua,
                // callback_queue: Vec::new(),
                queue: Vec::new(),
                rs_obj,
                local_set: LocalSet::new(),
            })),
        }
    }

    // ** Utils

    pub fn lua(&self) -> &'static Lua {
        self.inner.borrow().lua
    }

    // ** Loading and executing Lua code

    pub(crate) fn load(&self, code: &str) -> LuaResult<LuaFunction> {
        self.lua().load(code).into_function()
    }

    pub fn eval<T: FromLuaMulti<'static>>(&self, code: &str) -> LuaResult<T> {
        self.lua().load(code).eval()
    }

    pub fn exec(&self, code: &str) -> LuaResult<()> {
        self.lua().load(code).exec()
    }

    // ** Loading Rust modules

    pub fn load_modules(
        &self,
        modules: impl IntoIterator<Item = impl LuaModule + 'static>,
    ) -> LuaResult<()> {
        for module in modules {
            self.load_rs_module(module)?;
        }
        Ok(())
    }

    pub fn load_rs_module(&self, module: impl LuaModule + 'static) -> LuaResult<()> {
        let module = static_ref(module);

        let dict = self.lua().create_table()?;
        module.init(self, dict.clone())?; // only the ref
        self.inner.borrow_mut().rs_obj.set(module.name(), dict)?;
        Ok(())
    }

    // ** Constrol the runtime from Lua

    pub fn queue_callback(&self, callback: LuaFunction<'static>) {
        // self.inner.borrow_mut().callback_queue.push(callback);
        self.inner.borrow_mut().queue.push(Box::new(move |_| {
            callback.call::<(), LuaMultiValue>(()).map(|_| ())
        }));
    }

    // pub fn spawn_lua_fn<T: for<'l> FromLuaMulti<'l> + Send + 'static>(
    //     &self,
    //     fn_: LuaFunction<'static>,
    // ) -> LuaResult<JoinHandle<T>> {
    //     Ok(tokio::task::spawn_local(async move {
    //         fn_.call::<(), T>(()).unwrap()
    //     }))
    // }

    pub fn run_frame(&mut self, max_tasks: Option<usize>) {
        if let Some(n) = max_tasks {
            for mt in self.inner.borrow_mut().queue.drain(..n) {
                // mt.call::<_, ()>(()).expect("error running task");
                mt(self.lua()).expect("error running task");
            }
        } else {
            for mt in self.inner.borrow_mut().queue.drain(..) {
                // mt.call::<_, ()>(()).expect("error running task");
                mt(self.lua()).expect("error running task");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime() {
        {
            let rt = Runtime::new("tlua");
            let code = "return 2";
            let func = rt.load(code).unwrap();
            let result: Result<i32, _> = func.call(());
            assert!(matches!(result, Ok(2)));
        }

        {
            let rt = Runtime::new("tlua");
            let code = "a = {...}; sum = 0; for i = 1, #a do sum = sum + a[i] end; return sum";
            let func = rt.load(code).unwrap();
            let result: Result<i32, _> = func.call((1, 2, 3));
            assert!(matches!(result, Ok(6)));
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    #[ignore = "is blocking, was just an experiment"]
    fn test_ffi_module() {
        let rt = unsafe { Runtime::new_with_options(RuntimeOption::new().ffi(true)) };

        let code = r#"
local ffi = require("ffi")
ffi.cdef[[
int MessageBoxA(void *w, const char *txt, const char *cap, int type);
]]
ffi.C.MessageBoxA(nil, "Hello world!", "Test", 0)"#;
        rt.eval::<Option<bool>>(code).unwrap();
    }
}
