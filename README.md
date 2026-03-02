# pythoninfo

## Overview
Python environment diagnostics and package inspection.

## Location
- Repository: `/Users/farheinheigt/Projets/dev/pythoninfo`
- User entrypoint: `/Users/farheinheigt/Projets/dev/pythoninfo/bin/pythoninfo`
- Completion file: `/Users/farheinheigt/Projets/dev/pythoninfo/bin/_pythoninfo.completion.zsh`

## Usage
Run the command directly: `pythoninfo`.
Generate completion script: `pythoninfo --completion zsh`.

## Examples
`pythoninfo`
`pythoninfo requests`

## Requirements
- Runtime wrapper: `zsh`
- Build tool: `cargo`

## Notes
- The user-facing entrypoint remains `bin/pythoninfo`.
- The Rust source lives under `src/`.
- Package inspection still uses the active Python interpreter for `pip show` and metadata inspection.
- The Cargo build output stays in the repository-local `target/` directory.
