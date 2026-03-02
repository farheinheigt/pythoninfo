# pythoninfo

Python environment diagnostics and package inspection from a small autonomous Rust CLI.

## Entrypoints

- User command: `bin/pythoninfo`
- Zsh completion: `bin/_pythoninfo.completion.zsh`
- Rust source: `src/main.rs`

## Usage

Run the command directly: `bin/pythoninfo`.
Generate completion script: `bin/pythoninfo --completion zsh`.

## Examples

`bin/pythoninfo`
`bin/pythoninfo requests`

## Requirements

- Runtime wrapper: `zsh`
- Build tool: `cargo`

## Notes

- The user-facing entrypoint remains `bin/pythoninfo`.
- The Rust source lives under `src/`.
- Package inspection still uses the active Python interpreter for `pip show` and metadata inspection.
- The Cargo build output stays in the repository-local `target/` directory.
