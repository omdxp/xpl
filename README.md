# xpl

xpl (Extensible Programming Language) is a lightweight, XML-based language for writing and extending programs, featuring a dedicated VM and library support via `include`.

Programs can include libraries (e.g. `math.xpl`) with `<program include="math.xpl" ...>`.

## Quick Start

```sh
git clone https://github.com/yourusername/xpl.git
cd xpl
cargo install --path .
xpl examples/hello.xpl
```

## Examples

- math.xpl: library of arithmetic functions
- hello.xpl: "Hello, World!" script using math.xpl
- test.xpl: simple print script
- conditional.xpl: if/else demo with math.xpl

## License

MIT
