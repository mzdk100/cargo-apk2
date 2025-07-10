use {
    crate::{
        error::NdkError,
        manifest::AndroidManifest,
        ndk::{Key, Ndk},
        target::Target,
    },
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        ffi::OsStr,
        fs::{copy, create_dir_all, read_dir, remove_file, rename},
        io::{Error as IoError, ErrorKind},
        path::{Path, PathBuf},
        process::Command,
        str::from_utf8,
    },
};

//noinspection SpellCheckingInspection
/// 如何处理添加到 APK 的任何 `.so` 文件中的调试符号的选项。
///
/// 在您的货物清单中使用
/// [`strip`](https://doc.rust-lang.org/cargo/reference/profiles.html#strip)
/// 或 [`split-debuginfo`](https://doc.rust-lang.org/cargo/reference/profiles.html#split-debuginfo)
/// 可能会导致调试符号不存在于 `.so` 中，从而导致这些选项不执行任何操作。
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StripConfig {
    /// 不对调试符号进行特殊处理
    #[default]
    Default,
    /// 在将库复制到 APK 之前，从库中删除调试符号
    Strip,
    /// 将库拆分为 ELF（`.so`）和 DWARF（`.dwarf`）。只有 `.so` 会被复制到 APK 中
    Split,
}

pub struct ApkConfig {
    pub ndk: Ndk,
    pub build_dir: PathBuf,
    pub apk_name: String,
    pub use_aapt2: bool,
    pub assets: Option<PathBuf>,
    pub resources: Option<PathBuf>,
    pub java_classes: PathBuf,
    pub manifest: AndroidManifest,
    pub disable_aapt_compression: bool,
    pub strip: StripConfig,
    pub reverse_port_forward: HashMap<String, String>,
}

impl ApkConfig {
    fn build_tool(&self, tool: &'static str) -> Result<Command, NdkError> {
        let mut cmd = self.ndk.build_tool(tool)?;
        cmd.current_dir(&self.build_dir);
        Ok(cmd)
    }

    fn unaligned_apk(&self) -> PathBuf {
        self.build_dir
            .join(format!("{}-unaligned.apk", self.apk_name))
    }

    /// 调用 [`UnsignedApk::sign`] 时将写入的 APK 的路径
    #[inline]
    pub fn apk(&self) -> PathBuf {
        self.build_dir.join(format!("{}.apk", self.apk_name))
    }

    pub fn create_apk(&self) -> Result<UnalignedApk, NdkError> {
        create_dir_all(&self.build_dir)?;
        self.manifest.write_to(&self.build_dir)?;

        let target_sdk_version = self
            .manifest
            .sdk
            .target_sdk_version
            .unwrap_or_else(|| self.ndk.default_target_platform());

        if self.use_aapt2 {
            let out_dir = self
                .build_dir
                .join(format!("compiled_{}_resource", self.manifest.package).as_str());
            self.aapt2_compile(&out_dir)?;
            self.aapt2_link(target_sdk_version)?;
            if !self.manifest.application.debuggable.unwrap_or(false) {
                self.aapt2_optimize()?;
            }
            remove_file(out_dir)?;
        } else {
            self.aapt_package(target_sdk_version)?;
        }

        Ok(UnalignedApk {
            config: self,
            pending_libs: HashSet::default(),
        })
    }

    fn aapt_package(&self, target_sdk_version: u32) -> Result<(), NdkError> {
        let mut aapt = self.build_tool(bin!("aapt"))?;
        println!("Packing apk resources......");
        aapt.arg("package")
            .arg("-f")
            .arg("-F")
            .arg(self.unaligned_apk())
            .arg("-M")
            .arg("AndroidManifest.xml")
            .arg("-I")
            .arg(self.ndk.android_jar(target_sdk_version)?);

        if self.disable_aapt_compression {
            aapt.arg("-0").arg("");
        }

        if let Some(res) = &self.resources {
            aapt.arg("-S").arg(res);
        }

        if let Some(assets) = &self.assets {
            aapt.arg("-A").arg(assets);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }
        Ok(())
    }

    fn aapt2_compile(&self, out_dir: impl AsRef<OsStr>) -> Result<(), NdkError> {
        let mut aapt = self.build_tool(bin!("aapt2"))?;
        println!("Compiling apk resources...");
        aapt.arg("compile").arg("-o").arg(out_dir);

        if let Some(res) = &self.resources {
            aapt.arg("--dir").arg(res);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }
        Ok(())
    }

    fn aapt2_link(&self, target_sdk_version: u32) -> Result<(), NdkError> {
        let mut aapt = self.build_tool(bin!("aapt2"))?;
        println!("Linking apk resources...");
        aapt.arg("link")
            .arg("-o")
            .arg(self.unaligned_apk())
            .arg("--manifest")
            .arg("AndroidManifest.xml")
            .arg("-I")
            .arg(self.ndk.android_jar(target_sdk_version)?);

        if self.disable_aapt_compression {
            aapt.arg("--no-compress").arg("-0").arg("");
        }

        if let Some(assets) = &self.assets {
            aapt.arg("-A").arg(assets);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }
        Ok(())
    }

