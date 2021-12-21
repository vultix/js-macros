//! JS_MACRO: derive(SayHello) attributes(hello_message)

declare const MACRO_INPUT: string;
declare let MACRO_OUTPUT: string;

const type = /(?:struct|enum) (.*?)\s/.exec(MACRO_INPUT)[1];
const message = /#\[hello_message = "(.*)"]/.exec(MACRO_INPUT)?.[1] ?? "Default hello world message";

MACRO_OUTPUT = `
impl SayHello for ${type} {
	fn say_hello(){
		println!("${message}");
	}
}
`;
