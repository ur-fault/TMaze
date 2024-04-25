mod fs;

use std::time::Duration;

use mlua::prelude::*;

pub trait LuaModule {
    fn name(&'static self) -> &'static str;
    fn functions(
        &'static self,
        _: &'static Lua,
    ) -> LuaResult<Vec<(&'static str, LuaFunction<'static>)>> {
        Ok(vec![])
    }
}

pub struct UtilModule;

impl LuaModule for UtilModule {
    fn name(&self) -> &'static str {
        "util"
    }

    fn functions<'l>(&self, lua: &'l Lua) -> LuaResult<Vec<(&'static str, LuaFunction<'l>)>> {
        Ok(vec![
            (
                "to_dbg_string",
                lua.create_function(|_, value: LuaValue| Ok(format!("{:#?}", value)))?,
            ),
            (
                "sleep",
                lua.create_async_function(|_, dur: u64| async move {
                    tokio::time::sleep(Duration::from_secs(dur)).await;
                    Ok(())
                })?,
            ),
        ])
    }
}

pub struct TaskModule;

impl LuaModule for TaskModule {
    fn name(&self) -> &'static str {
        "task"
    }

    fn functions<'l>(&self, lua: &'l Lua) -> LuaResult<Vec<(&'static str, LuaFunction<'l>)>> {
        Ok(vec![
            (
                "sleep",
                lua.create_async_function(|_, dur: u64| async move {
                    tokio::time::sleep(Duration::from_secs(dur)).await;
                    Ok(())
                })?,
            ),
            (
                "ready",
                lua.create_async_function(|_, value: LuaValue| async move { Ok(value) })?,
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::Runtime, util::block_on};

    use super::*;

    #[test]
    fn test_util_module() {
        let rt = Runtime::new("tlua");
        rt.load_rs_module(UtilModule).unwrap();

        {
            let code = "return tlua.util.to_dbg_string(tlua)";
            let func = rt.load(code).unwrap();
            let result: Result<String, _> = func.call(());
            assert!(matches!(result, Ok(_)));
        }

        block_on(async {
            let code = "return coroutine.create(tlua.util.sleep)";
            let func = rt.load(code).unwrap();
            let thread: LuaThread = func.call(()).expect("failed to call function");
            thread.into_async::<_, ()>(3).await.unwrap();
        });
    }
}
