[![CI](https://github.com/nswarm/protox/workflows/CI/badge.svg)](https://github.com/nswarm/protox/actions/workflows/rust.yml)

# protox

protox is an executable that generates code, type definitions, or related output based on an [IDL](https://en.wikipedia.org/wiki/Interface_description_language). It uses types defined by the IDL and renders [handlebars](https://handlebarsjs.com/) templates.

protox is built to generate output based on a set of types or APIs. protox can eliminate the need to write IDL parsing logic or a protobuf compiler plugin by letting you get straight to writing templates for the existing context hierarchy.

You as a user provide:
- A set of files written in a supported IDL, e.g. Google's protobuf IDL.
- A configuration file that defines the specifics of your output, e.g. naming conventions of your output.
- A set of handlebars template files that structure the data when rendering.

protox produces:
- A set of files based on the IDL definitions, e.g. equivalent type definitions across multiple languages.

## Installation

Currently protox must be cloned/downloaded from source and built with `cargo build`. Then you can run it with `cargo run -- <protox args go here>`.

## Usage

Run `protox --help` to get information on the command line usage.

Take a look at `examples/run-examples.sh` which runs protox on the input inside `examples/input` and will produce sets of output in `examples/output`.

See [Templates](#templates) below for defining your own template set.

### Built-in Support

IDLs:
- Protobuf

Templates:
- Rust "Server" types and FFI (see below)
- C# "Client" types and FFI (see below)

Protobuf generated code:
- All [supported protobuf languages](https://developers.google.com/protocol-buffers) via the protobuf compiler itself (protoc)
- Rust via [prost](https://github.com/tokio-rs/prost)

See the `examples/run-examples.sh` script for various ways of using protox.

#### Built-in Templates

protox was primarily built to generate boilerplate code for [FFI](https://en.wikipedia.org/wiki/Foreign_function_interface) between two languages. The built-in templates provide an example of this between Rust and C#.

The user designates a **server** language and a **client** language. Code is generated slightly differently for each:
- Server language: Classes and a set of C functions for accessing the fields through an opaque pointer.
- Client language: The corresponding C function accessors and class wrappers that know how/when to call those functions to access fields.

At runtime this gives the server language ownership over objects of the generated types, and the client language a familiar interface into the objects owned by the server language, without needing to serialize or copy the entire object.

See [Server/Client Templates Background](#serverclient-templates-background) below for more context.

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
- As many other .hbs templates as you need

### Configuration

`config.json` defines how the protox renderer contexts are filled with data. The best source for information on what each field does is the [renderer_config.rs](https://github.com/nswarm/protox/blob/main/runner/src/template_renderer/renderer_config.rs).

### Data Context

The data available when rendering a template (e.g. what `name` is when you say `{{name}}`) is defined in context objects passed to the handlebars renderer.

All context data structures can be seen [here](https://github.com/nswarm/protox/tree/main/runner/src/template_renderer/context). `file.rs` and `metadata.rs` map to `file.hbs` and `metadata.hbs`, the rest are contained within those.

### Directory Metadata - `metadata.hbs`

protox supports generating an additional metadata file for each directory that has information about the generated files. By including a `metadata.hbs` in your template source directory, a `metadata` file will be generated using the [MetadataContext](https://github.com/nswarm/protox/blob/main/runner/src/template_renderer/context/metadata.rs) within each generated directory.

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
- Template validation tests, so users can specify the expected output of their set of templates to verify nothing breaks e.g. when upgrading protox's version.

## Architecture

protox operates in three main stages:
1. IDL -> Protobuf.
2. Protobuf -> Template Contexts.
3. Template Contexts -> Rendered Templates.

### 1. IDL -> Protobuf

Google's protobuf compiler `protoc` already can compile `.proto` files to a "descriptor_set" which describes every file, message, field, etc. within the set of input files.

### 2. Protobuf -> Template Contexts

The major thing protox does is convert a protobuf descriptor set into a hierarchy of "Context" objects.

### 3. Template Contexts -> Rendered Templates.

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

The above json is a simplification, you can see all the provided values inside the `runner/src/template_renderer/context` structs.

protox takes a `config.json` file alongside the template files that can customize the styles, name casing, and more that are provided in the context objects. The intent is to simplify the template files themselves as much as possible, within reason. protox is built to grow to new use cases over time, expanding its flexibility through configuration.

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

...the resulting output would be:
```rust
pub struct Msg {
    i: i64,
    str: String,
}
```

Blammo, code generation using only templates and some configuration.

Check out `examples/input/templates` for complete examples of templates and configuration files.

### Crates 
- cli: Thin wrapper to run the main library as a binary.
- runner: The core library that does the IDL -> code generation.

### Modules
- runner: Handles command line parsing into a `Config` class used by the other modules.
- protoc: Handles running the protobuf compiler `protoc` to generate the descriptor set file. It can also run protoc directly to generate protobuf code for various languages (See Built-in Support above).
- template_renderer: Handles converting a protobuf descriptor set file to a set of template context objects, and rendering those templates to files.
- context: Template context objects serialized to json for the rendering process.

### Protobuf Compiler

The way that **protoc**, the protobuf compiler, works is it is an executable that can run another executable as a plugin, passing it data on stdin and receiving results on stdout. For this to work inside of protox, we have the **cli** executable call the protoc executable with our **protoc-plugin** executable. **protoc-plugin** then calls into our **core** library code.

The **protoc** executable is assumed to be on your PATH. You can directly specify which protoc to use by setting the environment variable `PROTOC_EXE` to the path of the executable.

### "Server/Client" Templates Background

The "server" and "client" template sets exist to solve a specific problem: you have a large amount of owned by one language that you want to inspect in another language, for example a complex data model for an app but with UI is built in another language.

Given this struct hierarchy in C++:
```c++
struct Inner {
    std::string str;
    int32_t i;
};
struct Outer {
    Inner inner;
    std::string str;
};
```

How would it look to access this type in another language? For structs of only primitive types, we could copy the struct wholesale, but there's a few downsides:
1. That won't work once we have std::string or similar language-specific types added to the mix.
2. If our structs are large, and we don't need all of that information, we're doing a lot of wasted copying.

One way of solving this is to use **opaque pointers** with pure functions that know how to access the values. For example, the above structs could have an FFI layer that looks like: 

```cpp
void* outer_get_inner(void* outer) { /* returns a pointer to our inner type. */ }
size_t outer_get_str(void* outer, char* inout_buffer, const size_t buffer_size) { /* copies our str to the buffer */ }
size_t inner_get_str(void* inner, char* inout_buffer, const size_t buffer_size) { /* copies our str to the buffer */ }
int32_t inner_get_i(void* inner) { /* returns i */ }
```

Then in our other language, in this case C#, we could do something like:
```csharp
class Inner : SafeHandle {
    // C# familiar API.
    public string str => {
        get {
            // create or reuse a buffer.
            // call outer_get_str() to fill the buffer.
            // convert the result to a C# string.
        }
    }
    public int i => inner_get_i(this);
    
    // FFI.
    [DllImport("mydll")]
    private static extern size_t inner_get_str(Inner outer, IntPtr inout_buffer, size_t buffer_size);
    [DllImport("mydll")]
    private static extern size_t inner_get_i(Inner inner);
}

class Outer : SafeHandle {
    // C# familiar API.
    public Inner inner => outer_get_inner(this);
    public string str => {
        get {
            // create or reuse a buffer.
            // call outer_get_str() to fill the buffer.
            // convert the result to a C# string.
        }
    }

    // FFI.
    [DllImport("mydll")]
    private static extern Inner outer_get_inner(Outer outer);
    [DllImport("mydll")]
    private static extern size_t outer_get_str(Outer outer, IntPtr inout_buffer, size_t buffer_size);
}
```

Note that this is skimming over a lot of nuance and some necessary pieces are cut out for sake of brevity.

The benefits are it gives you a way to access objects on the other side without needing to copy the entire object, especially in large hierarchies (e.g. a user's inventory) and you can control access to more complex FFI operations like strings. There's various optimizations you can do within those complex FFI optimizations as well. And it's all wrapped in types familiar to each language.

There's a big downside though: that's a lot of boilerplate! It's also very brittle -- there's a lot of ways to mess it up when writing it by hand across many types.

That's where protox comes in. It generates code like the above so that you don't have to, eliminating the possibility of hand-written bugs.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
