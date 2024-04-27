mod fs;
mod util;
mod task;

use mlua::prelude::*;

pub trait LuaModule {
    /// Returns the name of the module.
    ///
    /// In most cases, it should be compile time constant. But it coulde be
    /// dynamic if you want to create multiple instances of the same module,
    /// with different configurations.
    fn name(&'static self) -> &'static str;

    /// Initializes the module.
    ///
    /// This method is called when the module is loaded. It should register
    /// all the functions and global variables that the module provides.
    fn init<'l>(&self, lua: &'l Lua, table: LuaTable<'l>) -> LuaResult<()>;
}
