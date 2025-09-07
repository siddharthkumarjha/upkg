#[macro_export]
macro_rules! lua_err_ctx {
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

#[macro_export]
macro_rules! io_err_ctx {
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

#[macro_export]
macro_rules! git_err_ctx {
    () => {
        |err| -> git2::Error {
            let err_msg = format!("[{}:{}] {}", file!(), line!(), err);
            git2::Error::from_str(&err_msg)
        }
    };
}

#[macro_export]
macro_rules! git_ok {
    ($arg:expr) => {
        $arg.map_err(git_err_ctx!())?
    };
}
