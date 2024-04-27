use bstr::BString;
use mlua::{prelude::*, Variadic};
use tokio::{
    fs::File as TkFile,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream},
};

use crate::{check_eof, util};

use super::LuaModule;

pub struct LuaFile {
    file: BufStream<TkFile>,
}

impl LuaUserData for LuaFile {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method_mut("read", |_, this, formats| async {
            this.read(formats).await
        });

        methods.add_async_method_mut("close", |_, this, ()| async {
            this.file.shutdown().await?;
            Ok(())
        });

        methods.add_async_method_mut("flush", |_, this, ()| async {
            this.file.flush().await?;
            Ok(())
        });

        methods.add_async_method_mut("write", |_, this, data: BString| async move {
            this.file.write_all(&data).await?;
            Ok(())
        });

        methods.add_async_method_mut("lines", |_, this, on_line: LuaFunction<'lua>| async move {
            while let Some(mut line) = this.read_line().await? {
                if line.last() == Some(&b'\n') {
                    line.pop();
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                }
                on_line.call(line)?;
                // println!("{}", line);
            }

            Ok(())
        });
    }
}

impl LuaFile {
    pub async fn read(
        &mut self,
        formats: Variadic<FileReadFormat>,
    ) -> LuaResult<Variadic<Option<FileReadResult>>> {
        let mut results = Variadic::new();

        if formats.is_empty() {
            results.push(self.read_line().await?.map(FileReadResult::Text));
            return Ok(results);
        }

        for format in formats {
            // `Option::is_some` basically, but funnier
            if let result @ Some(..) = self.read_by_format(format).await? {
                results.push(result);
            } else {
                break;
            }
        }

        Ok(results)
    }

    pub async fn read_by_format(
        &mut self,
        format: FileReadFormat,
    ) -> LuaResult<Option<FileReadResult>> {
        let res = match format {
            FileReadFormat::Line => self.read_line().await?.map(FileReadResult::Text),
            FileReadFormat::Number => self.read_number().await?.map(FileReadResult::Number),
            FileReadFormat::Count(count) => self.read_count(count).await?.map(FileReadResult::Text),
            // all cannot return `None`
            FileReadFormat::All => Some(self.read_all().await?).map(FileReadResult::Text),
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

    async fn read_line(&mut self) -> LuaResult<Option<BString>> {
        let mut buf = Vec::new();
        let read = self.file.read_until('\n' as u8, &mut buf).await?;
        if read == 0 {
            return Ok(None);
        } else {
            Ok(Some(BString::new(buf)))
        }
    }

    async fn read_all(&mut self) -> LuaResult<BString> {
        let mut buf = Vec::new();
        self.file.read_to_end(&mut buf).await?;
        Ok(BString::new(buf))
    }

    async fn read_count(&mut self, count: usize) -> LuaResult<Option<BString>> {
        let mut buf = vec![0; count];
        check_eof!(self.file.read_exact(&mut buf).await);
        Ok(Some(BString::new(buf)))
    }
}

pub struct FsModule;

impl LuaModule for FsModule {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn init<'l>(&self, lua: &'l Lua, table: LuaTable<'l>) -> LuaResult<()> {
        table.set(
            "open",
            lua.create_async_function(|_, path: String| async move {
                let file = BufStream::new(TkFile::open(path).await?);
                Ok(LuaFile { file })
            })?,
        )?;
        Ok(())
    }
}

pub enum FileReadFormat {
    Number,
    Line,
    All,
    Count(usize),
}

impl FromLua<'_> for FileReadFormat {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
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

pub enum FileReadResult {
    Text(BString),
    Number(f64),
}

impl IntoLua<'_> for FileReadResult {
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue<'_>> {
        match self {
            Self::Text(text) => Ok(text.into_lua(lua)?),
            Self::Number(n) => Ok(n.into_lua(lua)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lua_modules::util::UtilModule, runtime::Runtime};

    #[ignore = "global io file functions are removed"]
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
    async fn test_file_lines() {
        let rt = Runtime::new("tlua");
        rt.load_rs_module(FsModule).unwrap();
        rt.load_rs_module(UtilModule).unwrap();

        let code = r#"
return coroutine.create(function()
    local ahandle = tlua.fs.open('Cargo.toml')
    local alines = {}
    ahandle:lines(function(line)
        table.insert(alines, line)
    end)

    local shandle = io.open('Cargo.toml')
    local slines = {}
    for line in shandle:lines() do
        table.insert(slines, line)
    end

    return table.concat(alines, '\n'), table.concat(slines, '\n')
end)"#;
        let thread: LuaAsyncThread<(BString, BString)> =
            rt.eval::<LuaThread>(code).unwrap().into_async(());
        let (at, st) = thread.await.unwrap();
        assert_eq!(at.len(), st.len());
        assert_eq!(at, st);
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
        assert!(content.is_ok());
        assert!(dbg!(content).unwrap().contains("[package]"));
    }
}
