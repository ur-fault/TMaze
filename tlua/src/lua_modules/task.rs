use std::time::Duration;

use mlua::prelude::*;

use crate::runtime::{Callable, Runtime};

use super::LuaModule;

pub struct LuaTask<'l> {
    result: Option<LuaResult<LuaMultiValue<'l>>>,
    handle: Option<LuaThread<'l>>,
    args: Option<LuaMultiValue<'l>>,
}

impl LuaTask<'static> {
    pub fn new(handle: LuaThread<'static>, args: LuaMultiValue<'static>) -> Self {
        Self {
            result: None,
            handle: Some(handle),
            args: Some(args),
        }
    }

    pub fn update(&mut self) {
        // our work here is done
        if self.handle.is_none() {
            return;
        }

        let handle = self.handle.take().unwrap();
        let result = if let Some(args) = self.args.take() {
            // result being None, means we haven't called resume yet
            // so we need to pass the first args
            handle.resume(args)
        } else {
            handle.resume(())
        };

        // we ignore the yield return
        match result {
            Err(LuaError::CoroutineInactive) => {}
            // return the handle if the coroutine is still active
            _ => self.handle = Some(handle),
        }
        self.result = Some(result);
    }

    pub fn resume(&mut self) -> Option<LuaResult<LuaMultiValue<'static>>> {
        self.update();
        // if the handle is None, then the task is done,
        // and in `self.result` we have the final result
        if self.handle.is_none() {
            self.result.clone()
        } else {
            None
        }
    }
}

impl LuaUserData for LuaTask<'static> {
    fn add_methods<'l, M: LuaUserDataMethods<'l, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "resume",
            |lua, this, _: ()| -> LuaResult<LuaMultiValue<'l>> {
                let res = this.resume();
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

    fn init(&self, rt: Runtime, table: LuaTable<'static>) -> LuaResult<()> {
        table.set(
            "sleep",
            rt.lua().create_async_function(|_, dur: u64| async move {
                tokio::time::sleep(Duration::from_secs(dur)).await;
                Ok(())
            })?,
        )?;

        let rt2 = self.rt.clone();
        table.set(
            "spawn",
            rt.lua()
                .create_function(move |lua, (func, args): (LuaFunction, LuaMultiValue)| {
                    let thread = lua.create_thread(func)?;
                    rt2.queue_callback(Callable::LuaTask(LuaTask::new(thread.clone(), args)));
                    Ok(thread)
                })?,
        )?;

        table.set(
            "queue",
            rt.lua()
                .create_function(move |_, fnc: LuaFunction<'static>| {
                    rt.queue_callback(Callable::LuaFn(fnc));
                    Ok(())
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
