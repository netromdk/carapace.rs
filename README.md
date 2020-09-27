[![Build and Test](https://github.com/netromdk/carapace.rs/workflows/Build%20and%20Test/badge.svg)](https://github.com/netromdk/carapace.rs/actions)
[![Security Audit](https://github.com/netromdk/carapace.rs/workflows/Security%20audit/badge.svg)](https://github.com/netromdk/carapace.rs/actions)

# Carapace
Shell written in Rust

## Builtins
- `cd` (`pushd`) - Change directory and push to directory stack
- `popd` - Pop head directory from stack and set it as current directory
- `dirs` - Display stack of directories
- `export` - List or export new environment variables
- `unset` - Unset environment variables
- `set` - Set and unset shell options
- `hash` - Check command existence or rehash
- `rehash` - Rehash all executable programs in `$PATH`
- `history` (`hist`, `h`) - List historical commands
- `exit` - Exit with specific code or default `0`
- `quit` - Exit with code `0`
