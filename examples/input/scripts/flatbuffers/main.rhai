fn render_file(file) {
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
    return 0;
}

fn render_file(context, output) {
    let val = `hello there file ${context.source_file}!`;
    output.append(val);
    for msg in context.messages {
        let val = `hello there msg ${msg.name}!`;
        output.append(val);
    }

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

    return output;
}
