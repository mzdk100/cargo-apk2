#!/bin/bash

set -ex

# Make sure the package is removed since it may end up in the AVD cache. This causes
# INSTALL_FAILED_UPDATE_INCOMPATIBLE errors when the debug keystore is regenerated,
# as it is not stored/cached on the CI:
# https://github.com/rust-windowing/android-ndk-rs/blob/240389f1e281f582b84a8049e2afaa8677d901c2/ndk-build/src/ndk.rs#L308-L332
adb uninstall rust.ndk_examples || true

if [ -z "$1" ];
then
    cargo apk2 run -p ndk-examples --target x86_64-linux-android --features hello_world --no-logcat
else
    adb install -r "$1/ndk-examples.apk"
    adb shell am start -a android.intent.action.MAIN -n "rust.ndk_examples/android.app.NativeActivity"
fi

# Wait for the app to start and produce output
sleep 30

# Dump logcat with broader filter to capture both `rust` tag (from android_logger)
# and `hello_world` tag, plus System.out for println! output
adb logcat -d | tee ~/logcat.log

echo "--- Searching for 'hello world' in logcat ---"
if grep -i 'hello world' ~/logcat.log;
then
    echo "App running"
else
    echo "::error::App not running"
    # Show recent logcat entries for debugging
    echo "--- Recent logcat entries ---"
    adb logcat -d -t 50
    exit 1
fi

ERROR_MSG=$(grep -e 'thread.*panicked at' "$HOME"/logcat.log | true)
if [ -z "${ERROR_MSG}" ];
then
    exit 0
else
    echo "::error::${ERROR_MSG}"
    exit 1
fi
