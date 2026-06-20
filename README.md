# TinyLang

TinyLang is a small template language. You hand it a string with some holes
punched in it, you hand it the values to fill those holes with, and it hands
you back a finished string. Under the hood it's a tree-walk interpreter built
on [pest](https://pest.rs/), which is to say: pest turns your template into a
parse tree, and TinyLang walks that tree and figures out what each node should
become.

If you have ever used Jinja, Liquid, or Handlebars, you already know the shape
of the thing. There is text, and inside the text there are two kinds of holes:

```html
{{ user.name }} bought {{ count }} apples.
{% if count > 10 %}That is a lot of apples.{% end %}
```

`{{ ... }}` is a *print*: evaluate the expression inside and drop the result
into the output. `{% ... %}` is *dynamic*: it does something, like deciding
whether a chunk of the template shows up at all, but it does not print itself.
Everything outside of both is just text, copied through verbatim.

## The one rule that surprises people

You cannot assign to variables from inside a template. There is no `{% a = 2 %}`.

This is on purpose. A template's job is to render state, not to invent it. So
all of your state gets set up on the Rust side and passed in. The template can
read it, branch on it, iterate over it, call functions with it, but it cannot
reach back and change it.

There is exactly one exception, and it is the `for` loop, which has to bind a
loop variable for you or it would be useless:

```html
{% for apple in apples %}{{ apple }} {% end %}
```

Here `apple` exists only inside the loop body. That is the single place where a
template gets to name a new thing.

## Using it as a library

There is one function you care about: `eval`.

```rust
use std::collections::HashMap;
use tinylang::eval;

let output = eval("{{ 1 + 1 }}", HashMap::default()).unwrap();
assert_eq!(output, "2");
```

The signature is:

```rust
pub fn eval(input: &str, state: State) -> Result<String, TinyLangError>
```

`input` is your template. `state` is everything the template is allowed to see.
You get back either the rendered string or a `TinyLangError` explaining what
went wrong, with the error messages written for a human to read (for example,
`variable 'foo' is not defined`).

### State is just a map

`State` is a type alias, and it is exactly what you would guess:

```rust
pub type State = HashMap<String, TinyLangType>;
```

`TinyLangType` is the enum of everything a value can be inside the language.
You rarely build one by hand, because `From` is implemented for the common Rust
types, so `.into()` does the work:

```rust
use std::collections::HashMap;
use tinylang::eval;

let state = HashMap::from([
    ("name".into(), "Amos".into()),       // &str  -> String
    ("posts".into(), 42.into()),          // i32   -> Numeric
    ("is_admin".into(), true.into()),     // bool  -> Bool
]);

let output = eval("{{ name }} has {{ posts }} posts", state).unwrap();
assert_eq!(output, "Amos has 42 posts");
```

There are `From` impls for `&str`, `String`, `i32`, `f32`, `f64`, `bool`,
`Vec<TinyLangType>`, and `State`, so most values convert with a single
`.into()`.

## The types

```rust
pub enum TinyLangType {
    String(String),
    Numeric(f64),
    Bool(bool),
    Function(Function),
    Vec(Vec<TinyLangType>),
    Object(State),
    Nil,
}
```

A few of these have sharp edges worth knowing about.

**Numeric is always an `f64`.** There is no separate integer type. `3` and
`3.0` are the same value internally. They print as `3`, because the formatter
does not tack on a pointless `.0`, but do not go looking for integer semantics
that are not there.

**Functions are Rust functions, not template functions.** You cannot define a
function inside a template. You define it in Rust and put it in the state, and
then the template can call it:

```rust
use std::collections::HashMap;
use tinylang::eval;
use tinylang::types::{FuncArguments, State, TinyLangType};

let state = HashMap::from([(
    "shout".into(),
    TinyLangType::Function(|args: FuncArguments, _state: &State| {
        args.into_iter().next().unwrap().to_string().to_uppercase().into()
    }),
)]);

let output = eval("{{ shout('hi') }}", state).unwrap();
assert_eq!(output, "HI");
```

A function has the signature
`fn(Vec<TinyLangType>, &HashMap<String, TinyLangType>) -> TinyLangType`. Every
argument arrives in that `Vec`, in order. Every function must return something,
so if yours has nothing meaningful to give back, return `TinyLangType::Nil`.

The second argument is the whole state. That is more powerful than it looks: a
function that receives the state can call `eval` again, which is how you would
build something like `render_partial` without the language needing to know
about partials at all.

**Vectors and Objects cannot be built inside a template.** This follows from
the no-assignment rule. There is no list literal and no map literal in the
syntax. You construct them in Rust and pass them in. A `Vec` is what you
iterate with `for`. An `Object` is a `State` (a map) whose keys you reach with
the dot operator:

```rust
use std::collections::HashMap;
use tinylang::eval;

let user = HashMap::from([("name".into(), "Amos".into())]);
let state = HashMap::from([("user".into(), user.into())]);

let output = eval("{{ user.name }}", state).unwrap();
assert_eq!(output, "Amos");
```

You can nest the dots as deep as your objects go: `a.b.c.d`.

## Expressions

Inside both `{{ ... }}` and `{% ... %}` you write expressions. The pieces are:

| Thing | Looks like |
| ----- | ---------- |
| Numbers | `3`, `3.14` |
| Strings | `'single quoted'` |
| Booleans | `true`, `false` |
| Nil | `Nil` |
| Variables | `myvar` (starts with a letter, then letters, digits, or `_`) |
| Property access | `object.property`, `a.b.c` |
| Function calls | `my_function('abc', 2)` |

And you can combine them with operators:

| Kind | Operators |
| ---- | --------- |
| Arithmetic | `+`, `-`, `*`, `/` |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` |
| Logical | `and`, `or` |
| Unary | `-` (negation) |

Arithmetic only works on numbers, and comparisons only compare like with like.
Try to add a string to a boolean and you get a runtime error rather than a
guess. The language would rather stop than make something up.

## Control flow

There are two control structures, and both are closed with `{% end %}`.

The conditional:

```html
{% if logged_in %}Welcome back.{% else %}Please sign in.{% end %}
```

The `{% else %}` is optional. The condition has to actually evaluate to a bool;
TinyLang does not do truthiness, so `{% if 1 %}` is an error, not a yes.

The loop:

```html
<ul>
{% for item in items %}  <li>{{ item }}</li>
{% end %}
</ul>
```

`items` has to be a `Vec`, and `item` is bound fresh on each pass and forgotten
when the loop ends. You can nest loops, and you can put an `if` inside a `for`
or a `for` inside an `if`, and they behave the way you would hope.

## Errors

Everything that can go wrong comes back as a `TinyLangError`, which splits into
two families. `ParseError` is for templates that are malformed before anything
runs, like an `{% if %}` with no `{% end %}`, or an `{% else %}` with no `if`
to attach to. `RuntimeError` is for things that only blow up while rendering: a
variable that was never defined, a `for` over something that is not a vector, a
`.` on something that is not an object, an operator applied to types it does not
understand. The `Display` text on each is written to be read by a person, so in
most cases you can surface it directly.

## WebAssembly

There is a `webassembly` cargo feature that compiles TinyLang to wasm and
exposes an `eval_wasm` function for calling from JavaScript. It takes the
template string and a serialized description of the state. This is what powers
the live playground, where you can try the language without touching Rust at
all:

https://tinylang.elias.tools

## More reading

There is a write-up on the parser and the PEG approach behind it:

https://www.elias.sh/posts/peg_and_rust.html

The `tests/` directory is also worth a look. `tests/examples.rs` is written to
double as documentation, and `tests/regressions.rs` is a tour of the awkward
edge cases, like an `else` inside a disabled `if`, that the implementation has
had to get right.

## Acknowledgements

Thank you @Karreiro for the pair-programming on Twitch.

## License

MIT.
