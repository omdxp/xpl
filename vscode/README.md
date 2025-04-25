# xpl VSCode Extension

This extension provides:

- Syntax highlighting for `.xpl` files
- Language Server Protocol (LSP) client setup to connect to an xpl language server

## Features

- Tag and string highlighting via TextMate grammar
- Comments support (`//` and `<!-- -->`)
- Bracket matching for `< >`, `{ }`, and `[ ]`
- LSP client to enable diagnostics, completions, and more (requires a separate xpl language server)

## Getting Started

1. Install dependencies:
   ```sh
   cd vscode
   npm install
   ```

2. Build the extension:
   ```sh
   npm run compile
   ```

3. Open in VSCode:
   - Press `F5` to launch a new Extension Development Host

4. (Optional) Language Server Setup:
   - Place or build your xpl language server binary or script under `vscode/server/`
   - Update `extension.ts` to point to the server entry point

## Project Layout

```
vscode/
├── package.json         # Extension manifest
├── tsconfig.json        # TypeScript config
├── extension.ts         # LSP client activation code
├── syntaxes/
│   └── xpl.tmLanguage.json  # TextMate grammar
└── language-configuration.json  # Comment/bracket rules
```

## Publishing

- Update `package.json` publisher and version
- Run `vsce package` to create a `.vsix` for distribution
