#!/bin/bash
set -euo pipefail

# These examples run idlx, consuming the IDL files inside `examples/input` and generating
# output inside `examples/output`.
#
# For more info on how to use idlx, see the built-in help:
# idlx --help

cd "$(dirname "$0")/.."
rm -rf examples/output

# Proto input, proto output.
# This is more or less a passthrough to `protoc`, the protobuf compiler.
#
# Notes:
# - The argument is: --proto INPUT OUTPUT.
# - Some languages like csharp and js do not produce folder hierarchies in the output.
# - See idlx --help for all supported languages.
cargo run -- \
  --input examples/input/proto \
  --includes "$(pwd)/proto_options/protos" \
  --output-root examples/output/proto \
  --proto cpp proto-cpp \
  --proto csharp proto-csharp \
  --proto java proto-java \
  --proto rust proto-rust \
  --proto js proto-js \

# Proto input, templated "server" and "client" output, as per readme.
cargo run -- \
  --input examples/input/proto \
  --includes "$(pwd)/proto_options/protos" \
  --output-root examples/output/templates \
  --template-root examples/input/templates \
  --template rust-server rust-server

# Proto input, flatbuffers output.
#
# The "flatbuffers.proto" contains examples of every supported fbs option.
cargo run -- \
  --input examples/input/proto \
  --includes "$(pwd)/proto_options/protos" \
  --output-root examples/output \
  --template-root examples/input/templates \
  --template flatbuffers flatbuffers

