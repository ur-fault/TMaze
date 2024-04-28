use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    lua_modules::{task::LuaTask, LuaModule},
    util::static_ref,
};

/// The runtime options for the Lua runtime
///
/// # Safety
///
/// Both of these options are unsafe because they can be used to load unsafe
/// modules into the Lua runtime. FFI module is enabled by default.
/// Same cannot be said for the debug module, which is disabled by default.
///
/// TODO: fix these docs
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

    /// Enable or disable the debug module, `unsafe`.
    /// Enable with caution. Disabled by default.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable or disable the FFI module, `unsafe`.
    /// Enable with caution. Enabled by default.
    pub fn ffi(mut self, ffi: bool) -> Self {
        self.ffi = ffi;
        self
    }

    /// Set the name of the runtime. Default is `tlua`.
    pub fn rt_name(mut self, rt_name: &'static str) -> Self {
        self.rt_name = rt_name;
        self
    }

    /// Enable or disable the default modules. Enabled by default.
    pub fn with_default_mods(mut self, with_default_mods: bool) -> Self {
        self.default_mods = with_default_mods;
        self
    }

    /// Convert the runtime options to the standard library options
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

/// The TLua runtime
///
/// This is the main struct that holds the Lua state and the queue of tasks.
/// You use this as the main entry point to interact with Lua. Internally,
/// it's `Rc` so that it can be cloned and passed around easily. It's not
/// [`Send`] nor [`Sync`] because [`mlua::Lua`] is not neither.
///
/// World would be so beautiful if it was, but it's not. :(
/// - ur-fault, 2024
///
/// It has it's own queue of tasks that are executed in the order they are
/// added. It's not a priority queue, so if you add a task that takes a long
/// time to execute, it will block the other tasks from executing.
///
/// None of the tasks are executed in parallel. They are executed in the order
/// they are added. You should, at all costs, avoid adding tasks that take a
/// long time to execute.
///
/// TODO: events and handlers
#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RefCell<RuntimeInner<'static>>>,
}

struct RuntimeInner<'l> {
    lua: &'static Lua,
    queue: Vec<Callable<'l>>,
    rs_obj: LuaTable<'l>,
    rt_name: String,
}

impl Runtime {
    // ** Creating a new instances of the runtime.
    // ** Althought, there should be only single instance,
    // ** since we leak the Lua state, to make it 'static.
    //
    /// Create a new runtime with the name of the app module.
    pub fn new(rs_mod_name: &str) -> Self {
        Self::from_lua(Lua::new(), rs_mod_name)
            .with_default_mods()
            .unwrap()
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

        let new = Self {
            inner: Rc::new(RefCell::new(RuntimeInner {
                lua,
                queue: Vec::new(),
                rs_obj,
                rt_name: options.rt_name.to_string(),
            })),
        };

        // for clarity
        return if options.default_mods {
            new.with_default_mods().unwrap()
        } else {
            new
        };
    }

    /// Enable default rust modules for that App. These modules are:
    /// - [`crate::lua_modules::fs::FsModule`] module
    /// - [`crate::lua_modules::util::UtilModule`] module
    /// - [`crate::lua_modules::task::TaskModule`] module
    ///
    /// - And the global module [`GlobalModule`]
    ///
    /// All of these modules are in the `self.rt_name` namespace
    pub fn with_default_mods(self) -> LuaResult<Self> {
        use crate::lua_modules::prelude::*;
        self.load_rs_module(FsModule)?;
        self.load_rs_module(UtilModule)?;
        self.load_rs_module(TaskModule::new(self.clone()))?;
        self.load_g_module(GlobalModule)?;

        Ok(self)
    }

    /// Create a new runtime from an existing Lua state.
    /// Not a reccoemnded way to create a new runtime.
    pub fn from_lua(lua: Lua, rs_mod_name: &str) -> Self {
        let lua = static_ref(lua);
        let rs_obj = lua.create_table().unwrap();
        lua.globals().set(rs_mod_name, rs_obj.clone()).unwrap();

        Self {
            inner: Rc::new(RefCell::new(RuntimeInner {
                lua,
                queue: Vec::new(),
                rs_obj,
                rt_name: rs_mod_name.to_string(),
            })),
        }
    }

    // ** Utils

