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

    #[rhai_fn(get = "fbs_attributes")]
    pub fn file_attribute(opt: &mut FileOptions) -> rhai::Dynamic {
        opt.extension_data(extensions::fbs::FILE_ATTRIBUTE)
            .map(Clone::clone)
            .unwrap_or(Vec::new())
            .into()
    }

    ////////////////////////////////////////////////////
    // EnumOptions
    pub type EnumOptions = prost_types::EnumOptions;

    #[rhai_fn(get = "fbs_enum_type")]
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

    #[rhai_fn(get = "fbs_message_type")]
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

    #[rhai_fn(get = "fbs_field_type")]
    pub fn field_type(opt: &mut FieldOptions) -> String {
        opt.extension_data(extensions::fbs::FIELD_TYPE)
            .map(&String::clone)
            .unwrap_or(String::new())
    }
}
