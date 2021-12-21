use js_macros::say_hello;

/// This [say_hello] macro is defined in `../js-macros/custom_attribute.ts`
#[say_hello(message = "Hello, you called test!")]
fn test() {}

fn main() {
    test();
}
