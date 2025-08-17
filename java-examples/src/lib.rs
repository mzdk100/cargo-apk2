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
    // Defines a Rust string literal.
    let output = "Hello from Rust!";

    // Converts the Rust string to a Java string and returns it.
    env.new_string(output)
        .expect("Couldn't create Java string!")
}
