use crate::lua_err_context;

use mlua::prelude::*;
use serde::{Deserialize, Serialize};

pub use strum::{EnumIter, IntoEnumIterator};

pub trait LuaGTableValue {
    fn global_lua_value(lua: &Lua) -> LuaResult<impl IntoLua>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PkgInfo {
    pub name: String,
    pub ver: String,
    #[serde(default)]
    pub rel: Option<u32>,
    pub desc: String,
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
pub struct CheckSum(pub Vec<CheckSumField>);

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceField {
    pub proto: Proto,

    #[serde(alias = "url", alias = "file")]
    pub location: String,

    #[serde(alias = "tag", alias = "branch")]
    #[serde(default)]
    pub checkout: Option<String>,

    #[serde(default)]
    pub directory: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Source(pub Vec<SourceField>);

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
    pub pkg: PkgInfo,
    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub license: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub provides: Vec<DepInfo>,

    pub depends: Vec<DepInfo>,
    #[serde(default)]
    pub opt_depends: Vec<DepInfo>,
    #[serde(default)]
    pub check_depends: Vec<DepInfo>,
    #[serde(default)]
    pub make_depends: Vec<DepInfo>,

    #[serde(default)]
    pub conflicts: Vec<DepInfo>,
    #[serde(default)]
    pub replaces: Vec<DepInfo>,

    pub source: Source,
    pub checksum: CheckSum,
}
