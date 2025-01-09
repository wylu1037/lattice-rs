use std::fs;

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::FileDescriptor;
use protobuf_json_mapping::{parse_dyn_from_str, print_to_string};

/// Dynamic message: See https://github.com/stepancheg/rust-protobuf/blob/master/protobuf-examples/dynamic/src/main.rs

/// # Get FileDescriptor from proto content
///
/// ## Parameters
/// + `proto: &str`:
///
/// ## Returns
/// + `FileDescriptor`
pub fn make_file_descriptor(proto: &str) -> FileDescriptor {
    // Here we define `.proto` file source, we are not generating rust sources for it.
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_file = temp_dir.path().join("example.proto");

    // For now, we need to write files to the disk.
    fs::write(&temp_file, proto).unwrap();

    // Parse text `.proto` file to `FileDescriptorProto` message.
    // Note this API is not stable and subject to change.
    // But binary protos can always be generated manually with `protoc` command.
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .pure()
        .includes(&[temp_dir.path().to_path_buf()])
        .input(&temp_file)
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;

    // This is our .proto file converted to `FileDescriptorProto` from `descriptor.proto`.
    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    // Now this `FileDescriptorProto` initialized for reflective access.
    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();

    file_descriptor
}

/// # Serialize Dynamic Message
///
/// ## Parameters
/// + `fd: FileDescriptor`:
/// + `message_name: &str`:
/// + `json: &str`:
///
/// ## Returns
/// + `Vec<u8>`: serialized message bytes
pub fn serialize_message(fd: FileDescriptor, message_name: &str, json: &str) -> Vec<u8> {
    let message_descriptor = fd.message_by_package_relative_name(message_name).unwrap();

    let parse_result = parse_dyn_from_str(&message_descriptor, json).unwrap();
    let serialize_bytes = parse_result.write_to_bytes_dyn().unwrap();

    serialize_bytes
}

/// # Deserialize Dynamic Message
///
/// ## Parameters
/// + `fd: FileDescriptor`:
/// + `message_name: &str`:
/// + `bytes: Vec<u8>`:
///
/// ## Returns
/// + `String`: Json string
pub fn deserialize_message(fd: FileDescriptor, message_name: &str, bytes: Vec<u8>) -> String {
    let message_descriptor = fd.message_by_package_relative_name(message_name).unwrap();

    let mut message = message_descriptor.new_instance();
    message.merge_from_bytes_dyn(bytes.as_slice()).unwrap();

    // protobuf::text_format::print_to_string(message.as_ref());
    // format!("{}", message.to_string())
    print_to_string(message.as_ref()).unwrap()
}

#[cfg(test)]
mod test {
    use protobuf_json_mapping::parse_dyn_from_str;

    use super::*;

    const PROTO: &str = "syntax = 'proto3'; message Student { string name = 1; uint32 age = 2; Address address = 3; } message Address { string province = 1; string city = 2; }";

    #[test]
    fn test_make_file_descriptor() {
        // Here we define `.proto` file source, we are not generating rust sources for it.
        let proto = "syntax = 'proto3'; message Mmm { int32 aaa = 1; }";

        let file_descriptor = make_file_descriptor(proto);

        // Find the message.
        let message_descriptor = file_descriptor
            .message_by_package_relative_name("Mmm")
            .unwrap();
        // Create an empty message.
        // let mut message = message_descriptor.new_instance();

        //
        let result = parse_dyn_from_str(&message_descriptor, r#"{"aaa": 100}"#).unwrap();
        let binding = result.write_to_bytes_dyn().unwrap();
        let bytes = binding.as_slice();
        println!("dynamic {:?}", bytes);
        println!(
            "dynamic json {}",
            protobuf::text_format::print_to_string(result.as_ref())
        );

        // Find the field.
        //let aaa_field = message_descriptor.field_by_name("aaa").unwrap();
        // Set field.
        //aaa_field.set_singular_field(&mut *message, ReflectValueBox::I32(42));

        // Now serialize it to binary format.
        // field number = 1
        // wire_type = 0 (varint)
        // tag = (1 << 3) | 0 = 8
        // value = 42
        //assert_eq!(&[8, 42], message.write_to_bytes_dyn().unwrap().as_slice());

        // Print it as text format.
        //assert_eq!("aaa: 42", protobuf::text_format::print_to_string(&*message));
    }

    #[test]
    fn test_serialize() {
        let file_descriptor = make_file_descriptor(PROTO);
        let bytes = serialize_message(
            file_descriptor,
            "Student",
            r#"{"name": "Jack", "age": 18, "address": {"province": "AnHui", "city": "LuAn"}}"#,
        );

        assert_eq!(
            vec![
                10, 4, 74, 97, 99, 107, 16, 18, 26, 13, 10, 5, 65, 110, 72, 117, 105, 18, 4, 76,
                117, 65, 110,
            ],
            bytes.as_slice()
        )
    }

    #[test]
    fn test_deserialize() {
        let file_descriptor = make_file_descriptor(PROTO);
        let json = deserialize_message(
            file_descriptor,
            "Student",
            vec![
                10, 4, 74, 97, 99, 107, 16, 18, 26, 13, 10, 5, 65, 110, 72, 117, 105, 18, 4, 76,
                117, 65, 110,
            ],
        );

        assert_eq!(
            r#"{"name": "Jack", "age": 18, "address": {"province": "AnHui", "city": "LuAn"}}"#,
            json
        )
    }
}
