# Flatbuffers Generator
This generator creates [flatbuffers](https://google.github.io/flatbuffers/) IDL files. For the most part it will "just work" (excluding the [Missing Features](#missing-features)), but if you want to fine-tune your output to be more idiomatic, or use features in flatbuffers that don't exist in proto, the following options may help.

See [Writing a (Flatbuffers) Schema](https://google.github.io/flatbuffers/flatbuffers_guide_writing_schema.html) for more information.

## File Options

**File Attributes**

(Repeatable)

```
// proto
option (fbs.file_attribute) = "my_attr";

// output
attribute "my_attr";
```

**File Identifier**

```
// proto
option (fbs.file_identifier) = "ABCD";

// output
file_identifier "ABCD";
```

**File Extension**

```
// proto
option (fbs.file_extension) = "data";

// output
file_extension "data";
```

**Root Type**

```
// proto
option (fbs.root_type) = "SomeTable";

// output
root_type SomeTable;
```

## Enum Options

**Enum Type**

Protobuf doesn't support different underlying types for enums like fbs does. Use this option to specify what you need. Default is `byte`.

```
// proto
enum SomeEnum {
    option (fbs.enum_type) = UINT;
}

// output
enum SomeEnum : uint {...}
                ^^^^
```

## Message Options

Protobuf has only one message type. Use this option to specify between the default `table` and `struct`.

```
// proto
message SomeMsg {
    option (fbs.message_type) = STRUCT;
}

// output
struct SomeMsg : uint {}
^^^^^^
```

## Field Options

Flatbuffers has more scalar types than protobuf. Use this option to specify the specific type for your scalar field.

```
// proto
int32 some_field [(fbs.field_type) = USHORT];

// output
some_field: ushort;
             ^^^^^^
```

## Field Defaults

Flatbuffers supports default values for scalar types; protobuf does not. Use these three options to specify defaults.

```
// proto
bool some_bool [(fbs.bool_default) = true];
int32 some_int [(fbs.int_default) = 1234];
float some_float [(fbs.float_default) = 12.34];

// output
some_bool: bool = true;
                ^^^^^^
some_float: float32 = 12.34;
                    ^^^^^^^
some_int: int32 = 1234;
                ^^^^^^
```

## Field Attributes

Flatbuffers supports a number of different field attributes. Use this option to specify them directly.

```
// proto
int32 some_field [(fbs.field_attribute) = "required", (fbs.field_attribute) = "priority: 1"];

// output
some_field: int32 (required, priority: 1);
                  ^^^^^^^^^^^^^^^^^^^^^^^
```

## Examples
See `examples/input/proto/flatbuffers.proto` for an example of all implemented fbs options being used.

## Missing Features

- fbs fixed length arrays in structs
- proto maps
- proto oneofs
