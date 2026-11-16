## #1569 - Select LLVM ar for macOS wasm builds

**Chosen:** When the target-specific archiver variable is unset on macOS,
look for `llvm-ar` on PATH, then in Homebrew's LLVM prefix, then in the
active Rust toolchain's `llvm-tools` directory. Export the target-specific
variable before downloading or compiling. Preserve explicit values,
including an explicitly empty value. If discovery fails, name `llvm-ar`
and both installation choices in an early refusal.

**Why:** Apple's archive tools cannot index the wasm objects used by flecs.
Target-specific selection keeps native archiving unchanged. Dynamic prefixes
work with Intel, Apple silicon, and custom Homebrew installations.

**Dropped:** A hardcoded Homebrew prefix misses other installations. Setting
global `AR` could change native builds. Waiting for the linker to fail gives
the user undefined-symbol errors rather than the missing-tool diagnosis.

**Cost to change later:** The discovery block is isolated in the wasm build
script. Its seven hermetic regression cases run with `/bin/bash` and fake
toolchains, without downloads or compilation.