    fn aapt2_optimize(&self) -> Result<(), NdkError> {
        let mut aapt = self.build_tool(bin!("aapt2"))?;
        println!("Optimizing apk resources...");
        let path = self.unaligned_apk();
        let input_path = path.parent().unwrap().join(&self.manifest.package);
        rename(&path, &input_path)?;
        aapt.arg("optimize")
            .arg("-o")
            .arg(path)
            .arg(&input_path)
            .arg("--enable-sparse-encoding");

        if !aapt.status()?.success() {
            remove_file(input_path)?;
            return Err(NdkError::CmdFailed(aapt));
        }
        remove_file(input_path)?;
        Ok(())
    }
}

pub struct UnalignedApk<'a> {
    config: &'a ApkConfig,
    pending_libs: HashSet<String>,
}

impl<'a> UnalignedApk<'a> {
    pub fn config(&self) -> &ApkConfig {
        self.config
    }

    /// 将classes文件添加到APK中
    pub fn add_java_classes(&mut self) -> Result<(), NdkError> {
        // 递归收集所有class文件
        fn collect_class_files(dir: &Path, cmd: &mut Command) -> Result<(), NdkError> {
            for entry in read_dir(dir).map_err(|e| NdkError::IoPathError(dir.to_path_buf(), e))? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    collect_class_files(&path, cmd)?;
                } else if path.extension() == Some(OsStr::new("class")) {
                    cmd.arg(&path);
                }
            }

            Ok(())
        }

        if !self.config.java_classes.exists() {
            return Ok(());
        }

        // 尝试使用d8工具（Android SDK新版本）
        let target_sdk_version = self
            .config
            .manifest
            .sdk
            .target_sdk_version
            .unwrap_or_else(|| self.config.ndk.default_target_platform());
        let mut d8 = self
            .config()
            .build_tool(if cfg!(windows) { "d8.bat" } else { "d8" })?;
        d8.arg("--output")
            .arg(&self.config.build_dir)
            .arg("--lib")
            .arg(&self.config.ndk.android_jar(target_sdk_version)?);
        collect_class_files(&self.config.java_classes, &mut d8)?;

        let dex_file = self.config.build_dir.join("classes.dex");
        let d8_result = d8.status();
        let success = match d8_result {
            Ok(status) => status.success(),
            Err(_) => {
                // 如果d8不可用，尝试使用dx工具（旧版本）
                println!("d8 not found, trying dx...");
                let mut dx = Command::new("dx");
                dx.arg("--dex")
                    .arg("--output")
                    .arg(&dex_file)
                    .arg(&self.config.java_classes);

                dx.status()?.success()
            }
        };

        if !success {
            return Err(
                IoError::new(ErrorKind::Other, "Failed to convert class files to dex").into(),
            );
        }

        // 将classes.dex文件添加到APK中
        if dex_file.exists() {
            let mut aapt = self.config.build_tool(bin!("aapt"))?;
            aapt.arg("add")
                .arg(self.config.unaligned_apk())
                .arg("classes.dex");

            if !aapt.status()?.success() {
                return Err(NdkError::CmdFailed(aapt));
            }
        }

        Ok(())
    }

    pub fn add_lib(&mut self, path: &Path, target: Target) -> Result<(), NdkError> {
        if !path.exists() {
            return Err(NdkError::PathNotFound(path.into()));
        }
        let abi = target.android_abi();
        let lib_path = Path::new("lib").join(abi).join(path.file_name().unwrap());
        let out = self.config.build_dir.join(&lib_path);
        create_dir_all(out.parent().unwrap())?;

        match self.config.strip {
            StripConfig::Default => {
                copy(path, out)?;
            }
            StripConfig::Strip | StripConfig::Split => {
                let obj_copy = self.config.ndk.toolchain_bin("objcopy", target)?;

                {
                    let mut cmd = Command::new(&obj_copy);
                    cmd.arg("--strip-debug");
                    cmd.arg(path);
                    cmd.arg(&out);

                    if !cmd.status()?.success() {
                        return Err(NdkError::CmdFailed(cmd));
                    }
                }

                if self.config.strip == StripConfig::Split {
                    let dwarf_path = out.with_extension("dwarf");

                    {
                        let mut cmd = Command::new(&obj_copy);
                        cmd.arg("--only-keep-debug");
                        cmd.arg(path);
                        cmd.arg(&dwarf_path);

                        if !cmd.status()?.success() {
                            return Err(NdkError::CmdFailed(cmd));
                        }
                    }

                    let mut cmd = Command::new(obj_copy);
                    cmd.arg(format!("--add-gnu-debuglink={}", dwarf_path.display()));
                    cmd.arg(out);

                    if !cmd.status()?.success() {
                        return Err(NdkError::CmdFailed(cmd));
                    }
                }
            }
        }

        // Pass UNIX path separators to `aapt` on non-UNIX systems, ensuring the resulting separator
        // is compatible with the target device instead of the host platform.
        // Otherwise, it results in a runtime error when loading the \NativeActivity `.so` library.
        let lib_path_unix = lib_path.to_str().unwrap().replace('\\', "/");

        self.pending_libs.insert(lib_path_unix);

        Ok(())
    }

    pub fn add_runtime_libs(
        &mut self,
        path: &Path,
        target: Target,
        search_paths: &[&Path],
    ) -> Result<(), NdkError> {
        let abi_dir = path.join(target.android_abi());
        for entry in read_dir(&abi_dir).map_err(|e| NdkError::IoPathError(abi_dir, e))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension() == Some(OsStr::new("so")) {
                self.add_lib_recursively(&path, target, search_paths)?;
            }
        }
        Ok(())
    }

    pub fn add_pending_libs_and_align(self) -> Result<UnsignedApk<'a>, NdkError> {
        let mut aapt = self.config.build_tool(bin!("aapt"))?;
        aapt.arg("add");

        if self.config.disable_aapt_compression {
            aapt.arg("-0").arg("");
        }

        aapt.arg(self.config.unaligned_apk());

        for lib_path_unix in self.pending_libs {
            aapt.arg(lib_path_unix);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }

        let mut zipalign = self.config.build_tool(bin!("zipalign"))?;
        zipalign
            .arg("-f")
            .arg("-v")
            .arg("4")
            .arg(self.config.unaligned_apk())
            .arg(self.config.apk());

        if !zipalign.status()?.success() {
            return Err(NdkError::CmdFailed(zipalign));
        }

        Ok(UnsignedApk(self.config))
    }
}

