# 未发布

# 0.10.0 (2023-11-30)

- 将MSRV提升到1.70，以反映依赖项更新。
- 将 `ndk-build` 升级到 [`0.10.0`](https://github.com/rust-mobile/cargo-apk/releases/tag/ndk-build-0.10.0)，并修复了各种问题：
  - 改进了基于UID而不是PID的日志过滤；
  - 支持从Android主机构建APK；
  - 现在在 `Application` 和 `Activity` 元素上支持更多的清单属性。
- 基于https://github.com/rust-mobile/cargo-subcommand/pull/17改进了工件，以支持 `[lib]`、`[[bin]]` 和 `[[example]]` 的重命名。([#26](https://github.com/rust-mobile/cargo-apk/pull/26))

# 0.9.7 (2022-12-12)

- 通过升级到 [`cargo-subcommand 0.11.0`](https://github.com/rust-mobile/cargo-subcommand/releases/tag/0.11.0) 来重新实现基于 `$PWD` 或 `--manifest-path` 的默认包选择。([#4](https://github.com/rust-mobile/cargo-apk/pull/4))
- 从 `cargo apk --` 中移除已知参数解析，以避免使参数标志/值丢失，另请参见 [#375](https://github.com/rust-windowing/android-ndk-rs/issues/375)。([#377](https://github.com/rust-windowing/android-ndk-rs/pull/377))

# 0.9.6 (2022-11-23)

- 通过环境变量 `CARGO_APK_<PROFILE>_KEYSTORE` 和 `CARGO_APK_<PROFILE>_KEYSTORE_PASSWORD` 指定签名信息，这些环境变量优先于清单中的签名信息。除了 `dev` 配置文件外，这两个环境变量都是必需的，除非未设置 `CARGO_APK_DEV_KEYSTORE_PASSWORD`，此时将回退到默认密码。([#358](https://github.com/rust-windowing/android-ndk-rs/pull/358))
- 添加 `strip` 选项到 `android` 元数据，允许用户指定在 `cargo` 完成构建但在共享对象复制到APK之前如何处理调试符号。([#356](https://github.com/rust-windowing/android-ndk-rs/pull/356))
- 支持 `[workspace.package]` 从工作区根清单继承的 `version` 字段。([#360](https://github.com/rust-windowing/android-ndk-rs/pull/360))

(0.9.5，于2022-10-14发布，由于无意中通过 `quick-xml` crate 提升MSRV，并在切换到 `clap` 后破坏 `cargo apk --` 解析而被撤销。)

- 更新到 `cargo-subcommand 0.8.0`，使用 `clap` 参数解析器。([#238](https://github.com/rust-windowing/android-ndk-rs/pull/238))
- 通过 `Cargo.toml` 元数据自动化 `adb reverse` 端口转发。([#348](https://github.com/rust-windowing/android-ndk-rs/pull/348))

# 0.9.4 (2022-09-12)

- 通过在打包APK之前对齐库来升级到最新的 `ndk-build`。([#333](https://github.com/rust-windowing/android-ndk-rs/pull/333))
- 支持 `android:resizeableActivity`。([#338](https://github.com/rust-windowing/android-ndk-rs/pull/338))
- 添加 `--device` 参数以通过序列选择 `adb` 设备（参见 `adb devices` 以获取连接的设备和它们的序列号）。([#329](https://github.com/rust-windowing/android-ndk-rs/pull/329))
- 启动应用后打印并跟随 `adb logcat` 输出。([#332](https://github.com/rust-windowing/android-ndk-rs/pull/332))

# 0.9.3 (2022-07-05)

- 允许配置备用调试密钥库位置；要求发布构建的密钥库位置。([#299](https://github.com/rust-windowing/android-ndk-rs/pull/299))
- **破坏性变更：** 将 `Activity::intent_filters` 重命名为 `Activity::intent_filter`。([#305](https://github.com/rust-windowing/android-ndk-rs/pull/305))

# 0.9.2 (2022-06-11)

- 将 NDK r23 `-lgcc` 工作区移动到 `ndk_build::cargo::cargo_ndk()`，以也适用于我们的 `cargo apk --` 调用。([#286](https://github.com/rust-windowing/android-ndk-rs/pull/286))
- 为 [(默认) `dev` 配置文件](https://doc.rust-lang.org/cargo/reference/profiles.html)禁用 `aapt` 压缩。([#283](https://github.com/rust-windowing/android-ndk-rs/pull/283))
- 当用户未提供时，将 `--target` 追加到 blanket `cargo apk --` 调用中。([#287](https://github.com/rust-windowing/android-ndk-rs/pull/287))

# 0.9.1 (2022-05-12)

- 使用 `RUSTFLAGS` 重新实现 NDK r23 `-lgcc` 工作区，以应用于传递的 `cdylib` 编译。

# 0.9.0 (2022-05-07)

- **破坏性变更：** 使用 `min_sdk_version` 而不是 `target_sdk_version` 来选择编译器目标。([#197](https://github.com/rust-windowing/android-ndk-rs/pull/197))
  有关更多详细信息，请参阅 <https://developer.android.com/ndk/guides/sdk-versions#minsdkversion>。
- **破坏性变更：** 默认 `target_sdk_version` 为 `30` 或更低（而不是检测到的 NDK 工具链支持的最高 SDK 版本）
  以更一致地与 Android 向后兼容处理及其日益严格的用法规则进行交互：
  <https://developer.android.com/distribute/best-practices/develop/target-sdk>
  ([#203](https://github.com/rust-windowing/android-ndk-rs/pull/203))
- 允许在 `Cargo.toml` 中提供清单的 `package` 属性。([#236](https://github.com/rust-windowing/android-ndk-rs/pull/236))
- 在 `from_subcommand` 中添加 `MAIN` intent 过滤器，而不是依赖 `ndk-build` 中的自定义序列化函数。([#241](https://github.com/rust-windowing/android-ndk-rs/pull/241))
- 通过 `android:exported="true"` 导出唯一的 `NativeActivity`，以允许在目标 Android S 或更高版本时默认启动它。([#242](https://github.com/rust-windowing/android-ndk-rs/pull/242))
- 现在可以通过 `cargo apk version` 查询 `cargo-apk` 版本。([#218](https://github.com/rust-windowing/android-ndk-rs/pull/218))
- 从 `.cargo/config.toml` 的 `[env]` 部分传播环境变量到进程环境。([#249](https://github.com/rust-windowing/android-ndk-rs/pull/249))

# 0.8.2 (2021-11-22)

- 修复了清单中多个构建工件的情况下的库名称。
- 通过提供将 `libgcc` 重定向到 `libunwind` 的链接器脚本来解决 NDK r23 beta 3 及以上版本中缺少 `libgcc` 的问题。
  有关更多详细信息，请参阅 <https://github.com/rust-windowing/android-ndk-rs/issues/149> 和 <https://github.com/rust-lang/rust/pull/85806>。

# 0.8.1 (2021-08-06)

- 更新到使用 [ndk-build 0.4.2](../ndk-build2/CHANGELOG.md#042-2021-08-06)

# 0.8.0 (2021-07-06)

- 添加 `runtime_libs` 路径到 android 元数据，用于将额外的动态库打包到APK中。

# 0.7.0 (2021-05-10)

- 添加 `cargo apk check`。对于包含 C/C++ 依赖项或目标特定条件编译的编译测试crate非常有用，但不提供 cdylib 目标。
- 添加 `apk_name` 字段到 android 元数据，用于APK文件命名（如果未指定，则默认为 Rust 库名称）。
  从现在起，应用程序标签不再用于此目的，并且可以包含字符串资源ID。

# 0.6.0 (2021-04-20)

- **破坏性变更：** 使用 `ndk-build` 的新（反）序列化 `Manifest` 结构来正确地将 toml 的 `[package.metadata.android]` 序列化为 `AndroidManifest.xml`。
  `[package.metadata.android]` 现在几乎与 [一个android清单文件](https://developer.android.com/guide/topics/manifest/manifest-element) 相匹配。
  请参阅 [README](README.md) 以了解新的 `[package.metadata.android]` 结构和当前支持的所有清单属性。

# 0.5.6 (2020-11-25)

- 使用 `dunce::simplified` 来提取清单的资源和资源文件夹
- 更新到使用 [ndk-build 0.1.4](../ndk-build2/CHANGELOG.md#014-2020-11-25)

# 0.5.5 (2020-11-21)

- 更新到使用 [ndk-build 0.1.3](../ndk-build2/CHANGELOG.md#013-2020-11-21)

# 0.5.4 (2020-11-01)

- 添加了对活动元数据条目的支持。
- 修复工作区中的 glob 成员解析。

# 0.5.3 (2020-10-15)

- 修复 `res` 文件夹解析。

# 0.5.2 (2020-09-15)

- 更新到使用 [ndk-build 0.1.2](../ndk-build2/CHANGELOG.md#012-2020-09-15)

# 0.5.1 (2020-07-15)

- 更新到使用 [ndk-build 0.1.1](../ndk-build2/CHANGELOG.md#011-2020-07-15)

# 0.5.0 (2020-04-22)

- 更新到使用 [ndk-build 0.1.0](../ndk-build2/CHANGELOG.md#010-2020-04-22)
- 几乎3年来的首次发布！🎉
- **破坏性变更：** 许多事情都变了！
