# cargo apk2

[![Actions Status](https://github.com/mzdk100/cargo-apk2/actions/workflows/rust.yml/badge.svg)](https://github.com/mzdk100/cargo-apk2/actions)
[![最新版本](https://img.shields.io/crates/v/cargo-apk.svg?logo=rust)](https://crates.io/crates/cargo-apk2)
[![MSRV](https://img.shields.io/badge/rustc-1.82.0+-ab6000.svg)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![文档](https://docs.rs/cargo-apk2/badge.svg)](https://docs.rs/cargo-apk2)
[![仓库](https://tokei.rs/b1/github/rust-mobile/cargo-apk)](https://github.com/mzdk100/cargo-apk2)
![MIT](https://img.shields.io/badge/License-MIT-green.svg)
![Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-green.svg)

使用Rust语言构建安卓应用的工具，此工具的前身是cargo-apk，由于他已经被标记弃用状态，所以创建了cargo-apk2这个项目。
此工具打包apk非常简单高效，不需要配置Gradle的环境，因此非常适合刚接触Rust语言的新手。

## 安装

1. 直接从 crates.io 上获取:
   ```shell
   cargo install cargo-apk2
   ```
2. 从源代码:
   ```shell
   git clone https://github.com/mzdk100/cargo-apk2
   cargo install --path cargo-apk2/
   ```

## 支持的命令

- `build`: 编译当前包
- `run`: 运行本地包的二进制文件或示例
- `gdb`: 启动连接到 adb 设备的 gdb 会话并加载符号

## Manifest

`cargo` 支持 `metadata` 表，用于配置 `cargo apk2` 等外部工具。
`cargo apk2` 在 `[package.metadata.android]` 下支持以下配置选项：

```toml
[package.metadata.android]
# 指定清单的包属性。
package = "com.foo.bar"

# 指定要构建的目标数组。
build_targets = [ "armv7-linux-androideabi", "aarch64-linux-android", "i686-linux-android", "x86_64-linux-android" ]

# 应用程序的资源文件夹路径。
# 如果未指定，资源将不会包含在APK中。
resources = "path/to/resources_folder"

# 应用程序资产文件夹的路径。
# 如果未指定，资产将不会包含在APK中。
assets = "path/to/assets_folder"

# 最终APK文件的名称。默认为包名。
apk_name = "myapp"

# `default`（或未指定）- 如果存在，调试符号不会被特别处理。
#
# `strip`  - 在将共享库复制到APK之前，从共享库中剥离调试符号。
#
# `split`  - 功能与`strip`相同，不同之处在于调试符号与剥离的共享库一起写入 apk 输出目录，并带有`.dwarf`扩展名。
#
# 注意，`strip`和`split`选项只有在`.so`文件中存在调试符号时才会生效，
# 启用https://doc.rust-lang.org/cargo/reference/profiles.html#strip 或
# https://doc.rust-lang.org/cargo/reference/profiles.html#split-debuginfo
# 在您的cargo清单中可以导致调试符号不再存在于`.so`文件中。
strip = "default"

# 包含要在运行时动态加载的额外共享库的文件夹。
# 根据指定的build_targets，匹配`libs_folder/${android_abi}/*.so`的文件会被添加到APK中。
runtime_libs = "path/to/libs_folder"

# 与其他应用共享的Linux用户ID的名称。默认情况下，Android为每个应用分配一个唯一的用户ID。
# 但是，如果此属性为两个或更多应用设置为相同的值，它们将共享相同的ID，前提是它们的证书集相同。
# 具有相同用户ID的应用可以访问彼此的数据，并且如果需要，可以在同一进程中运行。
shared_user_id = "my.shared.user.id"

# 默认为`dev`配置文件的`$HOME/.android/debug.keystore`。仅当此文件不存在时才会生成新的debug.keystore。
# 其他配置文件永远不会自动生成keystore。
#
# keystore路径可以是绝对路径，也可以是相对于Cargo.toml文件的相对路径。
#
# 可以设置环境变量`CARGO_APK_<PROFILE>_KEYSTORE`和`CARGO_APK_<PROFILE>_KEYSTORE_PASSWORD`，
# 分别指定keystore路径和keystore密码。配置文件部分遵循与`<cfg>`相同的规则，
# 它是配置文件名称的大写形式，其中`-`被替换为`_`。
#
# 如果存在，它们将优先于清单中的签名信息。
[package.metadata.android.signing._profile_]
path = "relative/or/absolute/path/to/my.keystore"
keystore_password = "android"

# 参见 https://developer.android.com/guide/topics/manifest/uses-sdk-element
#
# 默认的`min_sdk_version`为24，`target_sdk_version`为35（如果检测到的NDK不支持，则为较低的版本）。
[package.metadata.android.sdk]
min_sdk_version = 24
target_sdk_version = 35
max_sdk_version = 35

# 参见 https://developer.android.com/guide/topics/manifest/uses-feature-element
#
# 注意：可以有多个.uses_feature条目。
[[package.metadata.android.uses_feature]]
name = "android.hardware.vulkan.level"
required = true
version = 1

# 参见 https://developer.android.com/guide/topics/manifest/uses-permission-element
#
# 注意：可以有多个.uses_permission条目。
[[package.metadata.android.uses_permission]]
name = "android.permission.WRITE_EXTERNAL_STORAGE"
max_sdk_version = 35

# 参见 https://developer.android.com/guide/topics/manifest/queries-element#provider
[[package.metadata.android.queries.provider]]
authorities = "org.khronos.openxr.runtime_broker;org.khronos.openxr.system_runtime_broker"
# 注意：`name`属性通常不是查询提供者的必需属性，但作为解决aapt错误缺少`android:name`属性的
# 工作方法是非可选的。如果/当cargo-apk迁移到aapt2时，它将变为可选。
name = "org.khronos.openxr"

# 参见 https://developer.android.com/guide/topics/manifest/queries-element#intent
[[package.metadata.android.queries.intent]]
actions = ["android.intent.action.SEND"]

# 参见 https://developer.android.com/guide/topics/manifest/queries-element#intent
# 注意：可以有多个.data条目。
[[package.metadata.android.queries.intent.data]]
mime_type = "image/jpeg"

# 参见 https://developer.android.com/guide/topics/manifest/queries-element#package
[[package.metadata.android.queries.package]]
name = "org.freedesktop.monado.openxr_runtime.in_process"

# 参见 https://developer.android.com/guide/topics/manifest/application-element
[package.metadata.android.application]

# 参见 https://developer.android.com/guide/topics/manifest/application-element#debug
#
# 默认为false。
debuggable = false

# 参见 https://developer.android.com/guide/topics/manifest/application-element#theme
#
# 示例显示将应用程序的主题设置为全屏。
theme = "@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen"

# 应用程序的任何mipmap级别的图标虚拟路径。
# 如果未指定，图标将不会包含在APK中。
icon = "@mipmap/ic_launcher"

# 参见 https://developer.android.com/guide/topics/manifest/application-element#label
#
# 默认为编译后的工件名称。
label = "Application Name"

# 参见 https://developer.android.com/guide/topics/manifest/application-element#extractNativeLibs
extract_native_libs = true

# 参见 https://developer.android.com/guide/topics/manifest/application-element#usesCleartextTraffic
uses_cleartext_traffic = true

# 参见 https://developer.android.com/guide/topics/manifest/meta-data-element
#
# 注意：可以有多个.meta_data条目。
# 注意：当前不支持`resource`属性。
[[package.metadata.android.application.meta_data]]
name = "com.samsung.android.vr.application.mode"
value = "vr_only"

# 参见 https://developer.android.com/guide/topics/manifest/activity-element
[package.metadata.android.application.activity]

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#config
#
# 默认为"orientation|keyboardHidden|screenSize"。
config_changes = "orientation"

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#label
#
# 默认为应用程序的标签。
label = "Activity Name"

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#lmode
#
# 默认为"standard"。
launch_mode = "singleTop"

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#screen
#
# 默认为"unspecified"。
orientation = "landscape"

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#exported
#
# 默认未设置，或当目标Android >= 31（S及更高版本）时为true。
exported = true

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#resizeableActivity
#
# 默认在Android >= 24上为true，对较早的API级别无效果。
resizeable_activity = false

# 参见 https://developer.android.com/guide/topics/manifest/activity-element#always
always_retain_task_state = true

# 参见 https://developer.android.com/guide/topics/manifest/meta-data-element
#
# 注意：可以有多个.meta_data条目。
# 注意：当前不支持`resource`属性。
[[package.metadata.android.application.activity.meta_data]]
name = "com.oculus.vr.focusaware"
value = "true"

# 参见 https://developer.android.com/guide/topics/manifest/intent-filter-element
#
# 注意：可以有多个.intent_filter条目。
[[package.metadata.android.application.activity.intent_filter]]
# 参见 https://developer.android.com/guide/topics/manifest/action-element
actions = ["android.intent.action.VIEW", "android.intent.action.WEB_SEARCH"]
# 参见 https://developer.android.com/guide/topics/manifest/category-element
categories = ["android.intent.category.DEFAULT", "android.intent.category.BROWSABLE"]

# 参见 https://developer.android.com/guide/topics/manifest/data-element
#
# 注意：可以有多个.data条目。
# 注意：未指定属性将排除在最终数据规范之外。
[[package.metadata.android.application.activity.intent_filter.data]]
scheme = "https"
host = "github.com"
port = "8080"
path = "/rust-windowing/android-ndk-rs/tree/master/cargo-apk"
path_prefix = "/rust-windowing/"
mime_type = "image/jpeg"

# 通过`adb reverse`设置反向端口转发，这意味着如果Android设备连接到`localhost`上的端口`1338`，
# 它将被路由到主机上的端口`1338`。源和目标端口可以不同，请参阅`adb`帮助页面以获取可能的配置。
[package.metadata.android.reverse_port_forward]
"tcp:1338" = "tcp:1338"
```


如果“cargo apk2”不支持清单属性，请随意创建 PR 来添加缺失的属性。