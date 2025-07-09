use {
    cargo_subcommand::Error as SubcommandError,
    ndk_build2::error::NdkError,
    std::{io::Error as IoError, path::PathBuf, process::Command},
    thiserror::Error,
    toml::de::Error as TomlError,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Subcommand(#[from] SubcommandError),
    #[error("Command `{}` had a non-zero exit code.", format!("{:?}", .0).replace('"', ""))]
    CmdFailed(Command),
    #[error("Failed to parse config.")]
    Config(#[from] TomlError),
    #[error(transparent)]
    Ndk(#[from] NdkError),
    #[error(
        "Java Development Kit is not found. \
    Please set the path to the JDK with the $JAVA_HOME \
    environment variable."
    )]
    JdkNotFound,
    #[error("Path `{0:?}` doesn't exist.")]
    PathNotFound(PathBuf),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("Configure a release keystore via `[package.metadata.android.signing.{0}]`")]
    MissingReleaseKey(String),
    #[error("`workspace=false` is unsupported")]
    InheritedFalse,
    #[error("`workspace=true` requires a workspace")]
    InheritanceMissingWorkspace,
    #[error("Failed to inherit field: `workspace.{0}` was not defined in workspace root manifest")]
    WorkspaceMissingInheritedField(&'static str),
}

impl Error {
    pub fn invalid_args() -> Self {
        Self::Subcommand(SubcommandError::InvalidArgs)
    }
}
