use {
    crate::{
        error::Error,
        manifest::{Inheritable, Manifest, Root},
    },
    cargo_subcommand::{Artifact, ArtifactType, CrateType, Profile, Subcommand},
    ndk_build2::{
        apk::{Apk, ApkConfig},
        cargo::{VersionCode, cargo_ndk},
        dylibs::get_libs_search_paths,
        error::NdkError,
        ndk::{Key, Ndk},
        target::Target,
    },
    std::{
        env::{var, var_os},
        ffi::OsStr,
        fs::{create_dir_all, read_dir, remove_dir_all},
        path::{Path, PathBuf},
        process::Command,
    },
};

pub struct ApkBuilder<'a> {
    cmd: &'a Subcommand,
    ndk: Ndk,
    java_home: PathBuf,
    manifest: Manifest,
    build_dir: PathBuf,
    classes_dir: PathBuf,
    build_targets: Vec<Target>,
    device_serial: Option<String>,
}

impl<'a> ApkBuilder<'a> {
    pub fn from_subcommand(
        cmd: &'a Subcommand,
        device_serial: Option<String>,
    ) -> Result<Self, Error> {
        println!(
            "Using package `{}` in `{}`",
            cmd.package(),
            cmd.manifest().display()
        );
        let ndk = Ndk::from_env()?;
        let java_home = android_build::java_home().ok_or(Error::JdkNotFound)?;
        let mut manifest = Manifest::parse_from_toml(cmd.manifest())?;
        let workspace_manifest: Option<Root> = cmd
            .workspace_manifest()
            .map(Root::parse_from_toml)
            .transpose()?;
        let build_targets = if let Some(target) = cmd.target() {
            vec![Target::from_rust_triple(target)?]
        } else if !manifest.build_targets.is_empty() {
            manifest.build_targets.clone()
        } else {
            vec![
                ndk.detect_abi(device_serial.as_deref())
                    .unwrap_or(Target::Arm64V8a),
            ]
        };
        let build_dir = dunce::simplified(cmd.target_dir())
            .join(cmd.profile())
            .join("apk");
        let classes_dir = build_dir.join("classes");

        let package_version = match &manifest.version {
            Inheritable::Value(v) => v.clone(),
            Inheritable::Inherited { workspace: true } => {
                let workspace = workspace_manifest
                    .clone()
                    .ok_or(Error::InheritanceMissingWorkspace)?
                    .workspace
                    .unwrap_or_else(|| {
                        // Unlikely to fail as cargo-subcommand should give us
                        // a `Cargo.toml` containing a `[workspace]` table
                        panic!(
                            "Manifest `{:?}` must contain a `[workspace]` table",
                            cmd.workspace_manifest().unwrap()
                        )
                    });

                workspace
                    .package
                    .ok_or(Error::WorkspaceMissingInheritedField("package"))?
                    .version
                    .ok_or(Error::WorkspaceMissingInheritedField("package.version"))?
            }
            Inheritable::Inherited { workspace: false } => return Err(Error::InheritedFalse),
        };
        let version_code = VersionCode::from_semver(&package_version)?.to_code(1);

        // 设置默认 Android 清单值
        if manifest
            .android_manifest
            .version_name
            .replace(package_version)
            .is_some()
        {
            panic!("version_name should not be set in TOML");
        }

        if manifest
            .android_manifest
            .version_code
            .replace(version_code)
            .is_some()
        {
            panic!("version_code should not be set in TOML");
        }

        let target_sdk_version = *manifest
            .android_manifest
            .sdk
            .target_sdk_version
            .get_or_insert_with(|| ndk.default_target_platform());

        manifest
            .android_manifest
            .application
            .debuggable
            .get_or_insert_with(|| *cmd.profile() == Profile::Dev);

        // 检查是否有Activity定义
        if manifest.android_manifest.application.activities.is_empty() {
            eprintln!(
                "Warning: No activities defined. Please add [[package.metadata.android.application.activity]] configuration in Cargo.toml."
            );
            eprintln!("Example:");
            eprintln!("[[package.metadata.android.application.activity]]");
            eprintln!("name = \"android.app.NativeActivity\"");
            eprintln!("[[package.metadata.android.application.activity.meta_data]]");
            eprintln!("name = \"android.app.lib_name\"");
            eprintln!("value = \"your_lib_name\"");
            eprintln!("[[package.metadata.android.application.activity.intent_filter]]");
            eprintln!("actions = [\"android.intent.action.VIEW\", \"android.intent.action.MAIN\"]");
            eprintln!("categories = [\"android.intent.category.LAUNCHER\"]");
        }

        // 如果用户未明确执行此操作，则在 Android S 及更高版本上导出 Activity。如果没有此操作，应用将无法在 S+ 上启动。
        // https://developer.android.com/about/versions/12/behavior-changes-12#exported
        if target_sdk_version >= 31 {
            manifest
                .android_manifest
                .application
                .activities
                .iter_mut()
                .for_each(|a| {
                    a.exported.get_or_insert(true);
                });
        }

        // 在`<meta-data />`中如果没有提供"android.app.lib_name"的值，则自动优先使用`[lib]`中提供的名称，否则将使用`[package]`中的名称
        let lib_name = cmd.artifacts().next().map(|i| i.name.replace("-", "_"));
        manifest
            .android_manifest
            .application
            .activities
            .iter_mut()
            .for_each(|i| {
                i.meta_data.iter_mut().for_each(|i| {
                    if i.name == "android.app.lib_name" && i.value.is_none() {
                        i.value = lib_name.clone();
                    }
                })
            });

        Ok(Self {
            cmd,
            ndk,
            java_home,
            manifest,
            build_dir,
            classes_dir,
            build_targets,
            device_serial,
        })
    }

