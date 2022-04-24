[![CI](https://github.com/nswarm/protox/workflows/CI/badge.svg)](https://github.com/nswarm/protox/actions/workflows/rust.yml)

# protox

protox is an executable that generates code, type definitions, or related output based on the [Protobuf IDL](https://developers.google.com/protocol-buffers/docs/reference/proto3-spec). Output is formatted using [rhai](https://rhai.rs/book/) scripts _or_ [handlebars](https://handlebarsjs.com/) templates.

protox is built to generate output based on a set of types or APIs. protox can eliminate the need to write IDL parsing logic or a protobuf compiler plugin by letting you get straight to writing much simpler scripts or templates.

You as a user provide:
- A set of files written in a supported Google's protobuf IDL.
- A configuration file that defines the specifics of your output, e.g. naming conventions of your output.
- A set of either rhai script files or handlebars template files that define how the data should be structured when rendering.

protox produces:
- A set of files based on the proto definitions. For example equivalent type definitions across multiple languages or another IDL entirely.

## Installation

Currently protox must be cloned/downloaded from source and built with `cargo build`. Then you can run it with `cargo run -- <protox args go here>`.

## Usage

Run `protox --help` to get information on the command line usage.

Take a look at `examples/run-examples.sh` which runs protox on the input inside `examples/input` and will produce sets of output in `examples/output`.

See [Scripts](#scripts) below for getting started with the scripted renderer.

See [Templates](#templates) below for getting started with the template renderer.

### Should I use the Template or Scripted renderer?

For simpler tasks, the template renderer may be preferred because it lets you visualize the output of the file inline with the variables.

In practice the scripted renderer is simple as well, as you will mostly only use simple constructs like variables and loops, so it's not like you need to learn an entirely new language. If you are looking to do more complex generation like actual code gen, the scripted renderer gives more power out of the box with built-in language features like string and array manipulation.

### Built-in Support

protox only supports protobuf as the input IDL.

**Script**
- Flatbuffers IDL

**Template**
- (None)

See the `builtin` folder for more information on each built-in script and template support.

**Protobuf Generated Code**
- All [supported protobuf languages](https://developers.google.com/protocol-buffers) via the protobuf compiler itself (protoc)
- Rust via [prost](https://github.com/tokio-rs/prost)

See the `examples/run-examples.sh` script for various ways of using protox.

## Proto Options

### Built-in

`file_key_value` `enum_key_value` `msg_key_value` `field_key_value`

These options exist as a seam for you to specify anything you need from your proto files without editing protox. If you want to use your own custom proto options, see [below](#using-your-own).

**Example**
```
// proto
option (protox.file_key_value) = "my_key=my_value";"

// script
output.append(`My value is: ${file.options.my_key}!`);

// template
My value is {{file.options.my_key}}!

// output
My value is my_value!
            ^^^^^^^^
```

These are repeatable.

`native_type`

This defines a string to _completely replace_ the usage of a type in the output.

**Example**
```
// proto
string special_string [(protox.native_type) = "Special"]
string normal_string;

// script (also works in templates)
output.append(`${field.relative_type} ${field.name};`)

// output
Special special_string;
String normal_string;
```

### Using your Own

You can add support for custom proto options, but you'll need to build from source. The following steps assuming a working directory of `proto_options`.

This only works for the Scripted Renderer.

Note that the `user` functions mentioned below are meant for you to use so that protox updates do not cause merge conflicts.

#### Setup

- Create a proto file with a custom package name in `protos`.
- Add the generated file as an include at the top of `src/lib.rs`.
- Create an API file in `src`.
  - This file should have a [rhai plugin](https://rhai.rs/book/plugins/module.html).
- Register the API's rhai plugin module in `src/lib.rs::register_user_script_apis`.

#### Adding Custom Options

- Add new option definitions to your proto file.
- Register the extension in `src/lib.rs::register_user_extensions`.
  - e.g. `registry.register(extensions::my_proto_options::MY_PROTO_OPTION);`
- Add a script accessor to your api file that reads the extension data from the appropriate

```rust
// Take special note of:
// - The rhai_fn definition defines what is used to access the value _in script_.
// - The fn name is irrelevant.
// - Ensure the *Options type is correct!
#[rhai_fn(get = "my_proto_option", pure)]
pub fn file_my_proto_option(opt: &mut FileOptions) -> String {
    opt.extension_data(extensions::my_proto_options::MY_PROTO_OPTION)
        .map(&String::clone)
        .unwrap_or(String::new())
}
```

#### Examples

Follow how the flatbuffers/fbs protos options are registered.

##### Example
The flatbuffers scripted output has a number of custom options similar to how you'll need to declare yours.

## Scripts

Scripted output is rendered using [rhai](https://rhai.rs/book/). In practice, you likely won't use much beyond the basic syntax, but it's there if you need it.

### Setup

protox requires only a couple files. `main.rhai` is the root of all scripts. `config.json` is how you configure the data available in the context when rendering.

**Note:** You can quickly initialize a directory with default files using `protox --init`.

Required:
- config.json
- main.rhai

Optional:
- metadata.hbs
- As many other `.rhai` scripts as you need

`main.rhai` must have the function `fn render_file(file, output)` which is the primary entrypoint that will be called for each output file.

### Configuration

`config.json` defines how the protox renderer contexts are filled with data. The best source for information on what each field does is the [renderer_config.rs](https://github.com/nswarm/protox/blob/main/generator/src/renderer/renderer_config.rs).

### Data Context

The data available when rendering is defined in context objects passed to the scripted renderer, and accessible directly as members on the associated object. For example `file.source_file` on a FileContext would give you the `source_file` member.

All context data structures can be seen [here](https://github.com/nswarm/protox/tree/main/generator/src/renderer/context). `file.rs` and `metadata.rs` map to the `render_file` and `render_metadata` script entrypoints, the rest are contained within those.

### Directory Metadata - `render_metadata` function

protox supports generating an additional metadata file for each directory that has information about the generated files. By including a `fn render_metadata(file, output)` in your `main.rhai` script file, a `metadata` file will be generated using the [MetadataContext](https://github.com/nswarm/protox/blob/main/generator/src/renderer/context/metadata.rs) within each generated directory.

### `render_file` and `render_metadata` Entrypoints

These are passed two parameters:
`file` - This is a `FileContext` meant to be used to write the output.
`output` - This is an `Output` value

### My output isn't showing up!

You can call other functions with either the syntax `my_func()` OR `my_func!()`. The latter runs the function in _same scope_ as the current function.

**IMPORTANT:** because functions pass all arguments _by value_ (they're copied), and you need to append all data to the same `Output` object and return it in `render_file`. That means you can either use the `my_func()` call style, and always return Strings to be added to the `Output`, or you can use `my_func!()` call style, and add directly to the output object.

### Using other `.rhai` files

You can import other `.rhai` files with:
```
import "my_module" as m;
// use like:
m::some_function();
```

Files are resolved based on the root script folder, i.e. where `main.rhai` lives.

### Additional Utilities

These are methods registered by default in the scripted renderer _in addition to_ the default rhai API.

**Array Methods**

`<array>.join(<separator>)`

to_strings and joins each element of `<array>`, separated by the string `<separator>`

```rust
[0, 1, 2].join("::") // "0::1::2"
```

## Templates

Templates are rendered using [Handlebars](https://handlebarsjs.com/). Specifically protox uses [handlebars-rust](https://github.com/sunng87/handlebars-rust) which supports the majority of handlebars functionality.

### Setup

protox requires only a couple files. `file.hbs` is the root of all templates. `config.json` is how you configure the data available in the context when rendering templates.

**Note:** You can quickly initialize a directory with default files using `protox --init`.

Required:
- config.json
- file.hbs

Optional:
- metadata.hbs
- As many other `.hbs` templates as you need

### Configuration

`config.json` defines how the protox renderer contexts are filled with data. The best source for information on what each field does is the [renderer_config.rs](https://github.com/nswarm/protox/blob/main/generator/src/renderer/renderer_config.rs).

### Data Context

The data available when rendering a template (e.g. what `name` is when you say `{{name}}`) is defined in context objects passed to the handlebars renderer.

All context data structures can be seen [here](https://github.com/nswarm/protox/tree/main/generator/src/renderer/context). `file.rs` and `metadata.rs` map to `file.hbs` and `metadata.hbs`, the rest are contained within those.

### Directory Metadata - `metadata.hbs`

protox supports generating an additional metadata file for each directory that has information about the generated files. By including a `metadata.hbs` in your template source directory, a `metadata` file will be generated using the [MetadataContext](https://github.com/nswarm/protox/blob/main/generator/src/renderer/context/metadata.rs) within each generated directory.

### Using Other Template Files

All `.hbs` files within the target template directory will be loaded with file name as their template name. These can be used by using template partials like `{{> template_name}}`.

`file.hbs`
```handlebars
Hello I am from {{source_file}}!

My messages are:
{{#each messages}}
    {{> message}}
{{/each}}
```

`message.hbs`
```handlebars
I am a message! My name is {{name}}.
```

### Custom Helpers

#### `if_equals`

Fairly self-explanatory:

```handlebars
{{#if_equals lhs rhs}}
    lhs == rhs
{{else}}
    lhs != rhs
{{/if_equals}}
```

#### `indent` Helper for Partials

There's a small bug in the template library that does not respect callsite indentation in [partials](https://handlebarsjs.com/guide/partials.html), e.g. `{{> other_template_name}}`. protox contains a workaround helper for this feature that can be used like so:
```handlebars
{{#indent 4}}
{{> package_tree_node}}
{{/indent}}
```

This will indent all content rendered by the partial by 4 spaces. If you're only using the partial once you may as well indent inside the partial itself, but this solves for recursive partials where the callsite indentation is important.

## Roadmap

While protox is largely functional, there's a few things it does not yet support, and a few quality of life features I intend on adding. 

- Retain comments from source protos.
- Protobuf `oneof` types.
- Protobuf nested types.
- Support for always using the fully qualified type name.
- Filtering input descriptor set based on e.g. message options.
- Validation tests, so users can specify the expected output of their set of templates to verify nothing breaks e.g. when upgrading protox's version.

## Architecture

protox operates in three main stages:
1. IDL -> Protobuf.
2. Protobuf -> Template Contexts.
3. Template Contexts -> Rendered Templates.

### 1. IDL -> Protobuf

Google's protobuf compiler `protoc` already can compile `.proto` files to a "descriptor_set" which describes every file, message, field, etc. within the set of input files.

### 2. Protobuf -> Contexts

The major thing protox does is convert a protobuf descriptor set into a hierarchy of "Context" objects.

### 3. Contexts -> Rendered Templates.

**Scripted Renderer**

The scripted renderer uses [rhai](https://rhai.rs/book/) to bind directly to the rust context objects, allowing you to build up complex output files with a powerful scripting language.

**Template Renderer**

The [Handlebars template library](https://handlebarsjs.com/) (specifically, protox uses [handlebars-rust](https://github.com/sunng87/handlebars-rust)) takes in objects defined in json which can be directly referenced within the template. This step serializes the context objects into json, and writes out files using the user-defined templates with the context as data sources.

### Examples

The `examples` folder contains a set of protos in `proto/inputs` that use nearly every available protobuf feature for the sake of testing output.

The templates within `input/templates` are fully functional template sets built for specific purposes. You may use them directly or as a basis for your own template sets.

#### Conceptual Example

A protobuf message defined like so:
```protobuf
message Msg {
  int64 i = 1;
  string str = 2;
}
```

Would be parsed into a `MessageContext` and two `FieldContexts` that would look something like this when serialized to json:

```json
{
  "name": "Msg",
  "fields": [
    {
      "field_name": "i",
      "relative_type": "i64"
    },
    {
      "field_name": "str",
      "relative_type": "String"
    }
  ]
}
```

The above json is a simplification, you can see all the provided values inside the `generator/src/renderer/context` structs.

protox takes a `config.json` file alongside the template files that can customize the styles, name casing, and more that are provided in the context objects. The intent is to simplify the template files themselves as much as possible, within reason. protox is built to grow to new use cases over time, expanding its flexibility through configuration.

**_Option: Script_**

Using code like below define in a `main.rhai` `render_file` function...

```rust
output.line(`pub struct ${message.name}`);
output.push_scope(); // If configured, this will add the "{" and set indent!
for field in message.fields {
    output.append(`${field.name}: ${field.relative_type}`);
}
output.pop_scope(); // Closes with "}" and reduces indent.
```

**_Option: Template_**

Using a template like below defined in `file.hbs`...

```handlebars
pub struct {{name}} {
{{#each fields}}
    {{> field}}
{{/each}}
}
```

and `field.hbs`...
```handlebars
    {{field_name}}: {{relative_type}},
```

**Result**

...the resulting output would be:
```rust
pub struct Msg {
    i: i64,
    str: String,
}
```

Blammo, code generation using only a few lines of script or templates and some configuration.

Check out `builtin` and `examples/input/templates` for complete examples of scripts, templates, and configuration files.

### Crates 
- cli: Thin wrapper to run the main library as a binary.
- generator: The core library that does the IDL -> code generation.

### Key Modules
- generator: Handles command line parsing into a `Config` class used by the other modules.
- protoc: Handles running the protobuf compiler `protoc` to generate the descriptor set file. It can also run protoc directly to generate protobuf code for various languages (See Built-in Support above).
- context: Template context objects serialized to json for the rendering process.
- proto_options: Custom protobuf options used to customize renderer output.
- renderer/scripted_renderer: Handles converting a protobuf descriptor set file to a set of template context objects, and rendering those templates to files.
- renderer/template_renderer: Handles converting a protobuf descriptor set file to a set of template context objects, and rendering those templates to files.

### Protobuf Compiler

The way that **protoc**, the protobuf compiler, works is it is an executable that can run another executable as a plugin, passing it data on stdin and receiving results on stdout. For this to work inside of protox, we have the **cli** executable call the protoc executable with our **protoc-plugin** executable. **protoc-plugin** then calls into our **core** library code.

The **protoc** executable is assumed to be on your PATH. You can directly specify which protoc to use by setting the environment variable `PROTOC_EXE` to the path of the executable.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
