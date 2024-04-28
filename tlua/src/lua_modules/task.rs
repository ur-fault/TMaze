use std::{future::Future, time::Duration};

use futures::FutureExt;
use mlua::prelude::*;
use tokio::task::JoinHandle;

use crate::runtime::Runtime;

use super::LuaModule;

pub struct LuaTask<'l> {
    result: Option<LuaResult<LuaMultiValue<'l>>>,
    handle: Option<JoinHandle<LuaResult<LuaMultiValue<'l>>>>,
}

impl LuaTask<'static> {
    pub fn new(handle: JoinHandle<LuaResult<LuaMultiValue<'static>>>) -> Self {
        Self {
            result: None,
            handle: Some(handle),
        }
    }

    fn update(&mut self) {
        if self.result.is_none() {
            if self.handle.is_some() {
                let handle = self.handle.take().unwrap();
                if handle.is_finished() {
                    let res = handle.now_or_never().expect("task should be finished");
                    self.result = Some(res.expect("could not join task"));
                }
            }
        }
    }

    pub fn get_result(&mut self) -> Option<LuaResult<LuaMultiValue<'static>>> {
        self.update();
        self.result.take().clone()
    }
}

impl LuaUserData for LuaTask<'static> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // methods.add_async_method_mut("join", |_, this, _: ()| async { Ok(this.join().await?) });
        methods.add_method_mut(
            "res",
            |lua, this, _: ()| -> LuaResult<LuaMultiValue<'lua>> {
                // this.get_result().transpose()
                let res = this.get_result();
                if let Some(res) = res {
                    res
                } else {
                    LuaValue::Nil.into_lua_multi(lua)
                }
            },
        );
    }
}

/// Module for task management
pub struct TaskModule {
    rt: Runtime,
}

impl TaskModule {
    pub fn new(lua: Runtime) -> Self {
        Self { rt: lua }
    }
}

impl LuaModule for TaskModule {
    fn name(&self) -> &'static str {
        "task"
    }

    fn init<'l>(&'l self, rt: &'l Runtime, table: LuaTable<'l>) -> LuaResult<()> {
        table.set(
            "sleep",
            rt.lua().create_async_function(|_, dur: u64| async move {
                tokio::time::sleep(Duration::from_secs(dur)).await;
                Ok(())
            })?,
        )?;

        table.set(
            "spawn",
            rt.lua()
                .create_async_function(|_, func: LuaThread| async move {
                    let handle = tokio::task::spawn_local(func.into_async(()));
                    Ok(LuaTask::new(handle))
                })?,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_module() {
        let rt = Runtime::new("tlua");
        rt.load_rs_module(TaskModule { rt: rt.clone() }).unwrap();

        {
            let code = r#"
local task = tlua.task.spawn(function() return 2 end)
return coroutine.create(task.join)"#;
            let func = rt.load(code).unwrap();
            let result: Result<i32, _> = func.call_async(()).await;
            assert!(matches!(result, Ok(2)));
        }
    }
}
