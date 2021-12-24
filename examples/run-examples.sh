#!/bin/bash
set -euo pipefail

# These examples run protoffi, consuming the IDL files inside `examples/input` and generating
# output inside `examples/output`.
#
# For more info on how to use protoffi, see the built-in help:
# protoffi --help

cd "$(dirname "$0")/.."
rm -rf examples/output

# Proto input, proto output.
# This is more or less a passthrough to `protoc`, the protobuf compiler.
#
# Notes:
# - The argument is: --proto INPUT OUTPUT.
# - Some languages like csharp and js do not produce folder hierarchies in the output.
# - See protoffi --help for all supported languages.
cargo run -- \
  --input examples/input/proto \
  --output-root examples/output/proto \
  --proto cpp proto_cpp \
  --proto csharp proto_csharp \
  --proto java proto_java \
  --proto rust proto_rust \
  --proto js proto_js
