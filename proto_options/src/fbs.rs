use prost::Extendable;
use prost_types;
use rhai::plugin::*;

use crate::extensions;
use crate::extensions::fbs::{IntType, MessageType};

fn int_type_display(int_type: IntType) -> String {
    match int_type {
        IntType::Byte => "byte",
        IntType::Ubyte => "ubyte",
        IntType::Short => "short",
        IntType::Ushort => "ushort",
        IntType::Int => "int",
        IntType::Uint => "uint",
        IntType::Long => "long",
        IntType::Ulong => "ulong",
    }
    .to_owned()
}

fn message_type_display(message_type: MessageType) -> String {
    match message_type {
        MessageType::Struct => "struct",
        MessageType::Union => "union",
    }
    .to_owned()
}

#[export_module]
pub mod api {
    ////////////////////////////////////////////////////
    // FileOptions
    pub type FileOptions = prost_types::FileOptions;

    #[rhai_fn(get = "fbs_file_attributes", pure)]
    pub fn file_attribute(opt: &mut FileOptions) -> rhai::Dynamic {
        opt.extension_data(extensions::fbs::FILE_ATTRIBUTE)
            .map(Clone::clone)
            .unwrap_or(Vec::new())
            .into()
    }

    #[rhai_fn(get = "fbs_file_identifier", pure)]
    pub fn file_identifier(opt: &mut FileOptions) -> String {
        opt.extension_data(extensions::fbs::FILE_IDENTIFIER)
            .map(&String::clone)
            .unwrap_or(String::new())
    }

    #[rhai_fn(get = "fbs_file_extension", pure)]
    pub fn file_extension(opt: &mut FileOptions) -> String {
        opt.extension_data(extensions::fbs::FILE_EXTENSION)
            .map(&String::clone)
            .unwrap_or(String::new())
    }

    #[rhai_fn(get = "fbs_root_type", pure)]
    pub fn root_type(opt: &mut FileOptions) -> String {
        opt.extension_data(extensions::fbs::ROOT_TYPE)
            .map(&String::clone)
            .unwrap_or(String::new())
    }

    ////////////////////////////////////////////////////
    // EnumOptions
    pub type EnumOptions = prost_types::EnumOptions;

    #[rhai_fn(get = "fbs_enum_type", pure)]
    pub fn enum_type(opt: &mut EnumOptions) -> String {
        let int_type = opt
            .extension_data(extensions::fbs::ENUM_TYPE)
            .map(|i| IntType::from_i32(*i).unwrap_or(IntType::Byte))
            .unwrap_or(IntType::Byte);
        int_type_display(int_type)
    }

    ////////////////////////////////////////////////////
    // EnumValueOptions
    // pub type EnumValueOptions = prost_types::EnumValueOptions;

    ////////////////////////////////////////////////////
    // MessageOptions
    pub type MessageOptions = prost_types::MessageOptions;

    #[rhai_fn(get = "fbs_message_type", pure)]
    pub fn message_type(opt: &mut MessageOptions) -> String {
        let message_type = opt
            .extension_data(extensions::fbs::MESSAGE_TYPE)
            .map(|i| MessageType::from_i32(*i).unwrap_or(MessageType::Struct))
            .unwrap_or(MessageType::Struct);
        message_type_display(message_type)
    }

    ////////////////////////////////////////////////////
    // FieldOptions
    pub type FieldOptions = prost_types::FieldOptions;

    #[rhai_fn(get = "fbs_field_type", pure)]
    pub fn field_type(opt: &mut FieldOptions) -> String {
        opt.extension_data(extensions::fbs::FIELD_TYPE)
            .map(&String::clone)
            .unwrap_or(String::new())
    }

    #[rhai_fn(get = "fbs_field_attributes", pure)]
    pub fn field_attr(opt: &mut FieldOptions) -> rhai::Dynamic {
        opt.extension_data(extensions::fbs::FIELD_ATTRIBUTE)
            .map(Clone::clone)
            .unwrap_or(Vec::new())
            .into()
    }

    #[rhai_fn(get = "fbs_bool_default", pure)]
    pub fn field_bool_default(opt: &mut FieldOptions) -> bool {
        opt.extension_data(extensions::fbs::BOOL_DEFAULT)
            .map(Clone::clone)
            .unwrap_or(false)
    }

    #[rhai_fn(get = "fbs_int_default", pure)]
    pub fn field_int_default(opt: &mut FieldOptions) -> rhai::INT {
        opt.extension_data(extensions::fbs::INT_DEFAULT)
            .map(Clone::clone)
            .unwrap_or(0)
    }

    #[rhai_fn(get = "fbs_float_default", pure)]
    pub fn field_float_default(opt: &mut FieldOptions) -> rhai::FLOAT {
        opt.extension_data(extensions::fbs::FLOAT_DEFAULT)
            .map(Clone::clone)
            .unwrap_or(0.0)
    }
}
