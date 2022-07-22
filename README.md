# var-8

A library to iterate over variable length code points and interpret joined code points.

## Disclaimer

Note that the create by all means does no implement the complete utf-8 standard, nor is it optimized in any meaningful way. This is to all extents and purposes to be considered as a research project.

## Example

Let's say you had the brilliant idea to write a language, where the `"🏳️‍🌈"` symbol represented `true`. If you then tried to parse the input in the following way, you will find out that this doesn't work:

```rust
const TRUE: &str = "🏳️‍🌈";

let chars = "var=🏳️‍🌈".chars().collect::<Vec<_>>();
let is_true = chars[4].to_string() == TRUE; // `false`
```

This is because the `🏳️‍🌈` symbol is actually not one utf-8 code-point, but four! The content of `chars` would be:

```rust
// chars = ['v', 'a', 'r', '=', '🏳', '\u{fe0f}', '\u{200d}', '🌈']
```

That's: A white having flag, a variation selector, a zero-width joiner, and a rainbow.

If now the `utf8_chars` equivalent is used, the code would work:


```rust
const TRUE: &str = "🏳️‍🌈";

let chars = "var=🏳️‍🌈".utf8_chars().collect::<Vec<_>>();
let is_true = chars[4].is(TRUE); // `true`
```

The chars vector now looks like this:

```rust
// chars = ['v', 'a', 'r', '=', '🏳️‍🌈']
```