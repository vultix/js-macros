# js-macros
[![Crates.io](https://img.shields.io/crates/v/js-macros.svg)](https://crates.io/crates/js-macros)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)
[![CI](https://github.com/vultix/js-macros/actions/workflows/ci.yml/badge.svg)](https://github.com/vultix/js-macros/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/d/js-macros.svg)](https://crates.io/crates/js-macros)

Quickly prototype Rust procedural macros using JavaScript or TypeScript!

Have you ever thought "this would be a great use case for a procedural macro," 
but didn't want to go through all the effort? This crate is the perfect fix for you!

```toml
[dependencies]
js-macros = "0.1"
```

## How to use
Setup is just three easy steps!

1. Create a `js-macros` folder at the root of your cargo workspace
2. Create a new `.js` or `.ts` macro file in the folder
3. Import your new macro and use it anywhere in your project!

_Helpful tip: These macros are invoked using `node`, allowing you to use tools like `require()`_

## Examples
Example macros can be found in this repository's [js-macros](https://github.com/vultix/js-macros/tree/main/js-macros) folder, 
with example usage in the [examples](https://github.com/vultix/js-macros/tree/main/examples) folder.

Here's all it takes to write a custom `derive(Copy)` macro:
```javascript
//! JS_MACRO: derive(Copy)

const type = /(?:struct|enum) (.*?)\s/.exec(MACRO_INPUT)[1];

MACRO_OUTPUT = `impl Copy for ${type} {}`;
```
Usage is as simple as `#[derive(js_macros::Copy, Clone)]`

## Debugging Macros
Any errors thrown by your js code will be captured and turned into a procedural macro error at build time.
You can take advantage of this by throwing errors to debug your js macros.

## Build Time Impact
Each macro expansion takes somewhere in the ballpark of 10ms to 40ms, which can add up quickly.

Typescript compilation is cached and should only affect initial builds.

## IDE Support
- **rust-analyzer** - works very well with js-macros, you'll just need to run the `Rust Analyzer: Restart Server` command after changing a js macro
- **IntelliJ-Rust** - More finicky, but still picks up js-macro generated items
  - First, you'll need to enable the experimental build script and procedural macro features:
    - Call Help | Find Action (`Ctrl+Shift+A` on Linux/Windows, `⌘⇧A` on macOS)
    - Search for Experimental Features
    - Enable `org.rust.cargo.evaluate.build.scripts` and `org.rust.macros.proc`
  - After making a change to a js macro, you'll want to run the "Refresh Cargo Projects" command.
    - Your old macro output still seems to be cached after this, so you'll need to change the body of the macro as well

