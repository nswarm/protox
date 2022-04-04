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

fn main(files) {
    print("hello from script!");
    other::print_from_other();

    for file in files {
        file::create(file.name);
    }

    return 0;
}
