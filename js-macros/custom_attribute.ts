//! JS_MACRO: attribute(say_hello)

declare const MACRO_INPUT: string;
declare const MACRO_ARGUMENTS: string;
declare let MACRO_OUTPUT: string;

const message = /message = "(.*?)"/.exec(MACRO_ARGUMENTS)?.[1] ?? "Default hello message";

MACRO_OUTPUT = MACRO_INPUT.replace(/fn(.*?){/, (_, match) => `fn ${match} { println!("${message}"); `);
