# Coding Style Guide

## General Coding Guidelines
- Avoid using the global Namespace
- Avoid unnecessary complex constructs. Everybody should be able to understand the code.
- Be mindful of optimizations. Especially in limiting 

[//]: <> (TODO: No passwords in code, never!)

## Limitations of this Guide
It is important to note that we only discuss guidelines here. If your code is significantly improved by breaking a rule, 
this is recommended.

## Rust coding Guide
- Use Clippy instead of Cargo check for code analysis

- Use BigInt for generic Integer values.
    Rather than
    ```
    {
        ...
        let my_values = [i32;3];
        ...
    }
    ```
    use 
    ```
    {
        ... 
        let my_values = [BigInt;3];
        ...
    }
    ```
- Always return Results.

    Rather than
    ```
    fn div(a: BigInt, b: BigInt) -> BigInt {
        if b.is_zero() {
            panic!("Dividing by zero!");
        }
        ...
    }
    ```
    use 
    ```
    fn div(a: BigInt, b: BigInt) -> Result<BigInt,Box<dyn Error>> {
        if b.is_zero() {
            Err("Dividing by zero!")?;
        }
        ...
    }
    ```
- Handle Results and avoid using unwrap.

    Rather than
    ```
    {
        ...
        let d = div(3,5).unwrap();
        ...
    }
    ```
    use 
    ```
    {
        ...
        let d = match div(3,5) {
             Ok(x) => x,
             Err(_) => 0    //defines a/0 := 0
        }
        ...
    }
    ```
- Avoid using unsafe code wherever possible.
- Specify crate versions (or git commits) in the dependencies. This helps to reduce the risk of dependency mismatches

    Rather than
    ```
    {
        ...
        num-bigint = {default-features = false, features = ["serde"]}
        ...
    }
    ```
    use 
    ```
    {
        ...
        num-bigint = { version = "0.4",default-features = false, features = ["serde"]}
        ...
    }
- Frequently update dependencies to take advantage of the newest features and bug fixes.
  

## Circom coding Guide
- Comment on every section of the code for its purpose.
- Number public signals in the ordern in which they are provided by the witness
- Comment all non-trivial assumptions over constraints.default-features

    Rather than
    ```
    {
        ...
        signal input a;
        signal d <-- a /2;
        signal r <-- a % 2;
        a === 2*d+r;   
        r === 1;
        ...
    }
    ```
    use 
    ```
    {
        ...
        signal input a;
        signal d <-- a /2;
        signal r <-- a % 2;
        a === 2*d+r;  //Verifies that a mod 2 = r
        r === 1;      //Verifies that a is odd
        ...
    }
    ```
