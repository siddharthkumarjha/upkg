use crate::lua_err_context;

use mlua::prelude::*;
use serde::{Deserialize, Serialize};

pub use strum::{EnumIter, IntoEnumIterator};

pub trait LuaGTableValue {
    fn global_lua_value(lua: &Lua) -> LuaResult<impl IntoLua>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PkgInfo {
    name: String,
    ver: String,
    #[serde(default)]
    rel: Option<u32>,
    desc: String,
}

#[derive(EnumIter, Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum Proto {
    git,
    url,
    file
}

impl LuaGTableValue for Proto {
    fn global_lua_value(lua: &Lua) -> LuaResult<impl IntoLua> {
        let proto_table = lua
            .create_table()
            .with_context(lua_err_context!("proto table"))?;

        for proto_type in Proto::iter() {
            let lua_val = lua
                .to_value(&proto_type)
                .with_context(lua_err_context!("{:?}", proto_type))?;

            proto_table
                .set(lua_val.clone(), lua_val)
                .with_context(lua_err_context!("{:?}", proto_type))?;
        }

        Ok(proto_table)
    }
}

#[derive(EnumIter, Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum CheckSumKind {
    sha256,
    sha512,
}

impl LuaGTableValue for CheckSumKind {
    fn global_lua_value(lua: &Lua) -> LuaResult<impl IntoLua> {
        let checksum_kind_table = lua
            .create_table()
            .with_context(lua_err_context!("checksumkind table"))?;

        for checksum_kind in CheckSumKind::iter() {
            let lua_val = lua
                .to_value(&checksum_kind)
                .with_context(lua_err_context!("{:?}", checksum_kind))?;

            checksum_kind_table
                .set(lua_val.clone(), lua_val)
                .with_context(lua_err_context!("{:?}", checksum_kind))?;
        }

        Ok(checksum_kind_table)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum CheckSumField {
    Skip,
    Value { kind: CheckSumKind, digest: String },
}

impl LuaGTableValue for CheckSumField {
    fn global_lua_value(lua: &Lua) -> LuaResult<impl IntoLua> {
        let none_type = CheckSumField::Skip;
        let lua_val = lua
            .to_value(&none_type)
            .with_context(lua_err_context!("{:?}", none_type))?;

        Ok(lua_val)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct CheckSum(Vec<CheckSumField>);

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceField {
    proto: Proto,

    #[serde(alias = "url", alias = "file")]
    location: String,

    #[serde(alias = "tag", alias = "branch")]
    #[serde(default)]
    checkout: Option<String>,

    #[serde(default)]
    directory: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Source(Vec<SourceField>);

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum DepInfo {
    Full {
        name: String,
        #[serde(default)]
        ver: Option<String>,
        #[serde(default)]
        rel: Option<u32>,
        #[serde(default)]
        desc: Option<String>,
    },
    Simple(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pkg: PkgInfo,
    #[serde(default)]
    url: String,

    #[serde(default)]
    license: Vec<String>,
    #[serde(default)]
    groups: Vec<String>,

    #[serde(default)]
    provides: Vec<DepInfo>,

    depends: Vec<DepInfo>,
    #[serde(default)]
    opt_depends: Vec<DepInfo>,
    #[serde(default)]
    check_depends: Vec<DepInfo>,
    #[serde(default)]
    make_depends: Vec<DepInfo>,

    #[serde(default)]
    conflicts: Vec<DepInfo>,
    #[serde(default)]
    replaces: Vec<DepInfo>,

    source: Source,
    checksum: CheckSum,
}
