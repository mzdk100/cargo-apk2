# cargo-apk2

[![Actions Status](https://github.com/mzdk100/cargo-apk2/actions/workflows/rust.yml/badge.svg)](https://github.com/mzdk100/cargo-apk2/actions)
[![最新版本](https://img.shields.io/crates/v/cargo-apk.svg?logo=rust)](https://crates.io/crates/cargo-apk2)
[![MSRV](https://img.shields.io/badge/rustc-1.86.0+-ab6000.svg)](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/)
[![文档](https://docs.rs/cargo-apk2/badge.svg)](https://docs.rs/cargo-apk2)
[![仓库](https://tokei.rs/b1/github/mzdk100/cargo-apk2)](https://github.com/mzdk100/cargo-apk2)
![MIT](https://img.shields.io/badge/License-MIT-green.svg)
![Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-green.svg)

使用 Rust 语言构建安卓应用的工具。此工具的前身是 [`cargo-apk`]，由于它的开发进度长期停滞，我们创建了 `cargo-apk2` 这个项目。

此工具打包 APK 非常简单高效，无需配置 Gradle 环境，因此非常适合刚接触 Rust 语言的新手。此工具主要用于创建基于 [`NativeActivity`] 的应用。请参阅 [`android-activity`]、[`ndk`] 文档。

与 `cargo-apk` 相比，`cargo-apk2` 还支持将各种基于 Java 虚拟机的源代码编译为 DEX 文件作为 APK 的一部分；支持声明多个 Activity 和 Service。

[`cargo-apk`]: https://crates.io/crates/cargo-apk
[`NativeActivity`]: https://developer.android.google.cn/reference/android/app/NativeActivity
[`android-activity`]: https://crates.io/crates/android-activity
[`ndk`]: https://crates.io/crates/ndk

## 安装

1. 直接通过 crates.io 获取：
   ```shell
   cargo install cargo-apk2
   ```
2. 从源代码构建：
   ```shell
   git clone https://github.com/mzdk100/cargo-apk2
   cargo install --path cargo-apk2/
   ```

## Crate 配置

`cargo-apk` 要求在 Cargo 配置中声明 `cdylib` 类型的[编译目标]。在仅需要编译为动态库的情况下, 可以这样配置 `Cargo.toml`：

```toml
[lib]
# 可选地，支持以 `path = "..."` 指定自定义的 lib 路径。
crate-type = ["cdylib"]
```

此工具的源码仓库中提供了一些示例配置。

[编译目标]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html

## 支持的命令

- `build`: 编译当前包
- `run`: 运行本地包的二进制文件或示例
- `gdb`: 启动连接到 adb 设备的 gdb 会话并加载调试符号

## Manifest

`cargo` 支持 `metadata` 表，用于配置 `cargo apk2` 等外部工具。
`cargo apk2` 在 `[package.metadata.android]` 下支持以下配置选项：

```toml
[package.metadata.android]
# 以 aapt2 构建工具取代 aapt 用于编译资源。
use_aapt2 = true

# 指定 Android Manifest 中的包名属性。
# 参见 https://developer.android.google.cn/guide/topics/manifest/manifest-element#package
package = "com.foo.bar"

# 以数组形式列出所有需要支持的编译目标。
build_targets = [ "armv7-linux-androideabi", "aarch64-linux-android", "i686-linux-android", "x86_64-linux-android" ]

# 应用程序的资源文件夹路径。
# 如果未指定，资源将不会包含在 APK 中。
resources = "path/to/resources_folder"

# 应用程序 assets 文件夹的路径。
# 如果未指定，APK 将不会包含该文件夹。
assets = "path/to/assets_folder"

# Java 源代码的文件夹路径。
# 如果未指定，则有可能跳过 Java 编译步骤。
java_sources = "path/to/java"

# Kotlin 源代码的文件夹路径。
# 如果未指定，则会跳过 Kotlin 编译步骤。
kotlin_sources = "path/to/kotlin"

# Groovy 源代码的文件夹路径。
# 如果未指定，则会跳过 Groovy 编译步骤。
groovy_sources = "path/to/groovy"

# Scala 源代码的文件夹路径。
# 如果未指定，则会跳过 Scala 编译步骤。
scala_sources = "path/to/scala"

# 最终 APK 文件的名称。默认为 Rust 包名。
apk_name = "myapp"

# `default`（或未指定）- 如果编译后的共享库中存在调试符号，本工具不会处理它们。
#
# `strip`  - 在将共享库复制到 APK 之前，从共享库中剥离调试符号。
#
# `split`  - 功能与 `strip` 相同，不同之处在于调试符号与被剥离调试符号的共享库会分别被写入 APK
#            输出目录，其中调试符号文件带有 `.dwarf` 扩展名。
#
# 注意，`strip` 和 `split` 选项只有在 `.so` 文件中存在调试符号时才会生效，
# 在您的 `Cargo.toml` 中启用 https://doc.rust-lang.org/cargo/reference/profiles.html#strip
# 或 https://doc.rust-lang.org/cargo/reference/profiles.html#split-debuginfo
# 可使得调试符号不再存在于 `.so` 文件中。
strip = "default"

# 包含需要在运行时加载的额外共享库（动态链接库）的文件夹。
# 根据指定的 `build_targets`，匹配 `libs_folder/${android_abi}/*.so` 的文件会被添加到 APK 中。
runtime_libs = "path/to/libs_folder"

# 与其他应用共享的 Linux 用户 ID 的名称。默认情况下，Android 为每个应用分配一个唯一的用户 ID。
# 但是，如果此属性为两个或更多应用设置为相同的值，它们将共享相同的 ID，前提是它们的证书集相同。
# 具有相同用户 ID 的应用可以访问彼此的数据，并且如果需要，可以在同一进程中运行。
shared_user_id = "my.shared.user.id"

# 对于 Cargo `dev` 配置文件，默认为 `$HOME/.android/debug.keystore`。仅当此文件不存在时才会生成
# 新的 `debug.keystore`。对于其他配置文件，永远不会自动生成 keystore。
#
# keystore 路径可以是绝对路径，也可以是相对于 `Cargo.toml` 文件的相对路径。
#
# 可以设置环境变量 `CARGO_APK_<PROFILE>_KEYSTORE` 和 `CARGO_APK_<PROFILE>_KEYSTORE_PASSWORD`，
# 分别指定 keystore 路径和 keystore 密码。配置文件部分遵循与 `<cfg>` 相同的规则，
# 它是配置文件名称的大写形式，其中 `-` 被替换为 `_`。
#
# 如果存在，它们将优先于 `Cargo.toml` 中的签名信息。
[package.metadata.android.signing._profile_]
path = "relative/or/absolute/path/to/my.keystore"
keystore_password = "android"

# 参见 https://developer.android.google.cn/guide/topics/manifest/uses-sdk-element
#
# `min_sdk_version` 默认为 24，`target_sdk_version` 默认为 36（如果检测到的 NDK 不支持，则为较低的版本）。
[package.metadata.android.sdk]
min_sdk_version = 24
target_sdk_version = 36
max_sdk_version = 36

# 参见 https://developer.android.google.cn/guide/topics/manifest/uses-feature-element
#
# 注意：可以有多个 `.uses_feature` 条目。
[[package.metadata.android.uses_feature]]
name = "android.hardware.vulkan.level"
required = true
version = 1

# 参见 https://developer.android.google.cn/guide/topics/manifest/uses-permission-element
#
# 注意：可以有多个 `.uses_permission` 条目。
[[package.metadata.android.uses_permission]]
name = "android.permission.WRITE_EXTERNAL_STORAGE"
max_sdk_version = 36

# 参见 https://developer.android.google.cn/guide/topics/manifest/queries-element#provider
[[package.metadata.android.queries.provider]]
authorities = "org.khronos.openxr.runtime_broker;org.khronos.openxr.system_runtime_broker"
# 注意：`name` 属性通常不是查询提供者的必需属性，但如果 aapt 报告缺少 `android:name` 属性
# 的错误，则需要设置该属性以应对错误。当指定使用 aapt2 时，它是一个可选属性。
name = "org.khronos.openxr"

# 参见 https://developer.android.google.cn/guide/topics/manifest/queries-element#intent
[[package.metadata.android.queries.intent]]
actions = ["android.intent.action.SEND"]

# 参见 https://developer.android.google.cn/guide/topics/manifest/queries-element#intent
# 注意：可以有多个 `.data` 条目。
[[package.metadata.android.queries.intent.data]]
mime_type = "image/jpeg"

# 参见 https://developer.android.google.cn/guide/topics/manifest/queries-element#package
[[package.metadata.android.queries.package]]
name = "org.freedesktop.monado.openxr_runtime.in_process"

# 参见 https://developer.android.google.cn/guide/topics/manifest/application-element
[package.metadata.android.application]

# 参见 https://developer.android.google.cn/guide/topics/manifest/application-element#debug
#
# 默认为 false。
debuggable = false

# 参见 https://developer.android.google.cn/guide/topics/manifest/application-element#theme
#
# 示例显示将应用程序的主题设置为全屏。
theme = "@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen"

# 应用程序的任何 mipmap 级别的图标虚拟路径。
# 如果未指定，图标将不会被包含在 APK 中。
icon = "@mipmap/ic_launcher"

# 参见 https://developer.android.google.cn/guide/topics/manifest/application-element#label
#
# 指定面向用户的应用名称。默认为编译后的工件名称（Rust 包名）。
label = "Application Name"

# 参见 https://developer.android.google.cn/guide/topics/manifest/application-element#extractNativeLibs
extract_native_libs = true

# 参见 https://developer.android.google.cn/guide/topics/manifest/application-element#usesCleartextTraffic
uses_cleartext_traffic = true

# 参见 https://developer.android.google.cn/guide/topics/manifest/meta-data-element
#
# 注意：可以有多个 `.meta_data` 条目。
[[package.metadata.android.application.meta_data]]
name = "com.samsung.android.vr.application.mode"
value = "vr_only"

# 支持多个 activity 元素（应至少有一个），cargo-apk2 默认不会生成任何隐含 Activity。
# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element
[[package.metadata.android.application.activity]]

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#config
#
# 默认为 "orientation|keyboardHidden|screenSize"。
config_changes = "orientation"

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#label
#
# 默认为应用程序的标签。
label = "Activity Name"

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#lmode
#
# 默认为 "standard"。
launch_mode = "singleTop"

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#screen
#
# 默认为 "unspecified"。
orientation = "landscape"

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#exported
#
# 默认未设置，但当目标 Android API >= 31（S 及更高版本）时默认为 true。
exported = true

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#resizeableActivity
#
# 默认在 Android API >= 24 上为 true，对较早的 API 级别无效果。
resizeable_activity = false

# 参见 https://developer.android.google.cn/guide/topics/manifest/activity-element#always
always_retain_task_state = true

# 参见 https://developer.android.google.cn/guide/topics/manifest/meta-data-element
#
# 注意：可以有多个 `.meta_data` 条目。
[[package.metadata.android.application.activity.meta_data]]
name = "com.oculus.vr.focusaware"
value = "true"

# 参见 https://developer.android.google.cn/guide/topics/manifest/intent-filter-element
#
# 注意：可以有多个 `.intent_filter` 条目。
[[package.metadata.android.application.activity.intent_filter]]
# 参见 https://developer.android.google.cn/guide/topics/manifest/action-element
actions = ["android.intent.action.VIEW", "android.intent.action.WEB_SEARCH"]
# 参见 https://developer.android.google.cn/guide/topics/manifest/category-element
categories = ["android.intent.category.DEFAULT", "android.intent.category.BROWSABLE"]

# 参见 https://developer.android.google.cn/guide/topics/manifest/data-element
#
# 注意：可以有多个 `.data` 条目。
# 注意：未指定属性将被排除在最终的数据属性清单之外。
[[package.metadata.android.application.activity.intent_filter.data]]
scheme = "https"
host = "github.com"
port = "8080"
path = "/rust-windowing/android-ndk-rs/tree/master/cargo-apk"
path_prefix = "/rust-windowing/"
mime_type = "image/jpeg"

# 支持多个 service 元素，同样支持 intent-filter。
# 参见 https://developer.android.google.cn/guide/topics/manifest/service-element
[[package.metadata.android.application.service]]

# 实现服务的 Service 子类的名称。这是一个完全限定的类名称，例如 "com.example.project.RoomService"。
# 不过，作为一种简写形式，如果名称的第一个字符是句点（例如 ".RoomService"），则会将其附加到
# <manifest> 元素中指定的软件包名称。
# 发布应用后，除非您已设置 `android:exported="false"`，否则请勿更改此名称。没有默认值。必须指定此名称。
name = ".MyService"

# 确定系统是否可以实例化服务。如果可以实例化，则设为 "true"，否则设为 "false"。默认值为 "true"。
# <application> 元素具有自己的 enabled 属性，该属性适用于所有应用组件，包括服务。
# <application> 和 <service> 属性必须都设为 "true"（这正是它们两者的默认设置），才会启用服务。
# 如果其中任一属性设为 "false"，则表示服务已停用；无法对其进行实例化。
enabled = true

# 实体启动服务或绑定到服务所需的权限的名称。如果没有向 startService()、bindService() 或 stopService()
# 的调用方授予此权限，该方法将不起作用，且系统不会将 Intent 对象传送给服务。
# 如果未设置该属性，则对服务应用由 <application> 元素的 permission 属性所设置的权限。如果二者均未设置，
# 则服务不受权限保护。
permission = "android.permission.BIND_JOB_SERVICE"

# 运行服务的进程的名称。通常，应用的所有组件都会在为应用创建的默认进程中运行。它与应用软件包的名称相同。
# <application> 元素的 process 属性可以为所有组件设置不同的默认值。不过，组件可以使用自己的 process 属性
# 替换默认属性，从而允许您跨多个进程分布应用。
# 如果为此属性分配的名称以英文冒号 (:) 开头，则系统会在需要时创建应用专用的新进程，并且服务会在该进程中运行。
# 如果进程名称以小写字符开头，则服务将在采用该名称的全局进程中运行，前提是它具有相应权限。这样，不同应用
# 中的组件就可以共享进程，从而减少资源使用量。
process = ":my_service"

[[package.metadata.android.application.service.meta_data]]
name = "android.accessibilityservice"
# 引用 XML 资源
resource = "@xml/accessibilityservice"

# 通过 `adb reverse` 设置反向端口转发，这意味着如果 Android 设备连接到 `localhost` 上的端口 `1338`，
# 它将被路由到主机上的端口 `1338`。源和目标端口可以不同，请参阅 `adb` 帮助页面以获取可能的配置。
[package.metadata.android.reverse_port_forward]
"tcp:1338" = "tcp:1338"
```

如果 `cargo apk2` 不支持您需要的某些 Android Manifest 属性，请随意创建 PR 来添加缺失的属性。

## 环境变量说明

`cargo-apk2` 在构建过程中会自动设置以下环境变量，方便在 `build.rs` 或自定义脚本中使用：

- `CARGO_APK2_APK_NAME`：当前生成的 APK 文件名。
- `CARGO_APK2_PACKAGE`：APK 的包名。
- `CARGO_APK2_ASSETS_DIR`：assets 目录的绝对路径。
- `CARGO_APK2_RESOURCES_DIR`：resources 目录的绝对路径。
- `CARGO_APK2_CLASSES_DIR`：用于存放编译后的 Java 类文件目录。
- `CARGO_APK2_RUNTIME_LIBS_DIR`：运行时动态库目录。
- `CARGO_APK2_JAVA_HOME`：Java 环境根目录。
- `CARGO_APK2_SDK_HOME`：Android SDK 根目录。
- `CARGO_APK2_ANDROID_JAR`：当前 `targetSdkVersion` 对应的 `android.jar` 路径。
- `CARGO_APK2_PLATFORM_DIR`：当前 `targetSdkVersion` 对应的 platform 目录。
- `CARGO_APK2_BUILD_TOOLS_VERSION`：当前使用的 Android Build Tools 版本号。
- `CARGO_APK2_MIN_SDK_VERSION`：`minSdkVersion`。
- `CARGO_APK2_TARGET_SDK_VERSION`：`targetSdkVersion`。
- `CARGO_APK2_ARTIFACT`：动态库输出路径。

这些变量可用于自动化脚本、资源处理、Java 集成等高级场景。
