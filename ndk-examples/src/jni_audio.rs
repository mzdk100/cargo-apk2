use {
    android_logger::{Config, init_once},
    jni::{
        ScopeToken, jni_sig, jni_str,
        objects::{JIntArray, JObject, JObjectArray, JString},
    },
    log::LevelFilter,
    winit::platform::android::activity::AndroidApp,
};

#[unsafe(no_mangle)]
fn android_main(_app: AndroidApp) {
    init_once(Config::default().with_max_level(LevelFilter::Info));
    enumerate_audio_devices().unwrap();
}

const GET_DEVICES_OUTPUTS: jni::sys::jint = 2;

fn enumerate_audio_devices() -> Result<(), Box<dyn std::error::Error>> {
    // Create a VM for executing Java calls
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
    let mut scope = ScopeToken::default();
    let mut env = unsafe { vm.attach_current_thread_guard(Default::default, &mut scope)? };
    let env = env.borrow_env_mut();
    let context = unsafe { JObject::from_raw(env, ctx.context().cast()) };

    // Query the global Audio Service
    let class_ctx = env.find_class(jni_str!("android/content/Context"))?;
    let audio_service = env.get_static_field(
        class_ctx,
        jni_str!("AUDIO_SERVICE"),
        jni_sig!(java.lang.String),
    )?;

    let audio_manager = env
        .call_method(
            context,
            jni_str!("getSystemService"),
            jni_sig!((java.lang.String) -> java.lang.Object),
            &[(&audio_service).into()],
        )?
        .l()?;

    // Enumerate output devices
    // 枚举输出设备
    let devices = env.call_method(
        audio_manager,
        jni_str!("getDevices"),
        jni_sig!((jint) -> [android.media.AudioDeviceInfo]),
        &[GET_DEVICES_OUTPUTS.into()],
    )?;

    println!("-- Output Audio Devices --");

    let device_array = unsafe { JObjectArray::<JObject>::from_raw(env, *devices.l()?) };
    let len = device_array.len(env)? as usize;
    for i in 0..len {
        let device = device_array.get_element(env, i)?;

        // Collect device information
        // See https://developer.android.com/reference/android/media/AudioDeviceInfo
        let product_name = {
            let name = env.call_method(
                &device,
                jni_str!("getProductName"),
                jni_sig!(() -> java.lang.CharSequence),
                &[],
            )?;
            let name = env
                .call_method(
                    name.l()?,
                    jni_str!("toString"),
                    jni_sig!(() -> java.lang.String),
                    &[],
                )?
                .l()?;
            unsafe { JString::from_raw(env, *name) }.to_string()
        };
        let id = env
            .call_method(&device, jni_str!("getId"), jni_sig!(() -> jint), &[])?
            .i()?;
        let ty = env
            .call_method(&device, jni_str!("getType"), jni_sig!(() -> jint), &[])?
            .i()?;

        let sample_rates = {
            let sample_array = unsafe {
                let arr = env.call_method(
                    &device,
                    jni_str!("getSampleRates"),
                    jni_sig!(() -> [jint]),
                    &[],
                )?;
                JIntArray::from_raw(env, *arr.l()?)
            };
            let len = sample_array.len(env)?;

            let mut sample_rates = vec![0; len as usize];
            sample_array.get_region(env, 0, &mut sample_rates)?;
            sample_rates
        };

        println!("Device {}: Id {}, Type {}", product_name, id, ty);
        println!("sample rates: {:#?}", sample_rates);
    }

    Ok(())
}
