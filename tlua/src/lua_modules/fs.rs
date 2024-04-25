use std::{collections::HashMap, hash::Hash};

use mlua::{prelude::*, Variadic};
use tokio::{
    io::AsyncReadExt,
    sync::{Mutex, MutexGuard},
};

use crate::check_eof;
use crate::util::ResultExt;

use super::LuaModule;

struct FsData {
    handles: HashMap<FileHandle, File>,
    new_handle_id: usize,
    reuse_ids: Vec<usize>,
}

impl FsData {
    fn new() -> Self {
        Self {
            handles: HashMap::new(),
            new_handle_id: 0,
            reuse_ids: Vec::new(),
        }
    }
}

impl FsData {
    async fn new_id(&mut self) -> Result<usize, mlua::Error> {
        if let Some(id) = self.reuse_ids.pop() {
            return Ok(id);
        }

        let id = self.new_handle_id;
        self.new_handle_id += 1;
        Ok(id)
    }

    async fn free_id(&mut self, id: usize) -> Result<(), mlua::Error> {
        self.reuse_ids.push(id);
        Ok(())
    }
}

pub struct File {
    inner: tokio::fs::File,
}

impl File {
    pub async fn read(&mut self, format: Variadic<FileReadFormat>) -> LuaResult<FileReadResult> {
        todo!()
    }

    async fn read_number(&mut self) -> LuaResult<Option<f64>> {
        let res = self.inner.read_f64().await;
        let res = check_eof!(res);

        Ok(Some(res))
    }

    async fn read_line(&mut self) -> LuaResult<Option<Vec<u8>>> {
        let mut buf = Vec::new();
        loop {
            let res = self.inner.read_u8().await;
            let byte = check_eof!(res, res, Some(res), None);

            match byte {
                Some(b'\n') => {
                    buf.push(b'\n');
                    break;
                }
                Some(byte) => buf.push(byte),
                None if buf.is_empty() => return Ok(None),
                None => break,
            }
        }

        Ok(Some(buf))
    }
}

#[derive(Clone)]
pub struct FileHandle {
    id: usize,
    module: &'static FsModule,
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileHandle").field("id", &self.id).finish()
    }
}

impl PartialEq for FileHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FileHandle {}

impl Hash for FileHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'lua> FromLua<'lua> for FileHandle {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        let Some(data) = value.as_userdata() else {
            return Err(mlua::Error::external("not a handle"));
        };

        data.borrow().map(|data| FileHandle::clone(&data))
    }
}

impl LuaUserData for FileHandle {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("read", |_, this: &Self, ()| async move {
            let mut buf = String::new();
            this.module
                .fs
                .lock()
                .await
                .handles
                .get_mut(&this)
                .ok_or(|| ())
                .map_text_err("invalid handle")?
                .inner
                .read_to_string(&mut buf)
                .await?;
            Ok(buf)
            // Ok(())
        });

        // methods.add_async_method("")
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        let self2 = self.clone();
        tokio::spawn(async move {
            let mut fs = self2.module.lock_fs().await;
            fs.handles.remove(&self2);
            fs.free_id(self2.id).await.unwrap();
        });
    }
}

pub struct FsModule {
    fs: Mutex<FsData>,
}

impl FsModule {
    pub fn new() -> Self {
        Self {
            fs: Mutex::new(FsData::new()),
        }
    }

    async fn lock_fs(&self) -> MutexGuard<FsData> {
        self.fs.lock().await
    }
}

impl LuaModule for FsModule {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn functions(
        &'static self,
        lua: &'static Lua,
    ) -> LuaResult<Vec<(&'static str, LuaFunction<'static>)>> {
        Ok(vec![(
            "open",
            lua.create_async_function(move |_, path: String| async move {
                let file = tokio::fs::File::open(path).await?;
                let mut fs = self.lock_fs().await;

                let id = fs.new_id().await?;
                fs.handles
                    .insert(FileHandle { id, module: self }, File { inner: file });
                Ok(FileHandle {
                    id: 0,
                    module: self,
                })
            })?,
        )])
    }
}

enum FileReadFormat {
    Count,
    Line,
    All,
    Number(usize),
}

impl FromLua<'_> for FileReadFormat {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match value {
            // s is a LuaString, potentionally not u UTF8 string
            LuaValue::String(s) => match s.to_str()? {
                "*n" => Ok(Self::Count),
                "*l" => Ok(Self::Line),
                "*a" => Ok(Self::All),
                _ => Err(mlua::Error::external("invalid format")),
            },
            LuaValue::Integer(n) => Ok(Self::Number(n as usize)),
            _ => Err(mlua::Error::external("invalid format")),
        }
    }
}

enum FileReadResult {
    Text(Vec<u8>),
    Number(f64),
    Nil,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        runtime::Runtime,
        util::{self, block_on},
    };

    #[test]
    #[ignore = "global io file functions are deprecated, alas removed"]
    fn test_fs_module() {
        util::block_on(async {
            let rt = Runtime::new("tlua");
            rt.load_rs_module(FsModule::new()).unwrap();

            let code = "return coroutine.create(tlua.fs.read_to_string)";
            let func = rt.load(code).unwrap();
            let thread: LuaThread = func.call(()).expect("failed to call function");
            let result: String = thread
                .into_async("Cargo.toml")
                .await
                .expect("failed to block on coroutine");
            assert!(result.contains("[package]"));
        });
    }

    #[test]
    fn test_file_userdata() {
        util::block_on(async {
            let rt = Runtime::new("tlua");
            rt.load_rs_module(FsModule::new()).unwrap();

            let code = r#"
return coroutine.create(function()
    local handle = tlua.fs.open('Cargo.toml')
    return handle:read()
end)"#;
            let thread: LuaAsyncThread<String> = rt.eval::<LuaThread>(code).unwrap().into_async(());
            let content = thread.await;
            dbg!(&content);
            assert!(content.is_ok());
            assert!(content.unwrap().contains("[package]"));
        });
    }

    #[test]
    fn test_ids() {
        block_on(async {
            let mut fs = FsData::new();

            let id1 = fs.new_id().await.unwrap();
            assert_eq!(id1, 0);
            assert_eq!(fs.new_id().await.unwrap(), 1);
            fs.free_id(id1).await.unwrap();
            assert_eq!(fs.new_id().await.unwrap(), 0);
            assert_eq!(fs.new_id().await.unwrap(), 2);
            assert_eq!(fs.new_id().await.unwrap(), 3);
            fs.free_id(2).await.unwrap();
            assert_eq!(fs.new_id().await.unwrap(), 2);
        });
    }
}
