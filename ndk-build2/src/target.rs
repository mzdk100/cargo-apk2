use {
    crate::error::NdkError,
    serde::{Deserialize, Serialize},
};

//noinspection SpellCheckingInspection
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[repr(u8)]
pub enum Target {
    #[serde(rename = "armv7-linux-androideabi")]
    ArmV7a = 1,
    #[serde(rename = "aarch64-linux-android")]
    Arm64V8a = 2,
    #[serde(rename = "i686-linux-android")]
    X86 = 3,
    #[serde(rename = "x86_64-linux-android")]
    X86_64 = 4,
}

//noinspection SpellCheckingInspection
impl Target {
    /// NDK 中用于引用 ABI 的标识符
    pub fn android_abi(self) -> &'static str {
        match self {
            Self::Arm64V8a => "arm64-v8a",
            Self::ArmV7a => "armeabi-v7a",
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
        }
    }

    /// 返回 abi 的“target”。
    pub fn from_android_abi(abi: &str) -> Result<Self, NdkError> {
        match abi {
            "arm64-v8a" => Ok(Self::Arm64V8a),
            "armeabi-v7a" => Ok(Self::ArmV7a),
            "x86" => Ok(Self::X86),
            "x86_64" => Ok(Self::X86_64),
            _ => Err(NdkError::UnsupportedTarget),
        }
    }

    /// 返回 rust 构建工具使用的三元组
    pub fn rust_triple(self) -> &'static str {
        match self {
            Self::Arm64V8a => "aarch64-linux-android",
            Self::ArmV7a => "armv7-linux-androideabi",
            Self::X86 => "i686-linux-android",
            Self::X86_64 => "x86_64-linux-android",
        }
    }

    /// 返回 Rust 三元组的“target”。
    pub fn from_rust_triple(triple: &str) -> Result<Self, NdkError> {
        match triple {
            "aarch64-linux-android" => Ok(Self::Arm64V8a),
            "armv7-linux-androideabi" => Ok(Self::ArmV7a),
            "i686-linux-android" => Ok(Self::X86),
            "x86_64-linux-android" => Ok(Self::X86_64),
            _ => Err(NdkError::UnsupportedTarget),
        }
    }

    /// 返回三重 NDK 提供的 LLVM
    pub fn ndk_llvm_triple(self) -> &'static str {
        match self {
            Self::Arm64V8a => "aarch64-linux-android",
            Self::ArmV7a => "armv7a-linux-androideabi",
            Self::X86 => "i686-linux-android",
            Self::X86_64 => "x86_64-linux-android",
        }
    }

    /// 返回 NDK 非 LLVM 部分使用的三元组
    pub fn ndk_triple(self) -> &'static str {
        match self {
            Self::Arm64V8a => "aarch64-linux-android",
            Self::ArmV7a => "arm-linux-androideabi",
            Self::X86 => "i686-linux-android",
            Self::X86_64 => "x86_64-linux-android",
        }
    }
}
