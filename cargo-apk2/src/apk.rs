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
        env::var,
        ffi::OsStr,
        fs::{create_dir_all, read_dir},
        path::{Path, PathBuf},
        process::Command,
    },
};

pub struct ApkBuilder<'a> {
    cmd: &'a Subcommand,
    ndk: Ndk,
    java_home: Option<PathBuf>,
    kotlin_home: Option<PathBuf>,
    scala_home: Option<PathBuf>,
    groovy_home: Option<PathBuf>,
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
        let java_home = android_build::java_home();
        let groovy_home = var("GROOVY_HOME").ok().map(PathBuf::from);
        let kotlin_home = var("KOTLIN_HOME").ok().map(PathBuf::from);
        let scala_home = var("SCALA_HOME").ok().map(PathBuf::from);
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
            groovy_home,
            kotlin_home,
            scala_home,
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

    /// 递归收集所有源文件
    fn collect_source_files<P>(
        dir: P,
        cmd: &mut Command,
        has_files: &mut bool,
        extension: &str,
    ) -> Result<(), NdkError>
    where
        P: AsRef<Path>,
    {
        for entry in
            read_dir(&dir).map_err(|e| NdkError::IoPathError(dir.as_ref().to_path_buf(), e))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_source_files(&path, cmd, has_files, extension)?;
            } else if path.extension() == Some(OsStr::new(extension)) {
                cmd.arg(&path);
                *has_files = true;
            }
        }

        Ok(())
    }

    pub fn compile_java_sources<P>(&self, java_sources: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        if !java_sources.as_ref().exists() {
            return Ok(());
        }

        // 获取Android SDK中的android.jar路径
        let android_jar = self.ndk.android_jar(self.target_sdk_version())?;

        // 使用javac编译Java源文件
        let mut javac = Command::new(
            self.java_home
                .as_ref()
                .ok_or(Error::JdkNotFound)?
                .join("bin")
                .join("javac"),
        );
        javac
            .arg("-d")
            .arg(&self.classes_dir)
            .arg("-classpath")
            .arg(&android_jar);

        // 添加所有Java源文件
        let mut has_java_files = false;
        Self::collect_source_files(&java_sources, &mut javac, &mut has_java_files, "java")?;
        if !has_java_files {
            println!("No Java source files found in {:?}", java_sources.as_ref());
            return Ok(());
        }

        if !javac.status()?.success() {
            return Err(Error::CmdFailed(javac));
        }

        Ok(())
    }

    //noinspection SpellCheckingInspection
    pub fn compile_kotlin_sources<P>(&self, kotlin_sources: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        if !kotlin_sources.as_ref().exists() {
            return Ok(());
        }

        // 获取Android SDK中的android.jar路径
        let android_jar = self.ndk.android_jar(self.target_sdk_version())?;

        // 使用kotlinc编译Kotlin源文件
        let mut kotlinc = Command::new(
            self.kotlin_home
                .as_ref()
                .ok_or(Error::KotlinNotFound)?
                .join("bin")
                .join(if cfg!(windows) {
                    "kotlinc.bat"
                } else {
                    "kotlinc"
                }),
        );
        kotlinc
            .arg("-d")
            .arg(&self.classes_dir)
            .arg("-classpath")
            .arg(&android_jar);

        // 添加所有Kotlin源文件
        let mut has_kotlin_files = false;
        Self::collect_source_files(&kotlin_sources, &mut kotlinc, &mut has_kotlin_files, "kt")?;
        if !has_kotlin_files {
            println!(
                "No Kotlin source files found in {:?}",
                kotlin_sources.as_ref()
            );
            return Ok(());
        }

        if !kotlinc.status()?.success() {
            return Err(Error::CmdFailed(kotlinc));
        }

        Ok(())
    }

    pub fn compile_scala_sources<P>(&self, scala_sources: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        if !scala_sources.as_ref().exists() {
            return Ok(());
        }

        // 获取Android SDK中的android.jar路径
        let android_jar = self.ndk.android_jar(self.target_sdk_version())?;

        // 使用scalac编译Scala源文件
        let mut scalac = Command::new(
            self.scala_home
                .as_ref()
                .ok_or(Error::ScalaNotFound)?
                .join("bin")
                .join(if cfg!(windows) {
                    "scalac.bat"
                } else {
                    "scalac"
                }),
        );
        const PATH_SEPARATOR: char = if cfg!(windows) { ';' } else { ':' };
        scalac
            .arg("-d")
            .arg(&self.classes_dir)
            .arg("-classpath")
            .arg(format!("{}{}", PATH_SEPARATOR, &android_jar.display()));

        // 添加所有Scala源文件
        let mut has_scala_files = false;
        Self::collect_source_files(&scala_sources, &mut scalac, &mut has_scala_files, "scala")?;
        if !has_scala_files {
            println!(
                "No Scala source files found in {:?}",
                scala_sources.as_ref()
            );
            return Ok(());
        }

        if !scalac.status()?.success() {
            return Err(Error::CmdFailed(scalac));
        }

        Ok(())
    }

    //noinspection SpellCheckingInspection
    pub fn compile_groovy_sources<P>(&self, groovy_sources: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        if !groovy_sources.as_ref().exists() {
            return Ok(());
        }

        // 获取Android SDK中的android.jar路径
        let android_jar = self.ndk.android_jar(self.target_sdk_version())?;

        // 使用groovyc编译Groovy源文件
        let mut groovyc = Command::new(
            self.groovy_home
                .as_ref()
                .ok_or(Error::GroovyNotFound)?
                .join("bin")
                .join(if cfg!(windows) {
                    "groovyc.bat"
                } else {
                    "groovyc"
                }),
        );
        groovyc
            .arg("-d")
            .arg(&self.classes_dir)
            .arg("-classpath")
            .arg(&android_jar)
            .arg("--compile-static");

        // 添加所有Groovy源文件
        let mut has_groovy_files = false;
        Self::collect_source_files(
            &groovy_sources,
            &mut groovyc,
            &mut has_groovy_files,
            "groovy",
        )?;
        if !has_groovy_files {
            println!(
                "No Groovy source files found in {:?}",
                groovy_sources.as_ref()
            );
            return Ok(());
        }

        if !groovyc.status()?.success() {
            return Err(Error::CmdFailed(groovyc));
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
        let kotlin_sources = self
            .manifest
            .kotlin_sources
            .as_ref()
            .map(|src| dunce::simplified(&crate_path.join(src)).to_owned());
        let scala_sources = self
            .manifest
            .scala_sources
            .as_ref()
            .map(|src| dunce::simplified(&crate_path.join(src)).to_owned());
        let groovy_sources = self
            .manifest
            .groovy_sources
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
        let build_dir = self.build_dir.join(artifact.build_dir());
        let gen_java_dir = build_dir.join("java");

        let config = ApkConfig {
            ndk: self.ndk.clone(),
            build_dir,
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
        let mut apk = config.create_apk(&gen_java_dir)?;

        // 创建临时目录用于编译Java/Kotlin/Scala/Groovy
        create_dir_all(&self.classes_dir)?;

        // 编译Java源文件
        if gen_java_dir.exists() || java_sources.is_some() {
            println!("Compiling Java sources...",);
        }
        if gen_java_dir.exists() {
            self.compile_java_sources(gen_java_dir)?;
        }
        if let Some(java_sources) = java_sources {
            self.compile_java_sources(java_sources)?;
        }

        // 编译Kotlin源文件
        if let Some(kotlin_sources) = kotlin_sources {
            println!("Compiling Kotlin sources...",);
            self.compile_kotlin_sources(kotlin_sources)?;
        }

        // 编译Scala源文件
        if let Some(scala_sources) = scala_sources {
            println!("Compiling Scala sources...",);
            self.compile_scala_sources(scala_sources)?;
        }

        // 编译Groovy源文件
        if let Some(groovy_sources) = groovy_sources {
            println!("Compiling Groovy sources...",);
            self.compile_groovy_sources(groovy_sources)?;
        }

        // 编译动态库
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
            if let Some(p) = self.java_home.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_JAVA_HOME", p);
            }
            if let Some(p) = self.kotlin_home.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_KOTLIN_HOME", p);
            }
            if let Some(p) = self.scala_home.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_SCALA_HOME", p);
            }
            if let Some(p) = self.groovy_home.as_ref()
                && let Some(p) = p.to_str()
            {
                cargo.env("CARGO_APK2_GROOVY_HOME", p);
            }
            if let Some(p) = self.ndk.android_sdk().to_str() {
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

        let path = var(&keystore_env).map(PathBuf::from);
        let password = var(&password_env).ok();

        let signing_key = match (path, password) {
            (Ok(path), Some(password)) => Key { path, password },
            (Ok(path), None) if is_debug_profile => {
                eprintln!(
                    "{} not specified, falling back to default password",
                    password_env
                );
                Key {
                    path,
                    password: ndk_build2::ndk::DEFAULT_DEV_KEYSTORE_PASSWORD.to_owned(),
                }
            }
            (Ok(path), None) => {
                eprintln!(
                    "`{}` was specified via `{}`, but `{}` was not specified, both or neither must be present for profiles other than `dev`",
                    path.display(),
                    keystore_env,
                    password_env
                );
                return Err(Error::MissingReleaseKey(profile_name.to_owned()));
            }
            (Err(_), _) => {
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

    pub fn run(
        &self,
        artifact: &Artifact,
        no_logcat: bool,
        show_logcat_time: bool,
    ) -> Result<(), Error> {
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
                .arg("-c")
                .status()?;

            let mut adb = self.ndk.adb(self.device_serial.as_deref())?;
            adb.arg("logcat").arg("-v");

            // 根据show_logcat_time参数决定是否显示时间
            if show_logcat_time {
                adb.arg("time");
            } else {
                adb.arg("brief");
            }
            // 添加颜色支持
            adb.arg("color");

            // 过滤指定应用的日志
            adb.arg("--uid").arg(uid.to_string());

            adb.status()?;
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
