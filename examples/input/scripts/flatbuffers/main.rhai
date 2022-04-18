fn render_file(file, output) {
    output.line(`
    /////////////////////////////////////////////////
    // Messages from: ${file.source_file}
    `);

    for imp in file.imports {
        output.line(`include "${imp.file_name}.fbs";`);
    }
    output.line();

    for enum in file.enums {
        enum!(enum);
        output.line();
    }
    for message in file.messages {
        message!(message);
        output.line();
    }
    return output;
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
    if enum.options.enum_type != "" {
        enum.options.enum_type
    } else {
        "ubyte"
    }
}

fn message(message) {
    output.append(`${message_type(message)} ${message.name}`);
    output.push_scope();
    for field in message.fields {
        field!(field, message.options.msg_type);
    }
    output.pop_scope();
}

fn message_type(message) {
    if message.options.msg_type != "" {
        message.options.msg_type
    } else {
        "table"
    }
}

fn field(field, msg_type) {
    if field.is_map {
        print(`WARNING: Field '${field.name}' has a map, which are unsupported in fbs.`);
        return;
    }
    if field.is_array {
        output.append(`${field.name}: [${field.relative_type}]`);
    } else {
        output.append(`${field.name}: ${field.relative_type}`);
    }
    output.line(field_separator(field, msg_type));
}

fn field_separator(field, msg_type) {
    if msg_type == "union" {
        ","
    } else {
        ";"
    }
}
