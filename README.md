# js-macros
Quickly prototype procedural macros using JavaScript or TypeScript!

Have you ever thought "this would be a great use case for a procedural macro," 
but didn't want to go through the effort? This crate is the perfect fix for you!

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
