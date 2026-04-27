#![cfg(target_os = "android")]

#[cfg(feature = "hello_world")]
mod hello_world;
#[cfg(feature = "jni_audio")]
mod jni_audio;
#[cfg(feature = "looper")]
mod looper;

#[cfg(not(any(feature = "hello_world", feature = "jni_audio", feature = "looper")))]
compile_error!("Please enable one of the features: hello_world, jni_audio, looper");