    /// Returns the inner Lua state. It's a static reference.
    pub fn lua(&self) -> &'static Lua {
        self.inner.borrow().lua
    }

    // ** Loading and executing Lua code

    /// Load a Lua code and return it as a function, when called, it will
    /// execute the code.
    pub(crate) fn load(&self, code: &str) -> LuaResult<LuaFunction> {
        self.lua().load(code).into_function()
    }

    /// Evaluate a Lua code and return the result.
    pub fn eval<T: FromLuaMulti<'static>>(&self, code: &str) -> LuaResult<T> {
        self.lua().load(code).eval()
    }

    /// Execute a Lua code and return nothings. Same as [`Runtime::eval`] but
    /// returns `()`.
    pub fn exec(&self, code: &str) -> LuaResult<()> {
        self.lua().load(code).exec()
    }

    // ** Loading Rust modules

    /// Loads a Rust module into the Lua runtime. The module must implement
    /// the [`LuaModule`] trait. This function is similar to the [`Self::load_rs_module`],
    /// but it loads the module as a global module, meaning it's items are accessible
    /// from the `self.rt_name` namespace.
    pub fn load_g_module(&self, module: impl LuaModule + 'static) -> LuaResult<()> {
        let module = static_ref(module);

        module.init(self.clone(), self.inner.borrow().rs_obj.clone())?; // only the ref
        Ok(())
    }

    /// Loads a Rust module into the Lua runtime. The module must implement
    /// the [`LuaModule`] trait. This function is similar to the [`Self::load_g_module`],
    /// but it loads the module as a regular module, meaning it's items are accessible
    /// from the `self.rt_name`. `[LuaModule::name]` namespace.
    pub fn load_rs_module(&self, module: impl LuaModule + 'static) -> LuaResult<()> {
        let module = static_ref(module);

        let dict = self.lua().create_table()?;
        module.init(self.clone(), dict.clone())?; // only the ref
        self.inner.borrow_mut().rs_obj.set(module.name(), dict)?;
        Ok(())
    }

    // ** Control the runtime from Lua

    /// Queue a callback to be executed in the next frame.
    /// The callbacks are of the type [`Callable`].
    pub fn queue_callback(&self, callback: impl Into<Callable<'static>>) {
        // self.inner.borrow_mut().callback_queue.push(callback);
        self.inner.borrow_mut().queue.push(callback.into());
    }

    /// Returns the length of the queue of tasks.
    pub fn queue_len(&self) -> usize {
        self.inner.borrow().queue.len()
    }

    /// Run a single frame of the runtime. This will execute all the tasks,
    /// up to the `max_tasks` limit. If `max_tasks` is `None`, it will execute
    /// all the tasks in the queue.
    pub fn run_frame(&self, max_tasks: Option<usize>) {
        let mut new_queue = Vec::new();

        let task_count = max_tasks.unwrap_or(self.queue_len());
        if task_count == 0 {
            // println!("no tasks to run");
            return;
        }

        let mut processed = 0;
        while let Some(clb) = {
            // othewise we get a mutable borrow error
            // idk why, but it should imo work
            let mut bor = self.inner.borrow_mut();
            bor.queue.pop()
        } {
            self.inner.borrow();
            match clb {
                Callable::LuaFn(fnc) => fnc.call::<_, ()>(()).expect("error running task"),
                Callable::Rust(fnc) => fnc(self.lua()).expect("error running task"),
                Callable::LuaThread(thread) => {
                    thread.resume::<_, ()>(()).expect("error running task");
                    new_queue.push(Callable::LuaThread(thread));
                }
                Callable::LuaCode(code) => self.exec(&code).expect("error running task"),
                Callable::LuaTask(mut task) => {
                    if task.resume().is_none() {
                        new_queue.push(Callable::LuaTask(task));
                    }
                }
            }

            processed += 1;
            if processed >= task_count {
                break;
            }
        }

        self.inner.borrow_mut().queue.extend(new_queue);
    }
}

/// A callable object that can be executed in the runtime.
/// This is used to queue tasks to be executed in the next frame.
pub enum Callable<'lua> {
    LuaFn(LuaFunction<'lua>),
    Rust(Box<dyn FnOnce(&'lua Lua) -> LuaResult<()> + 'lua>),
    LuaThread(LuaThread<'lua>),
    LuaCode(String),
    LuaTask(LuaTask<'lua>),
}

/// A default global Rust module that provides some basic functions.
/// This module is loaded by default when creating a new runtime.
///
/// The module provides the following functions:
/// - `exit(status: number)`: Exits the process with the given status code.
pub struct GlobalModule;

impl LuaModule for GlobalModule {
    fn name(&self) -> &'static str {
        "global"
    }

    fn init(&self, rt: Runtime, dict: LuaTable) -> LuaResult<()> {
        dict.set(
            "exit",
            rt.lua()
                .create_function(|_, status: Option<i32>| -> LuaResult<()> {
                    std::process::exit(status.unwrap_or(0))
                })?,
        )?;

        Ok(())
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
