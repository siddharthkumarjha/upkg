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

fn load_lua<ScriptPath: AsRef<Path>>(lua: &Lua, script_path: ScriptPath) -> LuaResult<()> {
    let script_path_utf8 = script_path.as_ref().to_string_lossy();
    let data = fs::read(script_path.as_ref()).map_err(lua_err_ctx!(script_path_utf8))?;

    set_globals(lua)?;

    lua_ok!(
        lua.load(data).set_name(script_path_utf8.as_ref()).exec(),
        script_path_utf8
    );

    Ok(())
}

fn upkg() -> LuaResult<()> {
    let lua = create_lua_instance()?;

    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pkgbuild = root_path.join("test-pkg/starship/pkgbuild.lua");

    if pkgbuild.is_subpath_of(root_path)? {
        load_lua(&lua, &pkgbuild)?;

        let pkg: Package = lua_ok!(lua.from_value(lua_ok!(lua.globals().get("Package"))));
        println!("{:#?}", pkg);

        for src in pkg.source.0 {
            match src.proto {
                Proto::git => {
                    let url = src.location;
                    let build_dir = pkgbuild
                        .parent()
                        .ok_or_else(|| {
                            LuaError::external(format!(
                                "[{}:{}] couldn't evaluate parent path of: {}",
                                file!(),
                                line!(),
                                pkgbuild.to_string_lossy()
                            ))
                        })?
                        .join("build");

                    if !build_dir.exists() {
                        io_ok!(fs::create_dir(&build_dir), build_dir.to_string_lossy());
                    }

                    git_2_lua_ok!(git_clone::git_sync_with_remote(
                        &url,
                        build_dir,
                        src.repo_name.as_deref(),
                        src.checkout
                    ));
                }
                _ => println!("got loc: {}", src.location),
            }
        }

        Ok(())
    } else {
        Err(LuaError::external(format!(
            "[{}:{}] {} is not a subpath of {}",
            file!(),
            line!(),
            pkgbuild.to_string_lossy(),
            root_path.to_string_lossy()
        )))
    }
}

fn main() {
    match upkg() {
        Ok(_) => {}
        Err(msg) => println!("{}", msg),
    }
}
