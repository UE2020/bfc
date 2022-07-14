# `bfc`
A brainfuck engine with a JIT compiler (x86) and an optimized interpreter as a fallback.


## Compilation
`cargo build --release`

## Usage
To execute brainfuck, simply run the engine with the brainfuck file as the first argument.
```
./target/release/bf mandelbrot.bf
```
