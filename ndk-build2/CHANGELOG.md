# 未发布

# 0.10.0 (2023-11-30)

- 在清单的 `Application` 元素中添加 `android:extractNativeLibs`、`android:usesCleartextTraffic` 属性，并在 `Activity` 元素中添加 `android:alwaysRetainTaskState`。([#15](https://github.com/rust-mobile/cargo-apk/pull/15))
- 启用从 `android` 主机构建。([#29](https://github.com/rust-mobile/cargo-apk/pull/29))
- 使用应用的 `uid` 而不是 `pid` 来限制 `logcat` 输出仅限于当前应用。([#33](https://github.com/rust-mobile/cargo-apk/pull/33))

# 0.9.0 (2022-11-23)

- 添加 `ndk::DEFAULT_DEV_KEYSTORE_PASSWORD` 并使 `apk::ApkConfig::apk` 公开。([#358](https://github.com/rust-windowing/android-ndk-rs/pull/358))
- `RUSTFLAGS` 现在在 `CARGO_ENCODED_RUSTFLAGS` 不存在时被考虑，允许 `cargo apk build` 在他们依赖 `RUSTFLAGS` 在构建之前被设置时不会破坏用户的构建，
  因为 `ndk-build` 在调用 `cargo` 之前设置的 `CARGO_ENCODED_RUSTFLAGS` 将优先于 [所有其他构建标志的来源](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)。([#357](https://github.com/rust-windowing/android-ndk-rs/pull/357))
- 添加 `ApkConfig::strip`，允许用户指定他们希望在 `cargo` 完成构建后但在共享对象复制到APK之前如何处理调试符号。([#356](https://github.com/rust-windowing/android-ndk-rs/pull/356))

(0.8.1，于2022-10-14发布，由于违反了semver而被撤销。)

- **破坏性变更：** 提供 `reverse_port_forwarding()` 来设置 `adb reverse` ([#348](https://github.com/rust-windowing/android-ndk-rs/pull/348))

# 0.8.0 (2022-09-12)

- **破坏性变更：** 在对齐之前推迟APK库打包，以消除可能重叠的条目。([#333](https://github.com/rust-windowing/android-ndk-rs/pull/333))
- 为 `detect_abi()` 和 `Apk::{install,start}()` 添加 `adb` 设备序列号参数。([#329](https://github.com/rust-windowing/android-ndk-rs/pull/329))
- 修复 `detect_abi()` 中 `adb` 在Windows上的缺少 `.exe` 扩展名。([#339](https://github.com/rust-windowing/android-ndk-rs/pull/339))
- `start()` 现在返回已启动的应用进程的PID（对于传递给 `adb logcat --pid` 有用）。([#331](https://github.com/rust-windowing/android-ndk-rs/pull/331))
- 在 `ndk-gdb` 中继承 `cargo-apk` 的 `ndk_gdb()` 函数，并使用适当的脚本扩展名跨平台。([#330](https://github.com/rust-windowing/android-ndk-rs/pull/330), [#258](https://github.com/rust-windowing/android-ndk-rs/pull/258))
- 提供 `adb` 路径给 `ndk-gdb`，允许它在没有 `adb` 在 `PATH` 中运行。([#343](https://github.com/rust-windowing/android-ndk-rs/pull/343))
- 从 `ndk-gdb` 中删除 `adb` 的引号，以修复Windows上的 `ndk-gdb`。([#344](https://github.com/rust-windowing/android-ndk-rs/pull/344))
- 通过 `ndk-gdb` 启动Android活动，以在调试器附加之前阻止应用启动。([#345](https://github.com/rust-windowing/android-ndk-rs/pull/345))
- 考虑 `ANDROID_SDK_ROOT` 作为已弃用，而不是 `ANDROID_HOME`。([#346](https://github.com/rust-windowing/android-ndk-rs/pull/346))
- **破坏性变更：** 将 `fn android_dir()` 重命名为 `fn android_user_home()` 并用 `ANDROID_SDK_HOME` 或 `ANDROID_USER_HOME` 种子。([#347](https://github.com/rust-windowing/android-ndk-rs/pull/347))

# 0.7.0 (2022-07-05)

- 修复 NDK r23 `-lgcc` 工作区，以解决包含空格的目标目录。([#298](https://github.com/rust-windowing/android-ndk-rs/pull/298))
- 直接调用 `clang` 而不是通过 NDK 的包装脚本。([#306](https://github.com/rust-windowing/android-ndk-rs/pull/306))
- **破坏性变更：** 将 `Activity::intent_filters` 重命名为 `Activity::intent_filter`。([#305](https://github.com/rust-windowing/android-ndk-rs/pull/305))

# 0.6.0 (2022-06-11)

- **破坏性变更：** 在 `cargo_ndk()` 函数中提供 NDK r23 `-lgcc` 工作区，现在需要 `target_dir` 作为参数。([#286](https://github.com/rust-windowing/android-ndk-rs/pull/286))
- **破坏性变更：** 添加 `disable_aapt_compression` 字段到 `ApkConfig` 以禁用 `aapt` 压缩。([#283](https://github.com/rust-windowing/android-ndk-rs/pull/283))

# 0.5.0 (2022-05-07)

- **破坏性变更：** 默认 `target_sdk_version` 为 `30` 或更低（而不是检测到的 NDK 工具链支持的最高 SDK 版本）
  以更一致地与 Android 向后兼容处理及其日益严格的用法规则进行交互：
  <https://developer.android.com/distribute/best-practices/develop/target-sdk>
- **破坏性变更：** 移除默认插入 `MAIN` intent 过滤器通过自定义序列化函数，这更好地由
  `cargo-apk` 中的默认设置填充。([#241](https://github.com/rust-windowing/android-ndk-rs/pull/241))
- 添加 `android:exported` 属性到清单的 `Activity` 元素。([#242](https://github.com/rust-windowing/android-ndk-rs/pull/242))
- 添加 `android:sharedUserId` 属性到清单的顶级 `manifest` 元素。([#252](https://github.com/rust-windowing/android-ndk-rs/pull/252))
- 添加 `queries` 元素到清单的顶级 `manifest` 元素。([#259](https://github.com/rust-windowing/android-ndk-rs/pull/259))

# 0.4.3 (2021-11-22)

- 从 NDK 根目录的 `source.properties` 中提供 NDK `build_tag` 版本。

# 0.4.2 (2021-08-06)

- 在非UNIX系统上，将UNIX路径分隔符传递给 `aapt`，以确保生成的分隔符与目标设备兼容，而不是主机平台。

# 0.4.1 (2021-08-02)

- 现在只选择 NDK 支持的最高平台作为默认平台。

# 0.4.0 (2021-07-06)

- 添加 `add_runtime_libs` 函数以在APK中包含额外的动态库。

# 0.3.0 (2021-05-10)

- 新的 `ApkConfig` 字段 `apk_name` 现在用于APK文件命名，而不是应用程序标签。
- 库搜索路径更加智能。
- `android:screenOrientation` 可配置。

# 0.2.0 (2021-04-20)

- **破坏性变更：** 将 `Manifest` 重构为适当的（反）序列化结构。`Manifest` 现在几乎与 [一个android清单文件](https://developer.android.com/guide/topics/manifest/manifest-element) 相匹配。
- **破坏性变更：** 取消使用 `Config`，而是直接使用新的 `Manifest` 结构。现在，你可以直接使用几乎所有的相同值来实例化 `Manifest`，而不是使用 `Config::from_config` 来创建 `Manifest`。

# 0.1.4 (2020-11-25)

- 在Windows上，修复了资源文件夹的UNC路径处理。

# 0.1.3 (2020-11-21)

- `android:launchMode` 可配置。

# 0.1.2 (2020-09-15)

- `android:label` 可配置。
- 库搜索路径更加智能。
- `android:screenOrientation` 可配置。

# 0.1.1 (2020-07-15)

- 添加了对自定义intent过滤器的支持。
- 在Windows上，修复了UNC路径处理。
- 修复了当NDK安装没有主机架构后缀在其预构建的LLVM目录上时工具链路径处理。

# 0.1.0 (2020-04-22)

- 初始发布！🎉
