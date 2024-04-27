use std::time::Duration;

use mlua::prelude::*;

use super::LuaModule;

pub struct TaskModule;

impl LuaModule for TaskModule {
    fn name(&self) -> &'static str {
        "task"
    }

    fn init<'l>(&self, lua: &'l Lua, table: LuaTable<'l>) -> LuaResult<()> {
        table.set(
            "sleep",
            lua.create_async_function(|_, dur: u64| async move {
                tokio::time::sleep(Duration::from_secs(dur)).await;
                Ok(())
            })?,
        )?;
        Ok(())
    }
}
