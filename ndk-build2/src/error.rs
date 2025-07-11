use {
    std::{
        io::Error as IoError, num::ParseIntError, path::PathBuf, process::Command, str::Utf8Error,
    },
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum NdkError {
    #[error(
        "Android SDK is not found. \
    Please set the path to the Android SDK with the $ANDROID_HOME \
    environment variable."
    )]
    SdkNotFound,
    #[error(
        "Android NDK is not found. \
        Please set the path to the Android NDK with $ANDROID_NDK_ROOT \
        environment variable."
    )]
    NdkNotFound,
    #[error(
        "GNU toolchain binary `{gnu_bin}` nor LLVM toolchain binary `{llvm_bin}` found in `{toolchain_path:?}`."
    )]
    ToolchainBinaryNotFound {
        toolchain_path: PathBuf,
        gnu_bin: String,
        llvm_bin: String,
    },
    #[error("Path `{0:?}` doesn't exist.")]
    PathNotFound(PathBuf),
    #[error("Command `{0}` not found.")]
    CmdNotFound(String),
    #[error("Android SDK has no build tools.")]
    BuildToolsNotFound,
    #[error("Android SDK has no platforms installed.")]
    NoPlatformFound,
    #[error("Platform `{0}` is not installed.")]
    PlatformNotFound(u32),
    #[error("Target is not supported.")]
    UnsupportedTarget,
    #[error("Host `{0}` is not supported.")]
    UnsupportedHost(String),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("IoError on `{0:?}`: {1}")]
    IoPathError(PathBuf, #[source] IoError),
    #[error("Invalid semver")]
    InvalidSemver,
    #[error("Command `{}` had a non-zero exit code.", format!("{:?}", .0).replace('"', ""))]
    CmdFailed(Command),
    #[error(transparent)]
    Serialize(#[from] quick_xml::SeError),
    #[error("String `{1}` is not a UID")]
    NotAUid(#[source] ParseIntError, String),
    #[error("Could not find `package:{package}` in output `{output}`")]
    PackageNotInOutput { package: String, output: String },
    #[error("Could not find `uid:` in output `{0}`")]
    UidNotInOutput(String),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
}
