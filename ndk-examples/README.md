# ndk-examples

English | [中文](https://gitcode.com/mzdk100/cargo-apk2/blob/main/ndk-examples/README-ZH.md)

Collection of examples showing different parts of the libraries.

## Examples

In order to see logs of the sample apps execute in a console:
```shell
adb logcat RustStdoutStderr:D '*:S'
```

### hello_world

Prints `hello world` in the console:

```shell
cargo apk2 run --features hello_world
```

### jni_audio

Prints output audio devices in the console:

```shell
cargo apk2 run --features jni_audio
```
