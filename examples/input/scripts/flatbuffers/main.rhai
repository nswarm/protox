fn render_file(file, output) {
    output.line(`
    /////////////////////////////////////////////////
    // Messages from: ${file.source_file}
    `);

    for imp in file.imports {
        output.line(`include "${imp.file_name}.fbs;"`);
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
    output.line(`enum ${enum.name} : ${enum_type(enum)} {`);
    for value in enum.values {
        output.line(`${value.name} = ${value.number},`);
    }
    output.line("}");
}

fn enum_type(enum) {
    if enum.options.fbs_enum_type != "" {
        enum.options.fbs_enum_type
    } else {
        "ubyte"
    }
}

fn message(message) {
    output.line(`${message_type(message)} ${message.name} {`);
    for field in message.fields {
        field!(field, message.options.fbs_msg_type);
    }
    output.line("}");
}

fn message_type(message) {
    if message.options.fbs_msg_type != "" {
        message.options.fbs_msg_type
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
