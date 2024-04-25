use bstr::BString;
use mlua::{prelude::*, Variadic};
use tokio::{fs::File as TkFile, io::AsyncReadExt};

use crate::{check_eof, util};

use super::LuaModule;

pub struct LuaFile {
    file: TkFile,
}

impl LuaUserData for LuaFile {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method_mut("read", |_, this: &mut Self, ()| async move {
            let mut buf = Vec::new();
            this.file.read_to_end(&mut buf).await?;

            // Reminder so that I don't forget:
            // lua doesn't need null-terminated strings,
            // so we can just return the Vec<u8> directly.
            // https://www.lua.org/manual/5.1/manual.html#lua_pushlstring
            Ok(BString::new(buf))
        });
    }
}

impl LuaFile {
    pub async fn read(&mut self, format: Variadic<FileReadFormat>) -> LuaResult<FileReadResult> {
        todo!()
    }

    pub async fn read_by_format(
        &mut self,
        format: FileReadFormat,
    ) -> LuaResult<Option<FileReadResult>> {
        let res = match format {
            FileReadFormat::Line => self.read_line().await?.map(FileReadResult::Text),
            FileReadFormat::Number => self.read_number().await?.map(FileReadResult::Number),
            _ => todo!(),
        };

        Ok(res)
    }

    async fn read_number(&mut self) -> LuaResult<Option<f64>> {
        let mut buf = Vec::new();
        let res = util::streams::read_dec_float(&mut buf, &mut self.file).await;
        check_eof!(res);

        Ok(Some(
            String::from_utf8(buf)
                .into_lua_err()?
                .parse()
                .into_lua_err()?,
        ))
    }

    async fn read_line(&mut self) -> LuaResult<Option<Vec<u8>>> {
        let mut buf = Vec::new();
        loop {
            let res = self.file.read_u8().await;
            let byte = check_eof!(res, res => Some(res), eof None);

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

pub struct FsModule;

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
                let file = TkFile::open(path).await?;
                Ok(LuaFile { file })
            })?,
        )])
    }
}

enum FileReadFormat {
    Number,
    Line,
    All,
    Count(usize),
}

impl FromLua<'_> for FileReadFormat {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match value {
            // s is a LuaString, potentionally not u UTF8 string
            LuaValue::String(s) => match s.as_bytes() {
                b"*n" => Ok(Self::Number),
                b"*l" => Ok(Self::Line),
                b"*a" => Ok(Self::All),
                _ => Err(mlua::Error::external("invalid format")),
            },
            LuaValue::Integer(n) => Ok(Self::Count(n as usize)),
            _ => Err(mlua::Error::external("invalid format")),
        }
    }
}

enum FileReadResult {
    Text(Vec<u8>),
    Number(f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;

    #[ignore = "global io file functions are deprecated, alas removed"]
    #[tokio::test]
    async fn test_fs_module() {
        let rt = Runtime::new("tlua");
        rt.load_rs_module(FsModule).unwrap();

        let code = "return coroutine.create(tlua.fs.read_to_string)";
        let func = rt.load(code).unwrap();
        let thread: LuaThread = func.call(()).expect("failed to call function");
        let result: String = thread
            .into_async("Cargo.toml")
            .await
            .expect("failed to block on coroutine");
        assert!(result.contains("[package]"));
    }

    #[tokio::test]
    async fn test_file_userdata() {
        let rt = Runtime::new("tlua");
        rt.load_rs_module(FsModule).unwrap();

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
    }
}
