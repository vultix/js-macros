//! JS_MACRO: function(hello_world)

declare const MACRO_INPUT: string;
declare let MACRO_OUTPUT: string;

const message = /"(.*?)"/.exec(MACRO_INPUT)?.[1] ?? "Default hello message";

MACRO_OUTPUT = `println!("${message}")`;
