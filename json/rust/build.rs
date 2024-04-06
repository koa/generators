use std::{
    collections::HashSet,
    default::Default,
    env::{self, current_dir},
    fs, path,
};

use convert_case::{Case, Casing};
use prettyplease::unparse;
use proc_macro2::{Span, TokenStream};

use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Comma, PathSep, Pub},
    Arm, Block, Expr, ExprMatch, Field, FieldMutability, FieldValue, File, Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemMod, Lit, Path,
    PathArguments, PathSegment, Stmt, Type, TypePath, Variant, Visibility,
};

use crate::json_model::{
    JsonAnyDefaultValue, JsonCategory, JsonConstantGroup, JsonContent, JsonDirection, JsonElement, JsonLocale, JsonPacketDescription,
    JsonPacketType,
};

fn main() {
    parse_json();
}
mod json_model {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::TfValueType;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonContent {
        pub author: Box<str>,
        pub api_version: JsonApiVersion,
        pub category: JsonCategory,
        pub device_identifier: i16,
        pub name: Box<str>,
        pub display_name: JsonDisplayName,
        pub manufacturer: Box<str>,
        pub description: JsonLocalizedText,
        pub released: bool,
        pub documented: bool,
        pub doc: JsonLocalizedText,
        pub packets: Box<[JsonPacketDescription]>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonApiVersion([u8; 3]);

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub enum JsonCategory {
        Brick,
        Bricklet,
        TNG,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonDisplayName {
        pub short: Box<str>,
        pub long: Box<str>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonLocalizedText(pub HashMap<JsonLocale, Box<str>>);

    #[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
    #[serde(deny_unknown_fields)]
    pub enum JsonLocale {
        #[serde(rename = "en")]
        En,
        #[serde(rename = "de")]
        De,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonPacketDescription {
        pub level: JsonLevel,
        pub r#type: JsonPacketType,
        pub name: Box<str>,
        pub function_id: u8,
        pub since_firmware: JsonApiVersion,
        pub doc: JsonDoc,
        pub elements: Box<[JsonElement]>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum JsonLevel {
        Normal,
        Low,
        High,
    }

    #[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum JsonPacketType {
        Function,
        Callback,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonDoc {
        pub r#type: JsonDocType,
        pub text: JsonLocalizedText,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum JsonDocType {
        BF,
        AF,
        CCF,
        C,
        IF,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonElement {
        pub level: JsonLevel,
        pub name: Box<str>,
        pub r#type: JsonElementType,
        pub cardinality: i32,
        pub direction: JsonDirection,
        pub role: Option<JsonRole>,
        pub extra: Box<[JsonElementExtra]>,
    }

    #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
    #[serde(rename_all = "lowercase")]
    pub enum JsonElementType {
        Int8,
        Int16,
        Int32,
        Int64,
        UInt8,
        UInt16,
        UInt32,
        UInt64,
        Float,
        Bool,
        Char,
        String,
    }

    impl Into<TfValueType> for JsonElementType {
        fn into(self) -> TfValueType {
            match self {
                JsonElementType::Int8 => TfValueType::I8,
                JsonElementType::Int16 => TfValueType::I16,
                JsonElementType::Int32 => TfValueType::I32,
                JsonElementType::Int64 => TfValueType::I64,
                JsonElementType::UInt8 => TfValueType::U8,
                JsonElementType::UInt16 => TfValueType::U16,
                JsonElementType::UInt32 => TfValueType::U32,
                JsonElementType::UInt64 => TfValueType::U64,
                JsonElementType::Float => TfValueType::Float,
                JsonElementType::Bool => TfValueType::Bool,
                JsonElementType::Char => TfValueType::Char,
                JsonElementType::String => TfValueType::String,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum JsonDirection {
        IN,
        OUT,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum JsonRole {
        StreamLength,
        StreamChunkData,
        StreamData,
        StreamChunkOffset,
        StreamChunkWritten,
        StreamWritten,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonElementExtra {
        pub index: Option<u8>,
        pub name: Box<str>,
        pub scale: JsonScale,
        pub unit: Option<JsonUnit>,
        pub range: Option<JsonRange>,
        pub default: Option<JsonAnyDefaultValue>,
        pub constant_group: Option<JsonConstantGroup>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    #[serde(deny_unknown_fields)]
    pub enum JsonScale {
        Fraction { numerator: u32, denominator: u32 },
        Named(JsonScaleName),
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum JsonScaleName {
        Dynamic,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonUnit {
        pub title: JsonLocalizedText,
        pub symbol: Box<str>,
        pub usage: JsonLocalizedText,
        pub sequence: JsonLocalizedText,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    #[serde(untagged)]
    pub enum JsonRange {
        Entries(Box<[JsonMinMaxEntry]>),
        Named(JsonRangeName),
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    #[serde(untagged)]
    pub enum JsonMinMaxEntry {
        Integer { minimum: i32, maximum: i32 },
        Character { minimum: char, maximum: char },
        Float { minimum: f32, maximum: f32 },
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    #[serde(untagged)]
    pub enum JsonAnyDefaultValue {
        Integer(i64),
        Character(char),
        Float(f32),
        Bool(bool),
        String(Box<str>),
        Array(Box<[JsonAnyDefaultValue]>),
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum JsonRangeName {
        Dynamic,
        Type,
        Constants,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonConstantGroup {
        pub name: Box<str>,
        pub constants: Box<[JsonConstantEntry]>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct JsonConstantEntry {
        pub name: Box<str>,
        pub value: JsonAnyDefaultValue,
    }
}
fn parse_json() {
    let bindings_dir = current_dir().expect("Cannot access current directory").join("bindings");
    let dir = bindings_dir.read_dir().expect("Cannot read directory");
    let file_contents: Vec<JsonContent> = dir
        .into_iter()
        .map(|entry| entry.expect("cannot access directory entry"))
        .filter(|entry| entry.file_name().to_str().map(|name| name.ends_with(".json")).unwrap_or(false))
        .map(|json_dir_entry| fs::File::open(&json_dir_entry.path()).expect("Cannot open json file"))
        .map(|json_file| serde_json::from_reader::<_, JsonContent>(json_file).expect("Cannot parse json"))
        .collect();
    let mut bindings_content = Vec::new();

    let mut device_variants: Punctuated<Variant, Comma> = Default::default();
    let mut device_encode_arms = Vec::new();
    let mut device_parse_arms = Vec::new();
    let mut device_name_arms = Vec::new();

    for tf_device in file_contents {
        if tf_device.device_identifier < 1 {
            println!("Invalid device identifier: {tf_device:#?}");
            continue;
        }
        let device_id: u16 = tf_device.device_identifier as u16;
        let raw_package_name = tf_device.name;
        println!("Device: {raw_package_name}");

        if raw_package_name.as_ref() == "Unknown" {
            // probleme mit doppelten einträgen in der config
            continue;
        }

        let package_name = raw_package_name.to_string().to_case(Case::Snake);
        let package_ident = create_ident(&package_name);
        let package_path = parse_quote!(crate::bindings::#package_ident);

        let device_name_prefix = raw_package_name.to_string().to_case(Case::UpperCamel);
        let device_st = match tf_device.category {
            JsonCategory::Brick => {
                format!("{}Brick", device_name_prefix)
            }
            JsonCategory::Bricklet => {
                format!("{}Bricklet", device_name_prefix)
            }
            JsonCategory::TNG => {
                format!("{}TNG", device_name_prefix)
            }
        };
        let device_struct_name = Ident::new(&device_st, Span::call_site());

        device_variants.push(parse_quote!(#device_struct_name));
        device_encode_arms.push(parse_quote!(DeviceIdentifier::#device_struct_name =>#device_id));
        device_parse_arms.push(parse_quote!(#device_id => Ok(DeviceIdentifier::#device_struct_name)));
        device_name_arms.push(parse_quote!(DeviceIdentifier::#device_struct_name =>#raw_package_name));

        let mut items = Vec::new();
        items.push(parse_quote!(
            #[allow(unused_imports)]
            use crate::byte_converter::{FromByteSlice, ToBytes};
        ));
        items.push(parse_quote!(
            #[allow(unused_imports)]
            use tokio_stream::StreamExt;
        ));
        items.push(parse_quote!(
            #[allow(unused_imports)]
            use std::convert::TryInto;
        ));

        println!("Name: {raw_package_name}");
        println!("Package: {package_name}");
        //println!("Tf Device: {tf_device:#?}");
        items.push(parse_quote!(
            #[derive(Clone, Debug)]
            pub struct #device_struct_name {
                device: crate::device::Device,
            }
        ));
        let mut device_impl: ItemImpl = parse_quote!(
            impl #device_struct_name {
                pub fn new(uid: crate::base58::Uid, connection: crate::ip_connection::async_io::AsyncIpConnection) -> #device_struct_name {
                    Self{
                        device: crate::device::Device::new(uid,connection,#raw_package_name)
                    }
                }
                pub fn uid(&self)->crate::base58::Uid{
                    self.device.uid()
                }
            }
        );
        let mut already_declared_constants = HashSet::new();
        for packet_description in tf_device.packets.iter() {
            let function =
                generate_packet_element_item_json(&mut items, packet_description, &package_path, &mut already_declared_constants);
            device_impl.items.push(ImplItem::Fn(function));

            /*for element in packet_description.elements.iter() {
                for extra in element.extra.iter() {
                    if let Some(constant_group) = &extra.constant_group {
                        if already_declared_constants.insert(constant_group.name.as_ref()) {
                            process_constant_group_json(&mut items, element, constant_group);
                        }
                    }
                }
            }*/
        }
        items.push(Item::Impl(device_impl));
        bindings_content.push(Item::Mod(ItemMod {
            attrs: vec![],
            vis: Visibility::Public(Default::default()),
            unsafety: None,
            mod_token: Default::default(),
            ident: package_ident,
            content: Some((Default::default(), items)),
            semi: None,
        }));
    }
    device_parse_arms.push(parse_quote!(_ => Err(())));

    bindings_content.push(Item::Enum(parse_quote!(
        #[derive(Copy,Clone,Eq,PartialEq,Debug,Ord, PartialOrd)]
        pub enum DeviceIdentifier{
            #device_variants
        }
    )));
    let name_match = match_self(device_name_arms);
    bindings_content.push(Item::Impl(parse_quote!(
        impl DeviceIdentifier {
            pub fn name(self) -> &'static str {
                #name_match
            }
        }
    )));
    let encode_match = match_self(device_encode_arms);
    bindings_content.push(Item::Impl(parse_quote!(
        impl Into<u16> for DeviceIdentifier {
            fn into(self) -> u16 {
                #encode_match
            }
        }
    )));
    let parse_match = match_self(device_parse_arms);
    bindings_content.push(Item::Impl(parse_quote!(
        impl std::convert::TryInto<DeviceIdentifier> for u16 {
            type Error = ();
            fn try_into(self) -> Result<DeviceIdentifier, Self::Error> {
                #parse_match
            }
        }
    )));

    let file = File { shebang: None, attrs: vec![], items: bindings_content };
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = path::Path::new(&out_dir).join("bindings.rs");
    fs::write(dest_path, unparse(&file)).expect("Cannot write source file");
}

#[derive(Debug, Eq, PartialEq)]
enum TfValueType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Bool,
    Char,
    String,
    Float,
}

impl ToTokens for TfValueType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(
            match self {
                TfValueType::U8 => {
                    quote!(u8)
                }
                TfValueType::U32 => {
                    quote!(u32)
                }
                TfValueType::U16 => {
                    quote!(u16)
                }
                TfValueType::Bool => {
                    quote!(bool)
                }
                TfValueType::Char => {
                    quote!(char)
                }
                TfValueType::String => {
                    quote!(Box<str>)
                }
                TfValueType::I8 => {
                    quote!(i8)
                }
                TfValueType::I16 => {
                    quote!(i16)
                }
                TfValueType::I32 => {
                    quote!(i32)
                }
                TfValueType::Float => {
                    quote!(f32)
                }
                TfValueType::U64 => {
                    quote!(u64)
                }
                TfValueType::I64 => {
                    quote!(i64)
                }
            }
            .into_token_stream(),
        )
    }
}

impl TfValueType {
    fn convert_value(&self, value: &JsonAnyDefaultValue) -> TokenStream {
        match (self, value) {
            (TfValueType::U8, JsonAnyDefaultValue::Integer(v)) => {
                let value: u8 = *v as u8;
                parse_quote!(#value)
            }
            (TfValueType::I8, JsonAnyDefaultValue::Integer(v)) => {
                let value: i8 = *v as i8;
                parse_quote!(#value)
            }
            (TfValueType::U16, JsonAnyDefaultValue::Integer(v)) => {
                let value: u16 = *v as u16;
                parse_quote!(#value)
            }
            (TfValueType::I16, JsonAnyDefaultValue::Integer(v)) => {
                let value: i16 = *v as i16;
                parse_quote!(#value)
            }
            (TfValueType::U32, JsonAnyDefaultValue::Integer(v)) => {
                let value: u32 = *v as u32;
                parse_quote!(#value)
            }
            (TfValueType::I32, JsonAnyDefaultValue::Integer(v)) => {
                let value: i32 = *v as i32;
                parse_quote!(#value)
            }
            (TfValueType::U64, JsonAnyDefaultValue::Integer(v)) => {
                let value: u64 = *v as u64;
                parse_quote!(#value)
            }
            (TfValueType::I64, JsonAnyDefaultValue::Integer(v)) => {
                let value: i64 = *v as i64;
                parse_quote!(#value)
            }
            (TfValueType::Bool, JsonAnyDefaultValue::Bool(b)) => {
                parse_quote!(#b)
            }
            (TfValueType::Char, JsonAnyDefaultValue::Character(c)) => {
                parse_quote!(#c)
            }
            (TfValueType::String, JsonAnyDefaultValue::String(v)) => {
                let string = v.as_ref();
                parse_quote!(#string)
            }
            (TfValueType::Float, JsonAnyDefaultValue::Float(v)) => {
                parse_quote!(#v)
            }

            _ => panic!("Invalid combination: Type {self:?}, value: {value:?}"),
        }
    }
    fn bytecount(&self, array_length: usize) -> usize {
        match self {
            TfValueType::U8 => array_length,
            TfValueType::I8 => array_length,
            TfValueType::U16 => array_length * 2,
            TfValueType::I16 => array_length * 2,
            TfValueType::U32 => array_length * 4,
            TfValueType::I32 => array_length * 4,
            TfValueType::Bool => (array_length + 7) / 8,
            TfValueType::Char => array_length,
            TfValueType::String => array_length,
            TfValueType::Float => array_length * 4,
            TfValueType::U64 => array_length * 8,
            TfValueType::I64 => array_length * 8,
        }
    }
}

fn generate_packet_element_item_json(
    items: &mut Vec<Item>,
    packet_entry: &JsonPacketDescription,
    base_path: &Path,
    already_declared_constants: &mut HashSet<Box<str>>,
) -> ImplItemFn {
    let packet_name = packet_entry.name.as_ref().to_case(Case::UpperCamel);
    let packet_type = &packet_entry.r#type;
    let function_id = packet_entry.function_id;
    let doc = &packet_entry.doc;

    println!("Packet: {packet_name}");
    let doc_de = doc.text.0.get(&JsonLocale::De).map(|v| v.as_ref()).unwrap_or_default();

    let (mut in_fields, mut out_fields) = parse_packet_elements_json(packet_entry, base_path, items, already_declared_constants);
    match packet_type {
        JsonPacketType::Function => {
            let (request_type, request_size): (Option<Type>, usize) = if in_fields.is_empty() {
                (None, 0)
            } else if in_fields.len() == 1 {
                let (first_field, length) = in_fields.remove(0);
                (Some(first_field.ty), length)
            } else {
                let name = format!("{packet_name}Request");
                let struct_name: Ident = create_ident(&name);
                let size = append_data_object(items, &in_fields, &struct_name);
                (Some(parse_quote!(#base_path::#struct_name)), size)
            };
            let (response_type, response_line): (Type, Option<Stmt>) = if out_fields.is_empty() {
                (parse_quote!(()), None)
            } else if out_fields.len() == 1 {
                let (first_field, length) = out_fields.remove(0);
                let length_literal: Lit = parse_quote!(#length);
                let method_ident = parse_quote!(from_le_byte_slice);
                let args = parse_quote!((&result.body()[0..#length_literal]));
                let read_method_call = static_method_call(&first_field.ty, method_ident, args);
                (first_field.ty, Some(Stmt::Expr(parse_quote!(Ok(#read_method_call)), None)))
            } else {
                let name = format!("{packet_name}Response");
                let struct_name: Ident = create_ident(&name);
                append_data_object(items, &out_fields, &struct_name);
                (
                    parse_quote!(#base_path::#struct_name),
                    Some(Stmt::Expr(parse_quote!(Ok(#base_path::#struct_name::from_le_byte_slice(result.body()))), None)),
                )
            };
            let function_name = create_ident(&packet_entry.name.as_ref().to_case(Case::Snake));
            let mut function_statements = Vec::new();
            if request_type.is_some() {
                function_statements.push(parse_quote!(let mut payload = [0; #request_size];));
                function_statements.push(parse_quote!(crate::byte_converter::ToBytes::write_to_slice(request,&mut payload);))
            } else {
                function_statements.push(parse_quote!(let payload = [0; #request_size];));
            }

            if let Some(response_line) = response_line {
                function_statements.push(parse_quote!(let result = self.device.get(#function_id, &payload).await?;));
                function_statements.push(response_line);
            } else {
                function_statements
                    .push(parse_quote!(self.device.set(#function_id, &payload,Some(std::time::Duration::from_secs(20))).await?;));
                function_statements.push(Stmt::Expr(parse_quote!(Ok(())), None));
            }
            let function_block = Block { brace_token: Default::default(), stmts: function_statements };
            if let Some(request_type) = request_type {
                parse_quote!(
                    #[doc = #doc_de]
                    pub async fn #function_name(&mut self, request: #request_type) -> Result<#response_type, crate::error::TinkerforgeError>
                        #function_block
                )
            } else {
                parse_quote!(
                    #[doc = #doc_de]
                    pub async fn #function_name(&mut self) -> Result<#response_type, crate::error::TinkerforgeError>
                        #function_block
                )
            }
        }
        JsonPacketType::Callback => {
            let function_name = create_ident(&format!("{}_stream", packet_entry.name.as_ref().to_case(Case::Snake)));
            if out_fields.is_empty() {
                let function_block: Block = parse_quote!({self.device
                        .get_callback_receiver(#function_id)
                        .await
                        .map(|_| ())});
                parse_quote!(
                    #[doc = #doc_de]
                    pub async fn #function_name(&mut self) -> impl futures_core::Stream<Item = ()>
                        #function_block
                )
            } else if out_fields.len() == 1 {
                let (first_field, length) = out_fields.remove(0);
                let length_literal: Lit = parse_quote!(#length);
                let method_ident = parse_quote!(from_le_byte_slice);
                let args = parse_quote!((&p.body()[0..#length_literal]));
                let read_method_call = static_method_call(&first_field.ty, method_ident, args);
                let struct_name = first_field.ty;
                let function_block: Block = parse_quote!(
                    {self.device
                            .get_callback_receiver(#function_id)
                            .await
                            .map(|p| #read_method_call)
                        }
                );
                parse_quote!(
                    #[doc = #doc_de]
                    pub async fn #function_name(&mut self) -> impl futures_core::Stream<Item = #struct_name>
                        #function_block
                )
            } else {
                let struct_name: Ident = create_ident(&format!("{packet_name}Callback"));
                append_data_object(items, &mut out_fields, &struct_name);
                let function_block: Block = parse_quote!({
                           self.device
                            .get_callback_receiver(#function_id)
                            .await
                            .map(|p| #struct_name::from_le_byte_slice(p.body()))}
                );
                parse_quote!(
                    #[doc = #doc_de]
                    pub async fn #function_name(&mut self) -> impl futures_core::Stream<Item = #base_path::#struct_name>
                        #function_block
                )
            }
        }
    }
}

fn parse_packet_elements_json(
    packet_entry: &JsonPacketDescription,
    base_path: &Path,
    constant_items: &mut Vec<Item>,
    already_declared_constants: &mut HashSet<Box<str>>,
) -> (Vec<(Field, usize)>, Vec<(Field, usize)>) {
    let mut in_fields = Vec::new();
    let mut out_fields = Vec::new();
    for element_entry in packet_entry.elements.iter() {
        let element_name = element_entry.name.as_ref();
        let element_name_rust = element_name.to_case(Case::Camel);
        let transfer_type: TfValueType = element_entry.r#type.into();
        if element_entry.cardinality < 1 {
            println!("Skip negative cardinality on {}; {element_name}", base_path.into_token_stream().to_string());
            continue;
        }
        let repeat_count = element_entry.cardinality as usize;
        let direction = element_entry.direction;
        let (fields, wrap_enum) = match direction {
            JsonDirection::IN => (&mut in_fields, true),
            JsonDirection::OUT => (&mut out_fields, false),
        };

        let ident = create_ident(&element_name_rust.to_case(Case::Snake));
        let (create_fields, field_size): (Box<[(Type, Ident)]>, _) =
            if element_entry.cardinality > 1 && transfer_type == TfValueType::String {
                (vec![(parse_quote!([char;#repeat_count]), parse_quote!(#ident))].into(), transfer_type.bytecount(repeat_count))
            } else {
                let base_type = transfer_type.to_token_stream();
                let mut found_types: Vec<(Type, Ident)> = Vec::with_capacity(element_entry.extra.len());
                for extra_entry in element_entry.extra.iter() {
                    let extra_entry_name = extra_entry.name.as_ref();
                    let extra_ident = if extra_entry_name == element_name {
                        ident.clone()
                    } else {
                        create_ident(&format!("{element_name_rust} {extra_entry_name}").to_case(Case::Snake))
                    };
                    found_types.push(if let Some(constant_group) = &extra_entry.constant_group {
                        if already_declared_constants.insert(constant_group.name.clone()) {
                            process_constant_group_json(constant_items, element_entry, constant_group);
                        }
                        let constant_type_name = Some(create_ident(&constant_group.name.as_ref().to_case(Case::UpperCamel)));
                        (
                            if wrap_enum {
                                parse_quote!(#base_path::#constant_type_name)
                            } else {
                                parse_quote!(crate::byte_converter::ParsedOrRaw<#base_path::#constant_type_name,#transfer_type>)
                            },
                            parse_quote!(#extra_ident),
                        )
                    } else {
                        (parse_quote!(#base_type), parse_quote!( #extra_ident))
                    });
                }
                if found_types.len() == 1 && element_entry.cardinality > 1 {
                    if let [(base_type, ident)] = &found_types[..] {
                        (vec![(parse_quote!([#base_type;#repeat_count]), ident.clone())].into(), transfer_type.bytecount(repeat_count))
                    } else {
                        panic!("Invalid");
                    }
                } else if found_types.len() == repeat_count {
                    (found_types.into_boxed_slice(), transfer_type.bytecount(1))
                } else {
                    panic!("Count mismatch {} != {}", found_types.len(), element_entry.cardinality);
                }
            };
        for (ty, ident) in create_fields.into_iter().cloned() {
            fields.push((
                Field {
                    attrs: vec![],
                    vis: Visibility::Public(Pub::default()),
                    mutability: FieldMutability::None,
                    ident: Some(ident),
                    colon_token: None,
                    ty,
                },
                field_size,
            ));
        }
    }
    (in_fields, out_fields)
}

fn process_constant_group_json(items: &mut Vec<Item>, element: &JsonElement, group: &JsonConstantGroup) {
    let camel_name = group.name.as_ref().to_case(Case::UpperCamel);
    println!("Constant group: {}", group.name);

    let ty: TfValueType = element.r#type.into();
    let enum_name_ident = create_ident(&camel_name);
    let mut variants: Punctuated<Variant, Comma> = Default::default();
    let mut encode_arms = vec![];
    let mut parse_arms = vec![];
    for constant_entry in group.constants.iter() {
        let name = constant_entry.name.as_ref();
        let value = &constant_entry.value;
        let variant_ident = create_ident(&name.to_case(Case::UpperCamel));
        variants.push(parse_quote!(#variant_ident));
        let value = ty.convert_value(value);
        encode_arms.push(parse_quote!(#enum_name_ident::#variant_ident =>#value));
        parse_arms.push(parse_quote!(#value => Ok(#enum_name_ident::#variant_ident)))
    }
    items.push(parse_quote!(
        #[derive(Copy,Clone,Eq,PartialEq,Debug)]
        pub enum #enum_name_ident{
            #variants
        }
    ));
    let encode_match = ExprMatch {
        attrs: vec![],
        match_token: Default::default(),
        expr: Box::new(parse_quote!(self)),
        brace_token: Default::default(),
        arms: encode_arms,
    };

    parse_arms.push(parse_quote!(_ => Err(())));
    let parse_match = ExprMatch {
        attrs: vec![],
        match_token: Default::default(),
        expr: Box::new(parse_quote!(self)),
        brace_token: Default::default(),
        arms: parse_arms,
    };

    items.push(Item::Impl(parse_quote!(
        impl Into<#ty> for #enum_name_ident {
            fn into(self) -> #ty {
                #encode_match
            }
        }
    )));
    items.push(parse_quote!(
        impl crate::byte_converter::ToBytes for #enum_name_ident {
            fn write_to_slice(self,target: &mut [u8]){
                <#enum_name_ident as Into<#ty>>::into(self).write_to_slice(target);
            }
        }
    ));
    let type_size = ty.bytecount(1);
    items.push(parse_quote!(
        impl crate::byte_converter::FromByteSlice for #enum_name_ident {
            fn from_le_byte_slice(bytes: &[u8])->Self{
                #ty::from_le_byte_slice(bytes).try_into().expect("unsupported enum value")
            }
            fn bytes_expected() -> usize{
                #type_size
            }
        }
    ));

    items.push(Item::Impl(parse_quote!(
        impl std::convert::TryInto<#enum_name_ident> for #ty {
            type Error = ();
            fn try_into(self) -> Result<#enum_name_ident, Self::Error> {
                #parse_match
            }
        }
    )));
}

fn create_ident(string: &str) -> Ident {
    if if string == "type" {
        true
    } else if let Some(first_char) = string.chars().next() {
        !first_char.is_alphabetic()
    } else {
        false
    } {
        Ident::new(&format!("_{string}"), Span::call_site())
    } else {
        Ident::new(string, Span::call_site())
    }
}

fn match_self(arms: Vec<Arm>) -> ExprMatch {
    ExprMatch { attrs: vec![], match_token: Default::default(), expr: Box::new(parse_quote!(self)), brace_token: Default::default(), arms }
}

fn append_data_object(items: &mut Vec<Item>, fields: &[(Field, usize)], struct_name: &Ident) -> usize {
    let mut reader_statements = Vec::<Stmt>::new();
    let mut writer_statements = Vec::<Stmt>::new();
    let mut initialization_fields = Punctuated::<FieldValue, Comma>::new();
    let mut offset = 0;
    let mut struct_fields = Punctuated::<Field, Comma>::new();
    for (field, size) in fields.iter() {
        if let Some(field_name) = &field.ident {
            let offset_before: Lit = parse_quote!(#offset);
            offset += *size;
            let offset_after: Lit = parse_quote!(#offset);
            let read_method_call =
                static_method_call(&field.ty, parse_quote!(from_le_byte_slice), parse_quote!((&bytes[#offset_before..#offset_after])));
            reader_statements.push(parse_quote!(let #field_name = #read_method_call;));
            initialization_fields.push(parse_quote!(#field_name));
            writer_statements.push(parse_quote!(self.#field_name.write_to_slice(&mut target[#offset_before..#offset_after]);));
            struct_fields.push(field.clone());
        }
    }
    let total_size: Lit = parse_quote!(#offset);
    items.push(parse_quote!(
        #[derive(Copy, Clone, PartialEq, Debug)]
        pub struct #struct_name {
            #struct_fields
        }

    ));

    reader_statements.push(Stmt::Expr(parse_quote!(Self{#initialization_fields}), None));
    let read_fields = Block { brace_token: Default::default(), stmts: reader_statements };
    items.push(parse_quote!(
       impl crate::byte_converter::FromByteSlice for #struct_name {
       fn from_le_byte_slice(bytes: &[u8]) -> Self
               #read_fields
       fn bytes_expected() -> usize {
         #total_size
       }
    }));
    let write_fields = Block { brace_token: Default::default(), stmts: writer_statements };
    items.push(parse_quote!(
         impl crate::byte_converter::ToBytes for #struct_name {
            fn write_to_slice(self, target: &mut [u8])
                #write_fields
        }
    ));
    offset
}

fn static_method_call(ty: &Type, method: Ident, args: Punctuated<Expr, Comma>) -> Expr {
    if let Type::Path(TypePath { qself: None, path: Path { leading_colon: _, segments } }) = &ty {
        let seg = segments.last();
        if let Some(PathSegment { ident, arguments: PathArguments::AngleBracketed(bracketed) }) = seg {
            return if segments.len() > 1 {
                let mut type_path: Punctuated<PathSegment, PathSep> = Punctuated::new();
                for segment in segments.iter().take(segments.len() - 1) {
                    type_path.push(segment.clone());
                }
                type_path.push(parse_quote!(#ident));
                parse_quote!(#type_path::#bracketed::#method #args)
            } else {
                parse_quote!(#ident::#bracketed::#method #args)
            };
        }
    }
    if let Type::Array(_) = ty {
        parse_quote!(<#ty>::#method #args)
    } else {
        parse_quote!(#ty::#method #args)
    }
}
