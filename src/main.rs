mod err_context;
mod lua;
mod proto;
mod sub_path;
mod upkg;

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

        let total_steps = 7;
        let mut current_step = 1;
        println!("({}/{}) Downloading Deps", current_step, total_steps);
        lua_ok!(upkg::download_deps::download(&pkg, &pkgbuild));

        current_step = current_step + 1;
        println!("({}/{}) Verifying Deps", current_step, total_steps);
        lua_ok!(upkg::verify_deps::verify(&pkg, &pkgbuild));

        current_step = current_step + 1;
        println!("({}/{}) Extracting Deps", current_step, total_steps);
        lua_ok!(upkg::extract_deps::extract(&pkg, &pkgbuild));

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
