# Flatbuffers Generator
This generator creates [flatbuffers](https://google.github.io/flatbuffers/) IDL files. For the most part it will "just work" (excluding the [Missing Features](#missing-features)), but if you want to fine-tune your output to be more idiomatic, or use features in flatbuffers that don't exist in proto, the following overlays may help.

See [Writing a (Flatbuffers) Schema](https://google.github.io/flatbuffers/flatbuffers_guide_writing_schema.html) for more information.

## File Options

**File Attributes**

```yaml
# overlay
attributes:
    - priority
    - deprecated
```

```flatbuffers
// output.fbs
attribute "priority";
attribute "deprecated";
```

**File Identifier**

```yaml
# overlay.yaml
identifier: ABCD
```

```flatbuffers
// output.fbs
file_identifier "ABCD";
```

**File Extension**

```yaml
# overlay.yaml
extension: data
```

```flatbuffers
// output.fbs
file_extension "data";
```

**Root Type**

```yaml
# overlay.yaml
root_type: SomeTable
```

```flatbuffers
// output.fbs
root_type SomeTable;
```

## Enum Options

**Enum Type**

Protobuf doesn't support different underlying types for enums like fbs does. Use this option to specify what you need. Default is `ubyte`.

```yaml
# overlay.yaml
type: uint
```

```flatbuffers
// output.fbs
enum SomeEnum : uint {}
              //^^^^
```

## Message Options

Protobuf has only one message type. Use this option to specify between the default `table` and `struct`. Default is `table`.

```yaml
# overlay.yaml
type: struct
```

```flatbuffers
// output.fbs
struct SomeMsg {}
//^^^^
```

## Field Options

Flatbuffers has more scalar types than protobuf. Use this overlay to specify the specific type for your scalar field.

```yaml
# overlay.yaml
type: ushort
```

```flatbuffers
// output.fbs
table Asdf {
    some_field: ushort;
              //^^^^^^
}
```

## Field Defaults

Flatbuffers supports default values for scalar types; protobuf does not. Use this overlay to specify a default.

```yaml
# overlay.yaml
default: 1234
```

```flatbuffers
// output.fbs
table Asdf {
    some_int: int32 = 1234;
                    //^^^^
}
```


## Field Attributes

Flatbuffers supports a number of different field attributes. Use this overlay to specify them directly.

```yaml
# overlay.yaml
attributes:
- "priority: 1"
- deprecated
```

```flatbuffers
// output.fbs
table Asdf {
    some_int: int32 = 1234 (priority: 1, deprecated);
                          //^^^^^^^^^^^  ^^^^^^^^^^
}
```

## Examples

See the example flatbuffers overlays file: [fbs_overlays.yml](examples/input/fbs_overlays.yml) for examples of using every overlay.

## Missing Features

- fbs fixed length arrays in structs
- proto maps
- proto oneofs
