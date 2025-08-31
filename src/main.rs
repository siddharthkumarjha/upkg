mod err_context;
mod git_clone;
mod lua_types;
mod sub_path;

use crate::lua_types::*;
use crate::sub_path::*;

use mlua::prelude::*;

use std::fs;

static LOCAL_INSTALL_PATH: &str = "/home/siddharth/tst/";

fn create_lua_instance() -> LuaResult<Lua> {
    let lua = Lua::new();
    lua.sandbox(true).with_context(lua_err_context!())?;

    Ok(lua)
}

fn set_globals(lua: &Lua) -> LuaResult<()> {
    lua.globals()
        .set("Proto", Proto::global_lua_value(lua)?)
        .with_context(lua_err_context!("setting global Proto table failed"))?;

    lua.globals()
        .set("CheckSumKind", CheckSumKind::global_lua_value(lua)?)
        .with_context(lua_err_context!("setting global CheckSumKind table failed"))?;

    lua.globals()
        .set("Skip", CheckSumField::global_lua_value(lua)?)
        .with_context(lua_err_context!("setting global Skip failed"))?;

    lua.globals()
        .set("InstallDir", LOCAL_INSTALL_PATH)
        .with_context(lua_err_context!("setting global InstallDir failed"))?;

    Ok(())
}

fn load_lua<ScriptPath: AsRef<Path>>(lua: &Lua, script_path: ScriptPath) -> LuaResult<()> {
    let script_path_utf8 = script_path.as_ref().to_string_lossy();
    let data = fs::read(script_path.as_ref()).map_err(lua_err_context!(script_path_utf8))?;

    set_globals(lua)?;

    lua.load(data)
        .set_name(script_path_utf8.as_ref())
        .exec()
        .with_context(lua_err_context!(script_path_utf8))?;

    Ok(())
}

fn upkg() -> LuaResult<()> {
    let lua = create_lua_instance()?;

    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let starship_pkgbuild = root_path.join("test-pkg/starship/pkgbuild.lua");

    if starship_pkgbuild.is_subpath_of(root_path)? {
        load_lua(&lua, starship_pkgbuild)?;

        let pkg: Package = lua
            .from_value(
                lua.globals()
                    .get("Package")
                    .with_context(lua_err_context!())?,
            )
            .with_context(lua_err_context!())?;
        println!("{:#?}", &pkg);

        for s in pkg.source.0 {
            match s.proto {
                Proto::git => {
                    let url = s.location;
                    match git_clone::git_clone(&url, std::env::temp_dir(), None) {
                        Ok(r) => r,
                        Err(e) => panic!("failed to clone {}", e),
                    };
                }
                _ => println!("got loc: {}", s.location),
            }
        }

        Ok(())
    } else {
        Err(lua_err_context!(
            "{} is not a subpath of {}",
            starship_pkgbuild.to_string_lossy(),
            root_path.to_string_lossy()
        )(()))
    }
}

fn main() {
    match upkg() {
        Ok(_) => {}
        Err(msg) => println!("{}", msg),
    }
}
