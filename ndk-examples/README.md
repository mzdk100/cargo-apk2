# ndk-示例

展示库不同部分的示例集合。

## 示例

为了查看示例应用程序在控制台中执行的日志：
```shell
adb logcat RustStdoutStderr:D '*:S'
```

### hello_world

在控制台中打印“hello world”

```shell
cargo apk2 run --features hello_world
```

### jni_audio

在控制台中打印输出音频设备

```shell
cargo apk2 run --features jni_audio
```
