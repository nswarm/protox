fn render_file(file, output) {
    output.multiline(`
    /////////////////////////////////////////////////
    // Messages from: ${file.source_file}
    `);
    namespace!();
    attributes!();
    imports!();
    enums!();
    messages!();
    file_identifier!();
    file_extension!();
    root_type!();
    return output;
}

fn namespace() {
    if file.package_ == "" {
        return;
    }
    output.line(`namespace ${file.package_};`);
    output.line();
}

fn attributes() {
    let attributes = file.overlay("attribute");
    if !attributes.is_valid() {
        return;
    }
    attributes = attributes.as_array();
    if attributes.len == 0 {
        return;
    }
    for attr in attributes {
        output.line(`attribute "${attr.as_str()}";`);
    }
    output.line();
}

fn imports() {
    if file.imports.len == 0 {
        return;
    }
    for imp in file.imports {
        output.line(`include "${imp.file_name}.fbs";`);
    }
    output.line();
}

fn enums() {
    if file.enums.len == 0 {
        return;
    }
    for enum in file.enums {
        enum!(enum);
        output.line();
    }
}

fn messages() {
    if file.messages.len == 0 {
        return;
    }
    for message in file.messages {
        message!(message);
        output.line();
    }
}

fn enum(enum) {
    output.append(`enum ${enum.name} : ${enum_type(enum)}`);
    output.push_scope();
    for value in enum.values {
        output.line(`${value.name} = ${value.number},`);
    }
    output.pop_scope();
}

fn enum_type(enum) {
    if enum.options.fbs_enum_type != "" {
        enum.options.fbs_enum_type
    } else {
        "ubyte"
    }
}

fn message(message) {
    let message_type = message_type(message);
    output.append(`${message_type} ${message.name}`);
    output.push_scope();
    for field in message.fields {
        field!(field, message_type);
    }
    output.pop_scope();
}

fn message_type(message) {
    if message.options.fbs_message_type != "" {
        message.options.fbs_message_type
    } else {
        "table"
    }
}

fn field(field, message_type) {
    if field.is_map {
        print(`WARNING: Field '${field.name}' has a map, which are unsupported in fbs.`);
        return;
    }
    let field_type = field_type(field);
    if field.is_array {
        output.append(`${field.name}: [${field_type}]`);
    } else {
        output.append(`${field.name}: ${field_type}`);
    }
    field_default!();
    field_attributes!();
    output.line(field_separator(field, message_type));
}

fn field_type(field) {
    if field.options.fbs_field_type != "" {
        field.options.fbs_field_type
    } else {
        field.relative_type
    }
}

fn field_default() {
    let value = "";
    if field.options.fbs_bool_default != false {
        value = field.options.fbs_bool_default.to_string();
    } else if field.options.fbs_int_default != 0 {
        value = field.options.fbs_int_default.to_string();
    } else if field.options.fbs_float_default != 0.0 {
        value = field.options.fbs_float_default.to_string();
    }
    if value != "" {
        output.append(` = ${value}`);
    }
}

fn field_attributes() {
    if field.options.fbs_field_attributes.len == 0 {
        return;
    }
    output.space();
    output.append("(");
    output.append(field.options.fbs_field_attributes.join(", "));
    output.append(")")
}

fn field_separator(field, message_type) {
    if message_type == "union" {
        ","
    } else {
        ";"
    }
}

fn file_identifier() {
    let identifier = file.overlay("identifier");
    if identifier.is_valid() {
        output.line(`file_identifier "${identifier.as_str()}";`);
    }
}
fn file_extension() {
    let extension = file.overlay("extension");
    if extension.is_valid() {
        output.line(`file_extension "${extension.as_str()}";`);
    }
}
fn root_type() {
    let root_type = file.overlay("root_type");
    if root_type.is_valid() {
        output.line(`root_type ${root_type.as_str()};`);
    }
}
