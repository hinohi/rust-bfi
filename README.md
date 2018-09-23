# rust-bfi

## what

Brainfuck interpreter written by Rust.
My practice for writing Rust.

## How to play

Running bf code with optimize

```sh
cargo build --release
./target/release/rust-bfi example/mandelbrot.bf.txt -O2
```

Debugging

```sh
cargo run example/mandelbrot.bf.txt -O1 -d1
```
