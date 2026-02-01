use {
    android_logger::{Config, init_once},
    jni::objects::{JIntArray, JObject, JObjectArray},
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
    // 创建用于执行 Java 调用的 VM
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let mut env = vm.attach_current_thread()?;

    // Query the global Audio Service
    // 查询全局音频服务
    let class_ctxt = env.find_class("android/content/Context")?;
    let audio_service = env.get_static_field(class_ctxt, "AUDIO_SERVICE", "Ljava/lang/String;")?;

    let audio_manager = env
        .call_method(
            context,
            "getSystemService",
            // JNI type signature needs to be derived from the Java API
            // (ArgTys)ResultTy
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[(&audio_service).into()],
        )?
        .l()?;

    // Enumerate output devices
    // 枚举输出设备
    let devices = env.call_method(
        audio_manager,
        "getDevices",
        "(I)[Landroid/media/AudioDeviceInfo;",
        &[GET_DEVICES_OUTPUTS.into()],
    )?;

    println!("-- Output Audio Devices --");

    let device_array = unsafe { JObjectArray::from_raw(devices.l()?.into_raw()) };
    let len = env.get_array_length(&device_array)?;
    for i in 0..len {
        let device = env.get_object_array_element(&device_array, i)?;

        // Collect device information
        // 收集设备信息
        // See https://developer.android.com/reference/android/media/AudioDeviceInfo
        let product_name: String = {
            let name =
                env.call_method(&device, "getProductName", "()Ljava/lang/CharSequence;", &[])?;
            let name = env.call_method(name.l()?, "toString", "()Ljava/lang/String;", &[])?;
            env.get_string((&name.l()?).into())?.into()
        };
        let id = env.call_method(&device, "getId", "()I", &[])?.i()?;
        let ty = env.call_method(&device, "getType", "()I", &[])?.i()?;

        let sample_rates = {
            let sample_array = unsafe {
                JIntArray::from_raw(
                    env.call_method(&device, "getSampleRates", "()[I", &[])?
                        .l()?
                        .into_raw(),
                )
            };
            let len = env.get_array_length(&sample_array)?;

            let mut sample_rates = vec![0; len as usize];
            env.get_int_array_region(sample_array, 0, &mut sample_rates)?;
            sample_rates
        };

        println!("Device {}: Id {}, Type {}", product_name, id, ty);
        println!("sample rates: {:#?}", sample_rates);
    }

    Ok(())
}
