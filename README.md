# xpl

xpl (Extensible Programming Language) is a lightweight, XML-based language for writing and extending programs, featuring a dedicated VM and library support via `include`.

Programs can include libraries (e.g. `math.xpl`) with `<program include="math.xpl" ...>`.

## Quick Start

```sh
git clone https://github.com/omdxp/xpl.git
cd xpl
cargo install --path .
xpl examples/hello.xpl
```

## VSCode Extension

A Visual Studio Code extension for xpl syntax highlighting and language features is available under the `vscode/` folder.

To install locally:

```sh
cd vscode
npm install
npm run compile
enable the extension in VS Code (Debug: Launch Extension)
```

See `vscode/README.md` for full documentation.

## Examples

- math.xpl: library of arithmetic functions
- hello.xpl: "Hello, World!" script using math.xpl
- test.xpl: simple print script
- conditional.xpl: if/else demo with math.xpl

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
