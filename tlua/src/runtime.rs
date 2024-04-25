use mlua::prelude::*;
use tokio::task::JoinHandle;

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
    _private: (), // so that it can't be constructed outside of this module
}

impl Default for RuntimeOption {
    fn default() -> Self {
        Self {
            debug: false,
            ffi: true,
            rt_name: "tlua",
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

pub struct Runtime<'l> {
    lua: &'static Lua,
    mt_queue: Vec<LuaFunction<'l>>,
    rs_obj: LuaTable<'l>,
}

impl<'l> Runtime<'l> {
    pub fn new(rs_mod_name: &str) -> Self {
        Self::from_lua(Lua::new(), rs_mod_name)
    }

    /// Create a new runtime with options
    ///
    /// # Arguments
    ///
    /// * `options` - The options to create the runtime with
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
            lua,
            mt_queue: Vec::new(),
            rs_obj,
        }
    }

    pub fn from_lua(lua: Lua, rs_mod_name: &str) -> Self {
        let lua = static_ref(lua);
        let rs_obj = lua.create_table().unwrap();
        lua.globals().set(rs_mod_name, rs_obj.clone()).unwrap();

        Self {
            lua,
            mt_queue: Vec::new(),
            rs_obj,
        }
    }

    pub fn lua(&self) -> &Lua {
        self.lua
    }

    pub(crate) fn load(&self, code: &str) -> LuaResult<LuaFunction> {
        self.lua.load(code).into_function()
    }

    pub fn eval<T: FromLua<'l>>(&self, code: &str) -> LuaResult<T> {
        self.lua.load(code).eval()
    }

    pub fn exec(&self, code: &str) -> LuaResult<()> {
        self.lua.load(code).exec()
    }

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

        let dict = self.lua.create_table()?;
        for (name, func) in module.functions(self.lua)? {
            dict.set(name, func)?;
        }
        self.rs_obj.set(module.name(), dict)?;
        Ok(())
    }

    pub fn spawn<T: FromLua<'l> + Send + 'static>(
        &self,
        fn_: LuaFunction<'static>,
    ) -> LuaResult<JoinHandle<T>> {
        Ok(tokio::task::spawn_local(async move {
            fn_.call::<(), T>(()).unwrap()
        }))
    }

    pub fn run_frame(&mut self, max_tasks: Option<usize>) {
        if let Some(n) = max_tasks {
            for mt in self.mt_queue.drain(..n) {
                mt.call::<(), ()>(()).unwrap();
            }
        } else {
            for mt in self.mt_queue.drain(..) {
                mt.call::<(), ()>(()).unwrap();
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
