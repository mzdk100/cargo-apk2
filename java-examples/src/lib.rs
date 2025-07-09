use jni::{
    JNIEnv,
    objects::{JClass, JString},
};

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_app_MainActivity_getMessageFromRust<'a>(
    env: JNIEnv<'a>,
    _class: JClass<'a>,
) -> JString<'a> {
    // 创建一个Rust字符串
    let output = "Hello from Rust!";

    // 将Rust字符串转换为Java字符串并返回
    env.new_string(output)
        .expect("Couldn't create Java string!")
}
