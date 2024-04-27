use mlua::prelude::*;

use super::LuaModule;

pub struct UtilModule;

impl LuaModule for UtilModule {
    fn name(&self) -> &'static str {
        "util"
    }

    fn init<'l>(&self, lua: &'l Lua, table: LuaTable<'l>) -> LuaResult<()> {
        table.set(
            "to_dbg_string",
            lua.create_function(|_, value: LuaValue| Ok(format!("{:#?}", value)))?,
        )?;
        Ok(())
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
    }
}
