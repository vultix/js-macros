use js_macros::SayHello;

/// This [SayHello] macro is defined in `../js-macros/custom_derive.ts`
#[derive(SayHello)]
#[hello_message = "Hello ts macro!"]
struct Example {}

trait SayHello {
    fn say_hello();
}

fn main() {
    Example::say_hello();
}
