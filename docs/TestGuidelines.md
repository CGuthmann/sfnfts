# Tests in Rust

## Introduction

Let's not write the next blog post about software testing. We all know that it is crucial, improves
quality, and will save us a lot of trouble, time, and your good mood when debugging. This page should  
briefly provide an overview of how we can write tests in Rust.

The most important rules for us are:
- **Unit tests should be written continuously together with new code**
- If workflow allows, don't write the tests for your functions yourself
- **Run the tests before merging a new feature** into the dev or main branch
- Tests should be simple (we don't want to debug tests...)
- **Don't duplicate implementation logic in your tests**

## Unit tests

Unit tests are common Rust functions that verify that the non-test code is functioning expectedly. The bodies of test 
functions typically perform these three actions:
1. Set up any needed data or state.
2. Run the code you want to test.
3. Assert that the results are what you expect.

**Unit tests are located in the same file as the code they test.** If you create your files with the
`cargo new` command, cargo will handle this. Otherwise, add a module named `tests` according to the
following template to your file:
```rust
#[cfg(test)]
mod tests {
    
    //this allows us to test private functions
    use super::*; 
    
    #[test]
    fn some_test() {
        /* ... */
    }
}
```

To check the results, Rust provides function-like macros like:
- `assert!(expression)`
- `assert_eq!(left, right)`
- `assert_ne!(left, right)`

You can run the test with the `cargo test` command. And run a specific test by executing
`cargo test --test test_name`.

Sometimes, you may want to write tests that you expect to fail. To do so, annotate the test function with
`#[should_panik]` like this:
```rust
#[test]
#[should_panic]
fn some_test() {
    /* ... */
}
```

It can also happen that individual tests take a very long time to complete, and you don't want
to run them every time together with the other tests. Such tests can be annotated with `#[ignore]`:
```rust
#[test]
#[ignore]
fn some_test() {
    /* ... */
}
```

After that, you can run ignored tests separately with `cargo test --ignored` or explicitly run all tests
by executing `cargo tests --include-ignored`.

There are way more possibilities to adjust the tests and their behavior. E.g., custom error messages. Please consider 
the Links in Section [Further Reading](#Further-Reading).

## Integration tests

While unit tests are testing one module in isolation at a time, integration tests are external to your
crate and use only its public interface like any other code would. They aim to test
that many parts of your program work correctly together.

Integration tests must be placed in a `tests` directory next to `src`. We suggest to group tests
that belong together in their own files.
```
crate_root
├── Cargo.lock
├── Cargo.toml
├── src
│   └── lib.rs
└── tests
    └── integration_test.rs
```

The test functions for integration tests are written just like unit tests. But **they are not located in a
`tests` module and we must import the code we want to test**.
```rust
// importing the module
use my_module;

#[test]
fn some_integration_test() {
    /* ... */
}
```

Integration tests are executed using the `cargo test` command after the unit test. You can run all
integration tests in one file separately with `cargo test --test file_name`.

As already mentioned, there are way more possibilities. E.g., you may group your integration test files in
submodules inside the `tests` directory. (cf. [Further Reading](#Further-Reading))

## Further Reading

- [Rust By Example: Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [The Rust Programming Language: Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
