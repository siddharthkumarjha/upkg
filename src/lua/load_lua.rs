use crate::*;
use mlua::prelude::*;

pub fn create_lua_instance() -> LuaResult<Lua> {
    let lua = Lua::new();
    lua_ok!(lua.sandbox(true));

    Ok(lua)
}

fn set_globals(lua: &Lua) -> LuaResult<()> {
    lua_ok!(
        lua.globals()
            .set("Proto", lua_ok!(Proto::global_lua_value(lua))),
        "setting global Proto table failed"
    );

    lua_ok!(
        lua.globals()
            .set("CheckSumKind", lua_ok!(CheckSumKind::global_lua_value(lua))),
        "setting global CheckSumKind table failed"
    );

    lua_ok!(
        lua.globals()
            .set("Skip", lua_ok!(CheckSumField::global_lua_value(lua))),
        "setting global Skip failed"
    );

    lua_ok!(
        lua.globals().set("InstallDir", LOCAL_INSTALL_PATH),
        "setting global InstallDir failed"
    );

    Ok(())
}

pub fn load_lua<ScriptPath: AsRef<Path>>(lua: &Lua, script_path: ScriptPath) -> LuaResult<()> {
    let script_path_utf8 = script_path.as_ref().to_string_lossy();
    let data = fs::read(script_path.as_ref()).map_err(lua_err_ctx!(script_path_utf8))?;

    lua_ok!(set_globals(lua));

    lua_ok!(
        lua.load(data).set_name(script_path_utf8.as_ref()).exec(),
        script_path_utf8
    );

    Ok(())
}
