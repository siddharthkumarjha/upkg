use crate::lua::lua_types::*;
use crate::*;

pub fn extract<P>(pkg: &Package, _pkgbuild: P) -> LuaResult<()>
where
    P: AsRef<std::path::Path>,
{
    for src in &pkg.source.0 {
        match src.proto {
            Proto::git => (),
            Proto::url => {
                todo!("implement tarball extraction of data recvd from server");
            }
            Proto::file => (),
        };
    }
    Ok(())
}
