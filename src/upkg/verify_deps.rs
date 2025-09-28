use crate::lua::lua_types::*;
use crate::*;

use crypto_common::Output;
use mlua::prelude::*;
use sha2::{Digest, Sha256, Sha512};

use std::io::Read;

fn calc_checksum<P, ShaType>(path: P) -> std::io::Result<Output<ShaType>>
where
    P: AsRef<std::path::Path>,
    ShaType: Digest,
{
    let mut file = fs::File::open(path)?;
    let mut hasher = ShaType::new();
    let mut buffer = [0u8; 8192];

    while let n = file.read(&mut buffer)?
        && n != 0
    {
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}

fn match_sha256<P: AsRef<std::path::Path>>(file: P, digest: &str) -> LuaResult<()> {
    let sha256 = std::format!("{:x}", calc_checksum::<&P, Sha256>(&file)?);
    println!("verifying sha256 for: {:?}", file.as_ref());
    if digest != sha256 {
        return Err(LuaError::external(format!(
            "[{}:{}] sha256 mismatch: file: {}, expected: {}, got: {}",
            file!(),
            line!(),
            file.as_ref().to_string_lossy(),
            digest,
            sha256
        )));
    }
    Ok(())
}

fn match_sha512<P: AsRef<std::path::Path>>(file: P, digest: &str) -> LuaResult<()> {
    let sha512 = std::format!("{:x}", calc_checksum::<&P, Sha512>(&file)?);
    println!("verifying sha512 for: {:?}", file.as_ref());
    if digest != sha512 {
        return Err(LuaError::external(format!(
            "[{}:{}] sha256 mismatch: file: {}, expected: {}, got: {}",
            file!(),
            line!(),
            file.as_ref().to_string_lossy(),
            digest,
            sha512
        )));
    }
    Ok(())
}

pub fn verify<P>(pkg: &Package, pkgbuild: P) -> LuaResult<()>
where
    P: AsRef<std::path::Path>,
{
    for (idx, chksum_field) in pkg.checksum.0.iter().enumerate() {
        match chksum_field {
            CheckSumField::Skip => (),
            CheckSumField::Value { kind, digest } => {
                let source = &pkg.source.0[idx];
                match source.proto {
                    Proto::git => todo!("implement checksum validation for git"),
                    Proto::url => todo!("implement checksum validation for url's"),
                    Proto::file => {
                        let file_loc = pkgbuild
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
                            .join(&source.location);

                        match kind {
                            CheckSumKind::sha256 => match_sha256(&file_loc, digest)?,
                            CheckSumKind::sha512 => match_sha512(&file_loc, digest)?,
                        };
                    }
                }
            }
        }
    }
    Ok(())
}
