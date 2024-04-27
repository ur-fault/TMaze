use std::future::Future;

use mlua::prelude::*;
use tokio::task::{self, JoinHandle};

pub struct LuaTask<'l> {
    handle: JoinHandle<LuaMultiValue<'l>>,
}

impl LuaTask<'static> {
    pub fn new(handle: JoinHandle<LuaMultiValue<'static>>) -> Self {
        Self { handle }
    }

    pub fn spawn<F>(fut: F) -> Self
    where
        F: Future<Output = LuaMultiValue<'static>> + 'static,
    {
        Self {
            handle: task::spawn_local(fut),
        }
    }

    pub async fn join(self) -> LuaResult<LuaMultiValue<'static>> {
        let values = self.handle.await.map_err(LuaError::external)?;
        Ok(values)
    }
}
