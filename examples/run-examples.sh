#!/bin/bash
set -euo pipefail

# These examples run struct-ffi-gen, consuming the IDL files inside examples/input and generating
# output inside examples/output.
#
# For more info on how to use struct-ffi-gen, see the built-in help:
# struct-ffi-gen --help

cd "$(dirname "$0")/.."

# Proto input, proto output.
# This is more or less a passthrough to `protoc`, the protobuf compiler.
#
# Notes:
# - The `js` output has a custom output folder specified: "proto_javascript".
# - Some languages like csharp and js do not produce folder hierarchies in the output.
cargo run -- \
  --input examples/input/proto \
  --output-root examples/output/proto \
  --proto cpp csharp java js=proto_javascript