pub struct UnsignedApk<'a>(&'a ApkConfig);

impl<'a> UnsignedApk<'a> {
    pub fn sign(self, key: Key) -> Result<Apk, NdkError> {
        let mut apksigner = self.0.build_tool(bat!("apksigner"))?;
        apksigner
            .arg("sign")
            .arg("--ks")
            .arg(&key.path)
            .arg("--ks-pass")
            .arg(format!("pass:{}", &key.password))
            .arg(self.0.apk());
        if !apksigner.status()?.success() {
            return Err(NdkError::CmdFailed(apksigner));
        }
        Ok(Apk::from_config(self.0))
    }
}

pub struct Apk {
    path: PathBuf,
    package_name: String,
    ndk: Ndk,
    reverse_port_forward: HashMap<String, String>,
}

impl Apk {
    pub fn from_config(config: &ApkConfig) -> Self {
        let ndk = config.ndk.clone();
        Self {
            path: config.apk(),
            package_name: config.manifest.package.clone(),
            ndk,
            reverse_port_forward: config.reverse_port_forward.clone(),
        }
    }

    pub fn reverse_port_forwarding(&self, device_serial: Option<&str>) -> Result<(), NdkError> {
        for (from, to) in &self.reverse_port_forward {
            println!("Reverse port forwarding from {} to {}", from, to);
            let mut adb = self.ndk.adb(device_serial)?;

            adb.arg("reverse").arg(from).arg(to);

            if !adb.status()?.success() {
                return Err(NdkError::CmdFailed(adb));
            }
        }

        Ok(())
    }

    pub fn install(&self, device_serial: Option<&str>) -> Result<(), NdkError> {
        let mut adb = self.ndk.adb(device_serial)?;

        adb.arg("install").arg("-r").arg(&self.path);
        if !adb.status()?.success() {
            return Err(NdkError::CmdFailed(adb));
        }
        Ok(())
    }

    pub fn start(&self, device_serial: Option<&str>, activity: Option<&str>) -> Result<(), NdkError> {
        let mut adb = self.ndk.adb(device_serial)?;
        adb.arg("shell")
            .arg("am")
            .arg("start")
            .arg("-a")
            .arg("android.intent.action.MAIN")
            .arg("-n");
        
        // 使用提供的activity参数，如果没有提供，则使用默认的NativeActivity
        let activity_name = activity.unwrap_or("android.app.NativeActivity");
        
        adb.arg(format!("{}/{}", self.package_name, activity_name));

        if !adb.status()?.success() {
            return Err(NdkError::CmdFailed(adb));
        }

        Ok(())
    }

    pub fn uidof(&self, device_serial: Option<&str>) -> Result<u32, NdkError> {
        let mut adb = self.ndk.adb(device_serial)?;
        adb.arg("shell")
            .arg("pm")
            .arg("list")
            .arg("package")
            .arg("-U")
            .arg(&self.package_name);
        let output = adb.output()?;

        if !output.status.success() {
            return Err(NdkError::CmdFailed(adb));
        }

        let output = from_utf8(&output.stdout)?;
        let (_package, uid) = output
            .lines()
            .filter_map(|line| line.split_once(' '))
            // `pm list package` uses the id as a substring filter; make sure
            // we select the right package in case it returns multiple matches:
            .find(|(package, _uid)| package.strip_prefix("package:") == Some(&self.package_name))
            .ok_or(NdkError::PackageNotInOutput {
                package: self.package_name.clone(),
                output: output.to_owned(),
            })?;
        let uid = uid
            .strip_prefix("uid:")
            .ok_or(NdkError::UidNotInOutput(output.to_owned()))?;
        uid.parse()
            .map_err(|e| NdkError::NotAUid(e, uid.to_owned()))
    }
}
