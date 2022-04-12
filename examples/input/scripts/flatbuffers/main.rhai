// fn render_file(file) {
    // append(`${message_type} ${name}`);
    // push_scope();
    // for field in message.fields {
    // if field.is_map {
    //     error("field {}: maps are unsupported in fbs.", field.name);
    //     return;
    // }
    // if field.is_array() {
    //     append(`${field.name}: [${field.relative_type}]`);
    // } else {
    //     append(`${field.name}: ${field.relative_type}`);
    // }
    // append(field_separator(field));
    // }
    // pop_scope();
//     return 0;
// }

fn render_file(context, output) {
    output.line(`
    /////////////////////////////////////////////////
    // Messages from: {{source_file}}`);

    for enum in context.enums {
        enum(enum, output);
    }
    output.line("");
    for message in context.messages {
        message(message, output);
    }
    return output;
}

fn enum(enum, output) {
    output.line(enum.name)
}

fn message(message, output) {
    output.line(message.name)
}
