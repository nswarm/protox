#!/bin/bash
set -euo pipefail

# These examples run protox, consuming the IDL files inside `examples/input` and generating
# output inside `examples/output`.
#
# For more info on how to use protox, see the built-in help:
# protox --help

cd "$(dirname "$0")/.."
rm -rf examples/output

# Example run using each type of output (proto, template, scripted).
#
# --proto
# Proto input, proto output.
# This is more or less a passthrough to `protoc`, the protobuf compiler.
#
# --template
# Proto input, templated "server" and "client" output, as per readme.
#
# --scripted
# This uses the scripted renderer to generate flatbuffers IDL from protobuf inputs.
# The "flatbuffers.proto" contains examples of every supported fbs option.
#
# Notes:
# - The argument is: --proto INPUT OUTPUT.
# - Some languages like csharp and js do not produce folder hierarchies in the output.
# - See protox --help for all supported languages.
cargo run -- \
  --input examples/input/proto \
  --includes "$(pwd)/proto_options/protos" \
  --output-root examples/output \
  --proto cpp proto-cpp \
  --proto csharp proto-csharp \
  --proto java proto-java \
  --proto rust proto-rust \
  --proto js proto-js \
  --template-root examples/input/templates \
  --template rust-server rust-server \
  --script-root examples/input/scripts \
  --script flatbuffers flatbuffers

