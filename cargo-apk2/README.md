# cargo-apk2

[![Actions Status](https://github.com/mzdk100/cargo-apk2/actions/workflows/rust.yml/badge.svg)](https://github.com/mzdk100/cargo-apk2/actions)
[![Latest version](https://img.shields.io/crates/v/cargo-apk.svg?logo=rust)](https://crates.io/crates/cargo-apk2)
[![MSRV](https://img.shields.io/badge/rustc-1.90.0+-ab6000.svg)](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/)
[![Documentation](https://docs.rs/cargo-apk2/badge.svg)](https://docs.rs/cargo-apk2)
[![Repository](https://tokei.rs/b1/github/mzdk100/cargo-apk2)](https://github.com/mzdk100/cargo-apk2)
![MIT](https://img.shields.io/badge/License-MIT-green.svg)
![Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-green.svg)

English | [中文](https://github.com/mzdk100/cargo-apk2/blob/main/cargo-apk2/README-ZH.md)

Tool for creating Android packages from native Rust crates. We have created this tool because the predecessor of this tool ([`cargo-apk`]) has stagnated for over a year at the time of writing this documentation.

This tool requires minimal setup and tooling, without the requirement of Gradle configuration. Ideal for apps that provide a [`NativeActivity`] via [`android-activity`] and [`ndk`].

In addition to features available in `cargo-apk`, this tool supports compiling sources of JVM-based languages into DEX data to be packaged in the APK file; declaration of multiple activities and even services are supported.

[`cargo-apk`]: https://crates.io/crates/cargo-apk
[`NativeActivity`]: https://developer.android.com/reference/android/app/NativeActivity
[`android-activity`]: https://crates.io/crates/android-activity
[`ndk`]: https://crates.io/crates/ndk

## Installation

From crates.io:

```shell
cargo install cargo-apk2
```

From locally downloaded or cloned source:

```shell
git clone https://github.com/mzdk100/cargo-apk2
cargo install --path cargo-apk2/
```

## Crate configuration

Android and thus `cargo-apk` require the [Cargo Target] to be a Shared Library, corresponding to Rust's `cdylib` `crate-type`.  For the sole library in a crate, configure this as follows in `Cargo.toml`:

```toml
[lib]
# Optionally supports a different library location using `path = "..."`
crate-type = ["cdylib"]
```

The source repository of this tool provides some example configurations.

[Cargo Target]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html

## Commands

- `build`: Compile the selected crate and package it into an APK
- `run`: Compile, install and run the selected crate/package on an attached Android device via `adb`
- `gdb`: Start a gdb session on an attached Android device via `adb`, with symbols loaded

## Examples

This repository includes various example projects demonstrating different features of `cargo-apk2`:

- `egui-examples/` - Examples using egui framework
- `groovy-examples/` - Examples with Groovy integration
- `java-examples/` - Examples with Java integration
- `kotlin-examples/` - Examples with Kotlin integration
- `ndk-examples/` - Examples using Android NDK
- `scala-examples/` - Examples with Scala integration

To run any example, navigate to its directory and execute:

```shell
cd egui-examples
cargo apk2 run
```

Make sure you have an Android device connected via `adb` and have installed `cargo-apk2` before running the examples.

## Manifest

`cargo apk` reads additional configuration from Cargo's `[package.metadata]` table. The following configuration options are supported by `cargo apk` under `[package.metadata.android]`:

```toml
[package.metadata.android]
# Use `aapt2` instead of `aapt` for compiling application resources.
use_aapt2 = true

# Specifies the package property of the manifest.
# See https://developer.android.com/guide/topics/manifest/manifest-element#package
package = "com.foo.bar"

# Specifies the array of targets to build for.
build_targets = [ "armv7-linux-androideabi", "aarch64-linux-android", "i686-linux-android", "x86_64-linux-android" ]

# Path to your application's resources folder.
# If not specified, resources will not be included in the APK.
resources = "path/to/resources_folder"

# Path to the folder containing your application's assets.
# If not specified, assets will not be included in the APK.
assets = "path/to/assets_folder"

# Directory path of Java source files.
# If not specified, the Java compilation process might be skipped.
java_sources = "path/to/java"

# Directory path of Kotlin source files.
# If not specified, the Kotlin compilation process will be skipped.
kotlin_sources = "path/to/kotlin"

# Directory path of Groovy source files.
# If not specified, the Groovy compilation process will be skipped.
groovy_sources = "path/to/groovy"

# Directory path of Scala source files.
# If not specified, the Scala compilation process will be skipped.
scala_sources = "path/to/scala"

# Name for final APK file.
# Defaults to package name.
apk_name = "myapp"

# `default` (or unspecified) - Debug symbols, if they exist, are not treated
#                              specially.
#
# `strip`                    - Debug symbols are stripped from the shared
#                              libraries before being copied into the APK.
#
# `split`                    - Functions the same as `strip`, except the debug
#                              symbols are written to the apk output directory
#                              alongside the stripped shared libraries, with
#                              a `.dwarf` extension.
#
# Note that the `strip` and `split` options will only have an effect if
# debug symbols are present in the `.so` file(s) produced by your build, enabling
# https://doc.rust-lang.org/cargo/reference/profiles.html#strip or
# https://doc.rust-lang.org/cargo/reference/profiles.html#split-debuginfo
# in your cargo manifest can cause debug symbols to no longer be present
# in the `.so`.
strip = "default"

# Folder containing extra shared libraries intended to be dynamically loaded at runtime.
# Files matching `libs_folder/${android_abi}/*.so` are added to the apk
# according to the specified build_targets.
runtime_libs = "path/to/libs_folder"

# Automatically bundle libc++_shared.so from NDK into the APK.
# Many Rust crates (PyO3, winit, etc.) require the C++ runtime library.
# This setting enables automatic detection and bundling of libc++_shared.so
# from the NDK. Only effective when the user hasn't provided their own runtime_libs.
include_cplusplus_shared = true

# Apply compatibility fix for Android 5 (API < 24) on 32-bit architectures.
# 64-bit fseek and ftell can break builds for 32-bit architectures on older Android versions.
# When enabled, this option silently replaces fseeko with fseek and ftello with ftell
# for armeabi-v7a architecture when targeting API < 24.
# Note: This will crash with files larger than 2GB, but provides compatibility
# for legacy Android 5 devices.
legacy_fseek_fix = true

# The name of a Linux user ID that is shared with other apps. By
# default, Android assigns each app its own unique user ID. However, if
# this attribute is set to the same value for two or more apps, they all
# share the same ID, provided that their certificate sets are identical.
# Apps with the same user ID can access each other's data and, if
# desired, run in the same process.
shared_user_id = "my.shared.user.id"

# Defaults to `$HOME/.android/debug.keystore` for the `dev` profile. Will ONLY
# generate a new debug.keystore if this file does NOT exist. A keystore is never
# auto-generated for other profiles.
#
# The keystore path can be absolute, or relative to the Cargo.toml file.
#
# The environment variables `CARGO_APK_<PROFILE>_KEYSTORE` and
# `CARGO_APK_<PROFILE>_KEYSTORE_PASSWORD` can be set to a keystore path
# and keystore password respectively. The profile portion follows the same rules
# as `<cfg>`, it is the uppercased profile name with `-` replaced with `_`.
#
# If present they take precedence over the signing information in the manifest.
[package.metadata.android.signing.<profile>]
path = "relative/or/absolute/path/to/my.keystore"
keystore_password = "android"

# See https://developer.android.com/guide/topics/manifest/uses-sdk-element
#
# Defaults to a `min_sdk_version` of `24` and `target_sdk_version` of `36`
# (or lower if the detected NDK doesn't support this).
[package.metadata.android.sdk]
min_sdk_version = 24
target_sdk_version = 36
max_sdk_version = 36

# See https://developer.android.com/guide/topics/manifest/uses-feature-element
#
# Note: there can be multiple .uses_feature entries.
[[package.metadata.android.uses_feature]]
name = "android.hardware.vulkan.level"
required = true
version = 1

# See https://developer.android.com/guide/topics/manifest/uses-permission-element
#
# Note: there can be multiple .uses_permission entries.
[[package.metadata.android.uses_permission]]
name = "android.permission.WRITE_EXTERNAL_STORAGE"
max_sdk_version = 36

# See https://developer.android.com/guide/topics/manifest/queries-element#provider
[[package.metadata.android.queries.provider]]
authorities = "org.khronos.openxr.runtime_broker;org.khronos.openxr.system_runtime_broker"
# Note: The `name` attribute is normally not required for a queries provider, but is non-optional
# as a workaround for aapt throwing errors about missing `android:name` attribute.
# This is optional when aapt2 is used.
name = "org.khronos.openxr"

# See https://developer.android.com/guide/topics/manifest/queries-element#intent
[[package.metadata.android.queries.intent]]
actions = ["android.intent.action.SEND"]

# See https://developer.android.com/guide/topics/manifest/queries-element#intent
# Note: there can be several .data entries.
[[package.metadata.android.queries.intent.data]]
mime_type = "image/jpeg"

# See https://developer.android.com/guide/topics/manifest/queries-element#package
[[package.metadata.android.queries.package]]
name = "org.freedesktop.monado.openxr_runtime.in_process"

# See https://developer.android.com/guide/topics/manifest/application-element
[package.metadata.android.application]

# See https://developer.android.com/guide/topics/manifest/application-element#debug
#
# Defaults to false.
debuggable = false

# See https://developer.android.com/guide/topics/manifest/application-element#theme
#
# Example shows setting the theme of an application to fullscreen.
theme = "@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen"

# Virtual path your application's icon for any mipmap level.
# If not specified, an icon will not be included in the APK.
icon = "@mipmap/ic_launcher"

# See https://developer.android.com/guide/topics/manifest/application-element#label
#
# User-readable application name. Defaults to the compiled artifact's name.
label = "Application Name"

# See https://developer.android.com/guide/topics/manifest/application-element#extractNativeLibs
extract_native_libs = true

# See https://developer.android.com/guide/topics/manifest/application-element#usesCleartextTraffic
uses_cleartext_traffic = true

# See https://developer.android.com/guide/topics/manifest/application-element#allowNativeHeapPointerTagging
allow_native_heap_pointer_tagging = true

# See https://developer.android.com/guide/topics/manifest/application-element#requestLegacyExternalStorage
request_legacy_external_storage = true

# See https://developer.android.com/guide/topics/manifest/meta-data-element
#
# Note: there can be several .meta_data entries.
[[package.metadata.android.application.meta_data]]
name = "com.samsung.android.vr.application.mode"
value = "vr_only"

# Supports multiple `activity` elements (there should be at least one); by default
# `cargo-apk2` does not generate any activity implicitly.
# See https://developer.android.com/guide/topics/manifest/activity-element
[[package.metadata.android.application.activity]]

# See https://developer.android.com/guide/topics/manifest/activity-element#config
#
# Defaults to "orientation|keyboardHidden|screenSize".
config_changes = "orientation"

# See https://developer.android.com/guide/topics/manifest/activity-element#label
#
# Defaults to the application's label.
label = "Activity Name"

# See https://developer.android.com/guide/topics/manifest/activity-element#lmode
#
# Defaults to "standard".
launch_mode = "singleTop"

# See https://developer.android.com/guide/topics/manifest/activity-element#screen
#
# Defaults to "unspecified".
orientation = "landscape"

# See https://developer.android.com/guide/topics/manifest/activity-element#exported
#
# Unset by default, or true when targeting Android >= 31 (S and up).
exported = true

# See https://developer.android.com/guide/topics/manifest/activity-element#resizeableActivity
#
# Defaults to true on Android >= 24, no effect on earlier API levels
resizeable_activity = false

# See https://developer.android.com/guide/topics/manifest/activity-element#always
always_retain_task_state = true

# See https://developer.android.com/guide/topics/manifest/meta-data-element
#
# Note: there can be several .meta_data entries.
[[package.metadata.android.application.activity.meta_data]]
name = "com.oculus.vr.focusaware"
value = "true"

# See https://developer.android.com/guide/topics/manifest/intent-filter-element
#
# Note: there can be several .intent_filter entries.
[[package.metadata.android.application.activity.intent_filter]]
# See https://developer.android.com/guide/topics/manifest/action-element
actions = ["android.intent.action.VIEW", "android.intent.action.WEB_SEARCH"]
# See https://developer.android.com/guide/topics/manifest/category-element
categories = ["android.intent.category.DEFAULT", "android.intent.category.BROWSABLE"]

# See https://developer.android.com/guide/topics/manifest/data-element
#
# Note: there can be several .data entries.
# Note: not specifying an attribute excludes it from the final data specification.
[[package.metadata.android.application.activity.intent_filter.data]]
scheme = "https"
host = "github.com"
port = "8080"
path = "/rust-windowing/android-ndk-rs/tree/master/cargo-apk"
path_prefix = "/rust-windowing/"
mime_type = "image/jpeg"

# Supports multiple `service` elements, and they can have intent filters as well.
# See https://developer.android.com/guide/topics/manifest/service-element
[[package.metadata.android.application.service]]

# The name of the Service subclass that implements the service. This is a fully qualified
# class name, such as "com.example.project.RoomService". However, as a shorthand, if the
# first character of the name is a period, such as ".RoomService", it is appended to the
# package name specified in the <manifest> element.
# Once you publish your application, don't change this name, unless you set
# `android:exported="false"`. There is no default. The name must be specified. 
name = ".MyService"

# Whether the service can be instantiated by the system. It's "true" if it can be, and
# "false" if not. The default value is "true".
# The <application> element has its own enabled attribute that applies to all application
# components, including services. The <application> and <service> attributes must both be
# "true", as they both are by default, for the service to be enabled. If either is "false",
# the service is disabled and can't be instantiated. 
enabled = true

# The name of a permission that an entity needs in order to launch the service or bind to it.
# If a caller of startService(), bindService(), or stopService() isn't granted this permission,
# the method doesn't work and the Intent object isn't delivered to the service.
# If this attribute isn't set, the permission set by the <application> element's permission
# attribute applies to the service. If neither attribute is set, the service isn't protected
# by a permission. 
permission = "android.permission.BIND_JOB_SERVICE"

# The name of the process where the service runs. Normally, all components of an application
# run in the default process created for the application. It has the same name as the
# application package. The <application> element's process attribute can set a different
# default for all components. But a component can override the default with its own process
# attribute, letting you spread your application across multiple processes.
# If the name assigned to this attribute begins with a colon (:), a new process, private to
# the application, is created when it's needed and the service runs in that process.
# If the process name begins with a lowercase character, the service runs in a global process
# of that name, provided that it has permission to do so. This lets components in different
# applications share a process, reducing resource usage. 
process = ":my_service"

# See https://developer.android.com/guide/topics/manifest/meta-data-element
[[package.metadata.android.application.service.meta_data]]
name = "android.accessibilityservice"
# References a XML resource.
resource = "@xml/accessibilityservice"

# Set up reverse port forwarding through `adb reverse`, meaning that if the
# Android device connects to `localhost` on port `1338` it will be routed to
# the host on port `1338` instead. Source and destination ports can differ,
# see the `adb` help page for possible configurations.
[package.metadata.android.reverse_port_forward]
"tcp:1338" = "tcp:1338"
```

If a manifest attribute is not supported by `cargo apk2`, feel free to create a PR that adds the missing attribute.

## Environmental variables provided by this tool

`cargo-apk2` sets environmental variables listed below to be used in `build.rs` or some custom script.

- `CARGO_APK2_APK_NAME`：File name of the currently generating APK.
- `CARGO_APK2_PACKAGE`：Package name.
- `CARGO_APK2_ASSETS_DIR`：Absolute path of the assets directory.
- `CARGO_APK2_RESOURCES_DIR`：Absolute path of the resources directory.
- `CARGO_APK2_CLASSES_DIR`：Directory storing compiled Java classes.
- `CARGO_APK2_RUNTIME_LIBS_DIR`：Directory storing runtime (dynamic) libraries.
- `CARGO_APK2_JAVA_HOME`：Root directory of the Java environment.
- `CARGO_APK2_SDK_HOME`：Root directory of the Android SDK.
- `CARGO_APK2_ANDROID_JAR`：Path of `android.jar` corresponding to the current `targetSdkVersion`.
- `CARGO_APK2_PLATFORM_DIR`：Path of `platforms/android-<api>` corresponding to the current `targetSdkVersion`.
- `CARGO_APK2_BUILD_TOOLS_VERSION`：Version of the Android Build Tools currently being used.
- `CARGO_APK2_MIN_SDK_VERSION`：`minSdkVersion`.
- `CARGO_APK2_TARGET_SDK_VERSION`：`targetSdkVersion`.
- `CARGO_APK2_ARTIFACT`：Output path of the main dynamic library.

These variables can be used in circumstances like automation scripting, resource processing, and Java integration.
