use crate::io_err_context;
pub use std::path::Path;

pub trait SubPath {
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