    pub fn check(&self) -> Result<(), Error> {
        for target in &self.build_targets {
            let mut cargo = cargo_ndk(
                &self.ndk,
                *target,
                self.min_sdk_version(),
                self.cmd.target_dir(),
            )?;
            cargo.arg("check");
            if self.cmd.target().is_none() {
                let triple = target.rust_triple();
                cargo.arg("--target").arg(triple);
            }
            self.cmd.args().apply(&mut cargo);
            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }
        }

        Ok(())
    }

    pub fn compile_java_sources(&self, java_sources: Option<PathBuf>) -> Result<(), Error> {
        // 创建临时目录用于编译
        let _ = remove_dir_all(&self.classes_dir); // 清理旧的编译结果
        create_dir_all(&self.classes_dir)?;

        let java_sources = match java_sources {
            Some(p) => p,
            None => return Ok(()),
        };

        if !java_sources.exists() {
            return Err(Error::PathNotFound(java_sources));
        }

        println!("Compiling Java sources...");

        // 获取Android SDK中的android.jar路径
        let target_sdk_version = self
            .manifest
            .android_manifest
            .sdk
            .target_sdk_version
            .unwrap_or_else(|| self.ndk.default_target_platform());
        let android_jar = self.ndk.android_jar(target_sdk_version)?;

        // 使用javac编译Java源文件
        let mut javac = Command::new(self.java_home.join("bin").join("javac"));
        javac
            .arg("-d")
            .arg(&self.classes_dir)
            .arg("-classpath")
            .arg(&android_jar);

        // 添加所有Java源文件
        let mut has_java_files = false;

        // 递归收集所有Java源文件
        fn collect_java_files(
            dir: &Path,
            javac: &mut Command,
            has_files: &mut bool,
        ) -> Result<(), NdkError> {
            for entry in read_dir(dir).map_err(|e| NdkError::IoPathError(dir.to_path_buf(), e))? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    collect_java_files(&path, javac, has_files)?;
                } else if path.extension() == Some(OsStr::new("java")) {
                    javac.arg(&path);
                    *has_files = true;
                }
            }

            Ok(())
        }

        collect_java_files(&java_sources, &mut javac, &mut has_java_files)?;

        if !has_java_files {
            println!("No Java source files found in {:?}", java_sources);
            return Ok(());
        }

        if !javac.status()?.success() {
            return Err(Error::CmdFailed(javac));
        }

        Ok(())
    }

    pub fn build(&self, artifact: &Artifact) -> Result<Apk, Error> {
        // 设置工件特定的清单默认值。
        let mut manifest = self.manifest.android_manifest.clone();
        let apk_package = &mut manifest.package;

        if apk_package.is_empty() {
            let name = artifact.name.replace('-', "_");
            *apk_package = match artifact.r#type {
                ArtifactType::Lib => format!("rust.{}", name),
                ArtifactType::Bin => format!("rust.{}", name),
                ArtifactType::Example => format!("rust.example.{}", name),
            };
        }

        if manifest.application.label.is_empty() {
            manifest.application.label = artifact.name.to_string();
        }

        let crate_path = self.cmd.manifest().parent().expect("invalid manifest path");

        let is_debug_profile = *self.cmd.profile() == Profile::Dev;

        let assets = self
            .manifest
            .assets
            .as_ref()
            .map(|assets| dunce::simplified(&crate_path.join(assets)).to_owned());
        let resources = self
            .manifest
            .resources
            .as_ref()
            .map(|res| dunce::simplified(&crate_path.join(res)).to_owned());
        let java_sources = self
            .manifest
            .java_sources
            .as_ref()
            .map(|src| dunce::simplified(&crate_path.join(src)).to_owned());
        let runtime_libs = self
            .manifest
            .runtime_libs
            .as_ref()
            .map(|libs| dunce::simplified(&crate_path.join(libs)).to_owned());
        let apk_name = self
            .manifest
            .apk_name
            .clone()
            .unwrap_or_else(|| artifact.name.to_string());
        let apk_package = apk_package.to_owned();
        let use_aapt2 = self.manifest.use_aapt2.unwrap_or(true);
        self.compile_java_sources(java_sources)?;

        let config = ApkConfig {
            ndk: self.ndk.clone(),
            build_dir: self.build_dir.join(artifact.build_dir()),
            apk_name: apk_name.clone(),
            use_aapt2,
            assets: assets.clone(),
            resources: resources.clone(),
            java_classes: self.classes_dir.clone(),
            manifest,
            disable_aapt_compression: is_debug_profile,
            strip: self.manifest.strip,
            reverse_port_forward: self.manifest.reverse_port_forward.clone(),
        };
        let mut apk = config.create_apk()?;

        for target in &self.build_targets {
            let triple = target.rust_triple();
            let build_dir = self.cmd.build_dir(Some(triple));
            let artifact = self.cmd.artifact(artifact, Some(triple), CrateType::Cdylib);

            let mut cargo = cargo_ndk(
                &self.ndk,
                *target,
                self.min_sdk_version(),
                self.cmd.target_dir(),
            )?;
            cargo.env("CARGO_APK2_APK_NAME", &apk_name);
            cargo.env("CARGO_APK2_PACKAGE", &apk_package);
            cargo.env("CARGO_APK2_ARTIFACT", &artifact);
            if let Some(p) = assets.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_ASSETS_DIR", p);
            }
            if let Some(p) = resources.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_RESOURCES_DIR", p);
            }
            if let Some(p) = self.classes_dir.to_str() {
                cargo.env("CARGO_APK2_CLASSES_DIR", p);
            }
            if let Some(p) = runtime_libs.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_RUNTIME_LIBS_DIR", p);
            }
            if let Some(p) = self.java_home.to_str() {
                cargo.env("CARGO_APK2_JAVA_HOME", p);
            }
            if let Some(p) = android_build::android_sdk()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_SDK_HOME", p);
            }
            if let Ok(p) = self.ndk.android_jar(self.target_sdk_version())
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_ANDROID_JAR", p);
            }
            if let Ok(p) = self.ndk.platform_dir(self.target_sdk_version())
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_PLATFORM_DIR", p);
            }
            cargo.env(
                "CARGO_APK2_BUILD_TOOLS_VERSION",
                self.ndk.build_tools_version(),
            );
            cargo.env(
                "CARGO_APK2_MIN_SDK_VERSION",
                self.min_sdk_version().to_string(),
            );
            cargo.env(
                "CARGO_APK2_TARGET_SDK_VERSION",
                self.target_sdk_version().to_string(),
            );

            cargo.arg("build");
            if self.cmd.target().is_none() {
                cargo.arg("--target").arg(triple);
            }
            self.cmd.args().apply(&mut cargo);

            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }

            let mut libs_search_paths =
                get_libs_search_paths(self.cmd.target_dir(), triple, self.cmd.profile().as_ref())?;
            libs_search_paths.push(build_dir.join("deps"));

            let libs_search_paths = libs_search_paths
                .iter()
                .map(|path| path.as_path())
                .collect::<Vec<_>>();

            apk.add_lib_recursively(&artifact, *target, libs_search_paths.as_slice())?;

            if let Some(runtime_libs) = &runtime_libs {
                apk.add_runtime_libs(runtime_libs, *target, libs_search_paths.as_slice())?;
            }
        }

        let profile_name = match self.cmd.profile() {
            Profile::Dev => "dev",
            Profile::Release => "release",
            Profile::Custom(c) => c.as_str(),
        };

        let keystore_env = format!(
            "CARGO_APK_{}_KEYSTORE",
            profile_name.to_uppercase().replace('-', "_")
        );
        let password_env = format!("{}_PASSWORD", keystore_env);

        let path = var_os(&keystore_env).map(PathBuf::from);
        let password = var(&password_env).ok();

        let signing_key = match (path, password) {
            (Some(path), Some(password)) => Key { path, password },
            (Some(path), None) if is_debug_profile => {
                eprintln!(
                    "{} not specified, falling back to default password",
                    password_env
                );
                Key {
                    path,
                    password: ndk_build2::ndk::DEFAULT_DEV_KEYSTORE_PASSWORD.to_owned(),
                }
            }
            (Some(path), None) => {
                eprintln!(
                    "`{}` was specified via `{}`, but `{}` was not specified, both or neither must be present for profiles other than `dev`",
                    path.display(),
                    keystore_env,
                    password_env
                );
                return Err(Error::MissingReleaseKey(profile_name.to_owned()));
            }
            (None, _) => {
                if let Some(msk) = self.manifest.signing.get(profile_name) {
                    Key {
                        path: crate_path.join(&msk.path),
                        password: msk.keystore_password.clone(),
                    }
                } else if is_debug_profile {
                    self.ndk.debug_key()?
                } else {
                    return Err(Error::MissingReleaseKey(profile_name.to_owned()));
                }
            }
        };

        // 添加Java类
        apk.add_java_classes()?;
        let unsigned = apk.add_pending_libs_and_align()?;

        println!(
            "Signing `{}` with keystore `{}`",
            config.apk().display(),
            signing_key.path.display()
        );
        Ok(unsigned.sign(signing_key)?)
    }

    pub fn run(&self, artifact: &Artifact, no_logcat: bool) -> Result<(), Error> {
        let apk = self.build(artifact)?;
        apk.reverse_port_forwarding(self.device_serial.as_deref())?;
        apk.install(self.device_serial.as_deref())?;

        // 查找第一个带有android.intent.action.MAIN的Activity
        let activity = self
            .manifest
            .android_manifest
            .application
            .activities
            .iter()
            .find(|activity| {
                activity.intent_filter.iter().any(|filter| {
                    filter
                        .actions
                        .contains(&"android.intent.action.MAIN".to_string())
                })
            })
            .map(|activity| activity.name.as_str());

        apk.start(self.device_serial.as_deref(), activity)?;
        let uid = apk.uidof(self.device_serial.as_deref())?;

        if !no_logcat {
            self.ndk
                .adb(self.device_serial.as_deref())?
                .arg("logcat")
                .arg("-v")
                .arg("color")
                .arg("--uid")
                .arg(uid.to_string())
                .status()?;
        }

        Ok(())
    }

    pub fn gdb(&self, artifact: &Artifact) -> Result<(), Error> {
        let apk = self.build(artifact)?;
        apk.install(self.device_serial.as_deref())?;

        let target_dir = self.build_dir.join(artifact.build_dir());
        self.ndk.ndk_gdb(
            target_dir,
            "android.app.NativeActivity",
            self.device_serial.as_deref(),
        )?;

        Ok(())
    }

    pub fn default(&self, cargo_cmd: &str, cargo_args: &[String]) -> Result<(), Error> {
        for target in &self.build_targets {
            let mut cargo = cargo_ndk(
                &self.ndk,
                *target,
                self.min_sdk_version(),
                self.cmd.target_dir(),
            )?;
            cargo.arg(cargo_cmd);
            self.cmd.args().apply(&mut cargo);

            if self.cmd.target().is_none() {
                let triple = target.rust_triple();
                cargo.arg("--target").arg(triple);
            }

            for additional_arg in cargo_args {
                cargo.arg(additional_arg);
            }

            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }
        }
        Ok(())
    }

    /// Returns `minSdkVersion` for use in compiler target selection:
    /// <https://developer.android.com/ndk/guides/sdk-versions#minsdkversion>
    ///
    /// Has a lower bound of `23` to retain backwards compatibility with
    /// the previous default.
    fn min_sdk_version(&self) -> u32 {
        self.manifest
            .android_manifest
            .sdk
            .min_sdk_version
            .unwrap_or(23)
            .max(23)
    }

    pub fn target_sdk_version(&self) -> u32 {
        self.manifest
            .android_manifest
            .sdk
            .target_sdk_version
            .unwrap_or(self.ndk.default_target_platform())
    }
}
