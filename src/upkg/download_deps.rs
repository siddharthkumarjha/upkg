use crate::lua::lua_types::*;
use crate::*;

pub fn download<P: AsRef<std::path::Path>>(pkg: &Package, pkgbuild: P) -> LuaResult<()> {
    for src in &pkg.source.0 {
        match src.proto {
            Proto::git => {
                let url = &src.location;
                let build_dir = pkgbuild
                    .as_ref()
                    .parent()
                    .ok_or_else(|| {
                        LuaError::external(format!(
                            "[{}:{}] couldn't evaluate parent path of: {}",
                            file!(),
                            line!(),
                            pkgbuild.as_ref().to_string_lossy()
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
                    &src.checkout
                ));
            }
            Proto::url => {
                todo!("add support to fetch deps over HTTP");
            }
            Proto::file => (),
        }
    }
    Ok(())
}
