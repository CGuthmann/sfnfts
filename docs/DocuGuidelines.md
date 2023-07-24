# Documentation Guidelines

## General Guidelines
- Every code file should start with a short introduction to its purpose.
- Every function should be proceeded by a short description.
- Complex code constructs or algorithms should be proceeded by a short explanation or reference to a source.
- Use descriptive variable names.
- Write whole sentences and use punctuation.
- Don‘t pre-announce features.

## Documentation in Rust

### How
- don't write to complicated and long texts, use bullet points that contains all interesting and important facts 
- Functions that belong to a trait should only be commented in the trait
- The implementation of functions that belong to a trait don't get a second doc string, only normal comments
- Please insert Links to used Types, Structs and functions to that they can be found more easily in the generated 
  documentation
- Provide examples of how to use your code

### Docstring Types
Rust knows _inner_ and _outer_ docstrings. You can apply them like this:
```
/// This is an outer docstring
struct my_struct {/* ... */}
```
```
struct my_struct {
    //! This is an inner docstring
    /* ... */
}
```

Inner docstrings are used to comment on a crate or module, by adding a inner docstring at the beginning of the file.

### Markdown-like documentation 
Docstrings in allow us to wirte the docstring in a markdown dialect.
Because of this please use the following template to document your functions.
You may leave out unnecessary sections.

```
/// <A brief description goes here>
/// Link used functions and Structs like this:
///  - [`String`] for things that are in scope
///  - [`some_fn()`](crate::some_module::some_fn) if they are not in scope
///
/// # Arguments
/// * `arg_name` - description
/// * ...
/// # Errors
/// * `error_name` - error descripton (why is this error returned?)
/// * ...
/// # Panics
/// <Situations in with the function will panic>
/// # Side effects
/// <Description of the side effects of the function>
/// # Return
/// <Description of the returned value>
/// # Examples
/// ```ignore
/// // this is a code example that will show up as rust code snippet in the documentation
/// let my_val = my_func();
/// /* ... */
/// ```
fn my_func() { /* ... */ }
```

We are using the `ignore` attribute here, because otherwise the examples in the docstrings are tested whether they 
compile or not and **will be executed** (use `no_run` to suppress this). For more information on possible attributes and 
doc tests consider the official rust documentation.

## Further Reading
 - [Rust by example](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
 - [The Rustdoc Book](https://doc.rust-lang.org/rustdoc/index.html)
 - [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html)