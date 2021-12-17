# struct-ffi-gen

struct-ffi-gen is an executable that generates C-ABI-compatible code in one or more languages for seamless and performant direct usage of those types across the library boundary.

The user designates a **server** language and a **client** language. Code is generated slightly differently for each:
- Server language: Classes and a set of C functions for accessing the fields through an opaque pointer.
- Client language: The corresponding C function accessors and class wrappers that know how/when to call those functions to access fields.

At runtime this gives the server language ownership over objects of the generated types, and the client language a familiar interface into the objects owned by the server language, without needing to serialize or copy the entire object.

## Supported Languages

IDLs:
- Protobuf

Server languages:
- Rust

Client languages:
- C#

Direct type output:
- (none)

Protobuf generated code:
- All [supported protobuf languages](https://developers.google.com/protocol-buffers) via the protobuf compiler itself (protoc)
- Rust via [prost](https://github.com/tokio-rs/prost)

See the `examples/run-examples.sh` script for various ways of using struct-ffi-gen.

## Architecture

struct-ffi-gen consumes an [IDL](https://en.wikipedia.org/wiki/Interface_description_language) and produces "client" and "server" structs in target languages using [mustache templates](https://mustache.github.io/).

### Plugins

struct-ffi-gen is built around plugins to allow incremental support for new languages. There are multiple types of plugins that can be added:
- IDL
- Server language
- Client language
- Direct type output (e.g. mirror the IDL types in target language syntax)

### Cargo Crates
The project is split into two main crates:
- core: The core library that does the IDL -> code generation.
- cli: 

Other crates:
- protoc-plugin: An executable passed to the protoc executable as a plugin.

#### Protobuf Compiler

The way that **protoc**, the protobuf compiler, works is it is an executable that can run another executable as a plugin, passing it data on stdin and receiving results on stdout. For this to work inside of struct-ffi-gen, we have the **cli** executable call the protoc executable with our **protoc-plugin** executable. **protoc-plugin** then calls into our **core** library code.

The **protoc** executable is assumed to be on your PATH. You can directly specify which protoc to use by setting the environment variable `PROTOC_EXE` to the path of the executable.

## Background

struct-ffi-gen is built to solve a very specific problem: you have a large amount of owned by one language that you want to inspect in another language, for example a complex data model for an app but with UI is built in another language.

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
2. If our structs are large, and I don't need all of that information, I'm doing a lot of wasted copying.

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

That's where struct-ffi-gen comes in. It generates code like the above so that you don't have to, eliminating the possibility of hand-written bugs.
