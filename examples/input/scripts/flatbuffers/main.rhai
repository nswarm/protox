fn render_file(file, output) {
    output.line(`
    /////////////////////////////////////////////////
    // Messages from: ${file.source_file}
    `);
    namespace!();
    imports!();
    enums!();
    messages!();
    return output;
}

fn namespace() {
    if (file.package_ == "") {
        return;
    }
    output.line(`namespace ${file.package_};`);
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
    output.line(field_separator(field, message_type));
}

fn field_type(field) {
    if field.options.fbs_field_type != "" {
        field.options.fbs_field_type
    } else {
        field.relative_type
    }
}

fn field_separator(field, message_type) {
    if message_type == "union" {
        ","
    } else {
        ";"
    }
}
