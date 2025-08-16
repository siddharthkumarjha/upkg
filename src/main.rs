use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use strum::{EnumIter, IntoEnumIterator};

macro_rules! lua_err_context {
    () => {
        |_| LuaError::external(format!("[{}:{}]", file!(), line!()))
    };

    ($arg:expr) => {
        |_| LuaError::external(format!("[{}:{}]: {}", file!(), line!(), $arg))
    };

    ($fmt:expr, $($arg:tt)*) => {
        |_| LuaError::external(format!(
            "[{}:{}] {}",
            file!(), line!(),
            format!($fmt, $($arg)*)
        ))
    };
}

macro_rules! io_err_context {
    () => {
        |err| std::io::Error::new(err.kind(), format!("[{}:{}] {}", file!(), line!(), err))
    };

    ($arg:expr) => {
        |err| std::io::Error::new(err.kind(), format!("[{}:{}]: {} {}", file!(), line!(), $arg, err))
    };

    ($fmt:expr, $($arg:tt)*) => {
        |err| std::io::Error::new(err.kind(), format!(
            "[{}:{}] {} {}",
            file!(), line!(),
            format!($fmt, $($arg)*),
            err
        ))
    };
}

trait SubPath {
    fn is_subpath_of<P: AsRef<Path>>(&self, base: P) -> std::io::Result<bool>;
}

impl SubPath for Path {
    fn is_subpath_of<P: AsRef<Path>>(&self, base: P) -> std::io::Result<bool> {
        let base_canon = base
            .as_ref()
            .canonicalize()
            .map_err(io_err_context!(base.as_ref().to_string_lossy()))?;
        let child_canon = self
            .canonicalize()
            .map_err(io_err_context!(self.to_string_lossy()))?;

        if child_canon.starts_with(base_canon) {
            return Ok(true);
        }
        Ok(false)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PkgInfo {
    name: String,
    desc: String,
    ver: String,
    rel: Option<u32>,
}

#[derive(EnumIter, Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
enum Proto {
    git,
    patch,
}

fn create_lua_instance() -> LuaResult<Lua> {
    let lua = Lua::new();
    lua.sandbox(true).with_context(lua_err_context!())?;

    Ok(lua)
}

fn build_proto_table(lua: &Lua) -> LuaResult<LuaTable> {
    let proto_table = lua.create_table()?;

    for proto_type in Proto::iter() {
        let lua_val = lua
            .to_value(&proto_type)
            .with_context(lua_err_context!("{:?}", proto_type))?;

        proto_table.set(lua_val.clone(), lua_val)?;
    }

    Ok(proto_table)
}

fn set_globals(lua: &Lua) -> LuaResult<()> {
    lua.globals()
        .set("Proto", build_proto_table(lua)?)
        .with_context(lua_err_context!("setting global Proto table failed"))?;

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

        let pkg_info: PkgInfo = lua
            .from_value(
                lua.globals()
                    .get("PkgInfo")
                    .with_context(lua_err_context!())?,
            )
            .with_context(lua_err_context!())?;
        println!("{:#?}", pkg_info);

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
