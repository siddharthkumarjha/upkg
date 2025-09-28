mod err_context;
mod lua;
mod proto;
mod sub_path;

use crate::lua::load_lua::*;
use crate::lua::lua_types::*;
use crate::proto::*;
use crate::sub_path::*;

use mlua::prelude::*;

use std::fs;

static LOCAL_INSTALL_PATH: &str = "/home/siddharth/tst/";

fn upkg() -> LuaResult<()> {
    let lua = lua_ok!(create_lua_instance());

    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pkgbuild = root_path.join("test-pkg/starship/pkgbuild.lua");

    if io_ok!(pkgbuild.is_subpath_of(root_path)) {
        lua_ok!(load_lua(&lua, &pkgbuild));

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
                Proto::url => {}
                Proto::file => {
                    println!("got loc: {}", src.location);
                }
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
