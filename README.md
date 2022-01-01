# idlx

idlx is an executable that generates code, type definitions, or related output based on an [IDL](https://en.wikipedia.org/wiki/Interface_description_language). It uses types defined by the IDL as input to render [handlebars](https://handlebarsjs.com/) templates.

You as a user provide:
- A set of files written in a supported IDL.
- A configuration file that defines the specifics of your output, for example the naming convention of your output language.
- A set of handlebars template files that structure the context data.

idlx is built to generate output based on a set of types or APIs, it's especially easy if you already use one of the supported IDLs. idlx can eliminate the need to write IDL parsing logic or a protobuf compiler plugin by letting you get straight to writing templates for the existing context hierarchy.

idlx fills a fairly niche use case, but it is fairly flexible within that niche. Depending on how complex the use case, you may need to modify idlx to support additional data in your templates, but it should be a solid base to build on.

### Built-in Templates

idlx was primarily built to generate boilerplate code for [FFI](https://en.wikipedia.org/wiki/Foreign_function_interface) between two languages. The built-in templates provide an example of this between Rust and C#.

The user designates a **server** language and a **client** language. Code is generated slightly differently for each:
- Server language: Classes and a set of C functions for accessing the fields through an opaque pointer.
- Client language: The corresponding C function accessors and class wrappers that know how/when to call those functions to access fields.

At runtime this gives the server language ownership over objects of the generated types, and the client language a familiar interface into the objects owned by the server language, without needing to serialize or copy the entire object.

See `Server/Client Templates Background` below for more context.

## Usage

Take a look at `examples/run-examples.sh` which runs idlx on the input inside `examples/input` and will produce sets of output in `examples/output`.

You can also run `idlx --help` to get more information on the command line usage.


## Built-in Support

IDLs:
- Protobuf

Templates:
- Rust "Server" types and FFI
- C# "Client" types and FFI

Protobuf generated code:
- All [supported protobuf languages](https://developers.google.com/protocol-buffers) via the protobuf compiler itself (protoc)
- Rust via [prost](https://github.com/tokio-rs/prost)

See the `examples/run-examples.sh` script for various ways of using idlx.

## Roadmap

- Protobuf `oneof` types.
- Protobuf nested types.
- Support for always using the fully qualified type name.
- Filtering input descriptor set based on e.g. message options.
- Template validation tests, so users can specify the expected output of their set of templates to verify nothing breaks e.g. when upgrading idlx's version.

### Creating a new template set

*** todo ***

## Architecture

idlx operates in three main stages:
1. IDL -> Protobuf.
2. Protobuf -> Template Contexts.
3. Template Contexts -> Rendered Templates.

### 1. IDL -> Protobuf

Google's protobuf compiler `protoc` already can compile `.proto` files to a "descriptor_set" which describes every file, message, field, etc. within the set of input files. The other main supported IDL flatbuffers has support to transform flatbuffers files into protobuf files. Any other IDL support would be easiest to do something similar.

### 2. Protobuf -> Template Contexts

The major thing idlx does is convert a protobuf descriptor set into a hierarchy of "Context" objects.

### 3. Template Contexts -> Rendered Templates.

The [Handlebars template library](https://handlebarsjs.com/) (specifically, idlx uses [handlebars-rust](https://github.com/sunng87/handlebars-rust)) takes in objects defined in json which can be directly referenced within the template. This step serializes the context objects into json, and writes out files using the user-defined templates with the context as data sources.

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

idlx takes a `config.json` file alongside the template files that can customize the styles, name casing, and more that are provided in the context objects. The intent is to simplify the template files themselves as much as possible, within reason. idlx is built to grow to new use cases over time, expanding its flexibility through configuration.

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

The way that **protoc**, the protobuf compiler, works is it is an executable that can run another executable as a plugin, passing it data on stdin and receiving results on stdout. For this to work inside of idlx, we have the **cli** executable call the protoc executable with our **protoc-plugin** executable. **protoc-plugin** then calls into our **core** library code.

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

That's where idlx comes in. It generates code like the above so that you don't have to, eliminating the possibility of hand-written bugs.
