use std::{collections::HashMap, env, env::current_dir, fs, path};

use convert_case::{Case, Casing};
use prettyplease::unparse;
use proc_macro2::{Span, TokenStream};
use pyo3::{
    exceptions::PyTypeError,
    prelude::PyModule,
    types::{PyDict, PyList, PyString, PyTuple},
    FromPyObject, PyAny, PyErr, PyResult, Python,
};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Comma, PathSep, Pub},
    Arm, Block, Expr, ExprMatch, Field, FieldMutability, FieldValue, File, Ident, Item, ItemImpl, ItemMod, Lit, Path, PathArguments,
    PathSegment, Stmt, Type, TypePath, Variant, Visibility,
};

fn main() -> Result<(), PyErr> {
    Python::with_gil(|py| {
        parse_python(py).map_err(|error| {
            error.print_and_set_sys_last_vars(py);
            error
        })
    })
}

#[derive(Debug, FromPyObject)]
struct TfDevice<'a> {
    #[pyo3(item)]
    author: &'a str,
    #[pyo3(item)]
    api_version: [u8; 3],
    #[pyo3(item)]
    category: &'a str,
    #[pyo3(item)]
    device_identifier: i16,
    #[pyo3(item)]
    name: &'a str,
    #[pyo3(item)]
    display_name: &'a str,
    #[pyo3(item)]
    manufacturer: &'a str,
    #[pyo3(item)]
    description: HashMap<&'a str, &'a str>,
    #[pyo3(item)]
    released: bool,
    #[pyo3(item)]
    documented: bool,
    #[pyo3(item)]
    discontinued: bool,
    #[pyo3(item)]
    features: Vec<&'a str>,
    #[pyo3(item)]
    constant_groups: Vec<ConstantGroupEntry<'a>>,
    #[pyo3(item)]
    packets: Vec<PacketEntry<'a>>,
}

#[derive(Debug, FromPyObject)]
struct ConstantGroupEntry<'a> {
    #[pyo3(item)]
    name: &'a str,
    #[pyo3(item("type"))]
    r#type: &'a str,
    #[pyo3(item)]
    constants: Vec<(&'a str, &'a PyAny)>,
}

#[derive(Debug, FromPyObject)]
struct PacketEntry<'a> {
    #[pyo3(item("type"))]
    r#type: &'a str,
    #[pyo3(item)]
    name: &'a str,
    #[pyo3(item)]
    elements: Vec<&'a PyAny>,
    #[pyo3(item)]
    doc: &'a PyAny,
    #[pyo3(item)]
    since_firmware: [u8; 3],
}

#[derive(Debug, FromPyObject)]
struct ElementEntry<'a>(&'a str, &'a str, u8, &'a str, &'a PyAny);

#[derive(Debug, FromPyObject)]
struct ElementDescription<'a> {
    #[pyo3(item)]
    unit: Option<&'a str>,
}

#[derive(Debug, Eq, PartialEq)]
enum TfValueType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    Bool,
    Char,
    String,
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
            }
            .into_token_stream(),
        )
    }
}

impl TfValueType {
    fn try_parse_type(value: &str) -> Option<Self> {
        if value == "uint8" {
            Some(Self::U8)
        } else if value == "uint32" {
            Some(Self::U32)
        } else if value == "uint16" {
            Some(Self::U16)
        } else if value == "int8" {
            Some(Self::I8)
        } else if value == "int16" {
            Some(Self::I16)
        } else if value == "int32" {
            Some(Self::I32)
        } else if value == "bool" {
            Some(Self::Bool)
        } else if value == "char" {
            Some(Self::Char)
        } else if value == "string" {
            Some(Self::String)
        } else {
            None
        }
    }
    fn parse_token_value(&self, value: &PyAny) -> PyResult<TokenStream> {
        let string = value.to_string();
        Ok(match self {
            TfValueType::U8 => string.parse::<u8>()?.into_token_stream(),
            TfValueType::U32 => string.parse::<u32>()?.into_token_stream(),
            TfValueType::U16 => string.parse::<u16>()?.into_token_stream(),
            TfValueType::Bool => string.parse::<bool>()?.into_token_stream(),
            TfValueType::Char => string
                .parse::<char>()
                .map_err(|e| PyErr::new::<PyTypeError, _>(format!("Cannot parse {string} as char: {e}")))?
                .into_token_stream(),
            TfValueType::String => {
                parse_quote!(#string.to_boxed_str())
            }
            TfValueType::I8 => string.parse::<i8>()?.into_token_stream(),
            TfValueType::I16 => string.parse::<i16>()?.into_token_stream(),
            TfValueType::I32 => string.parse::<i32>()?.into_token_stream(),
        })
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
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum TfPacketType {
    Function,
    Callback,
}

impl TfPacketType {
    fn try_parse_type(value: &str) -> Option<Self> {
        if value == "function" {
            Some(Self::Function)
        } else if value == "callback" {
            Some(Self::Callback)
        } else {
            None
        }
    }
}

fn parse_python(py: Python) -> Result<(), PyErr> {
    let buf = current_dir()?;
    let root_dir = buf.parent();
    let path = root_dir.map(|p| p.join("configs")).unwrap();
    let dir = path.read_dir().expect("Cannot read directory");
    PyModule::from_code(
        py,
        "import sys

if sys.hexversion < 0x3040000:
    print('Python >= 3.4 required')
    sys.exit(1)

import os
import shutil
import subprocess
import importlib.util
import importlib.machinery

def create_generators_module():
    generators_dir = os.path.split(os.path.dirname(os.path.realpath(__file__)))[0]

    if sys.hexversion < 0x3050000:
        generators_module = importlib.machinery.SourceFileLoader('generators', os.path.join(generators_dir, '__init__.py')).load_module()
    else:
        generators_spec = importlib.util.spec_from_file_location('generators', os.path.join(generators_dir, '__init__.py'))
        generators_module = importlib.util.module_from_spec(generators_spec)

        generators_spec.loader.exec_module(generators_module)

    sys.modules['generators'] = generators_module

if 'generators' not in sys.modules:
    create_generators_module()

from generators import common",
        "generators.rs",
        "initializer",
    )?;
    let mut bindings_content = Vec::new();
    let mut device_variants: Punctuated<Variant, Comma> = Default::default();
    let mut device_encode_arms = Vec::new();
    let mut device_parse_arms = Vec::new();
    let mut device_name_arms = Vec::new();
    let mut last_device_id = None;

    for entry in dir.into_iter().flatten() {
        if entry.path().file_name().and_then(|f| f.to_str()).map(|n| n.ends_with(".py")).unwrap_or(false) {
            let content = fs::read_to_string(entry.path())?;
            let module = PyModule::from_code(py, &content, "", "")?;
            let com_struct = module.getattr("com")?;
            let tf_device = TfDevice::extract(com_struct)?;
            let name = tf_device.name;
            if Some(tf_device.device_identifier) == last_device_id {
                continue;
            }
            last_device_id = Some(tf_device.device_identifier);
            println!("Python file: {:?}: {name}", entry.path());

            let package_name = name.to_case(Case::Snake);
            let device_struct_name = Ident::new(
                &format!("{}{}", name.to_case(Case::UpperCamel), tf_device.category.to_case(Case::UpperCamel)),
                Span::call_site(),
            );
            let value = tf_device.device_identifier;
            if value > 0 {
                let value = value as u16;
                device_variants.push(parse_quote!(#device_struct_name));
                device_encode_arms.push(parse_quote!(DeviceIdentifier::#device_struct_name =>#value));
                device_parse_arms.push(parse_quote!(#value => Ok(DeviceIdentifier::#device_struct_name)));
                device_name_arms.push(parse_quote!(DeviceIdentifier::#device_struct_name =>#name));
            }

            if name == "Master" {
                let mut items = Vec::new();
                items.push(parse_quote!(
                    use std::time::Duration;
                ));
                items.push(parse_quote!(
                    use crate::{
                        base58::Uid,
                        byte_converter::{FromByteSlice, ToBytes},
                        device::Device,
                        error::TinkerforgeError,
                        ip_connection::async_io::AsyncIpConnection,
                    };
                ));

                println!("Name: {name}");
                println!("Package: {package_name}");
                //println!("Tf Device: {tf_device:#?}");
                items.push(parse_quote!(
                    #[derive(Clone, Debug)]
                    pub struct #device_struct_name {
                        device: Device,
                    }
                ));
                let api_version = tf_device.api_version;
                let av1 = api_version[0];
                let av2 = api_version[1];
                let av3 = api_version[2];
                let mut device_impl: ItemImpl = parse_quote!(
                    impl #device_struct_name {
                        pub fn new(uid: Uid, connection: AsyncIpConnection) -> #device_struct_name {
                            Self{
                                device: Device::new([#av1,#av2,#av3],uid,connection,#name)
                            }
                        }
                    }
                );
                for group in tf_device.constant_groups {
                    let camel_name = group.name.to_case(Case::UpperCamel);
                    let ty = if let Some(ty) = TfValueType::try_parse_type(group.r#type) {
                        ty
                    } else {
                        continue;
                    };
                    let enum_name_ident = Ident::new(&camel_name, Span::call_site());
                    let mut variants: Punctuated<Variant, Comma> = Default::default();
                    let mut encode_arms = vec![];
                    let mut parse_arms = vec![];
                    for (name, value) in group.constants {
                        let variant_ident = Ident::new(&name.to_case(Case::UpperCamel), Span::call_site());
                        variants.push(parse_quote!(#variant_ident));
                        let value = ty.parse_token_value(value)?;
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
                    items.push(Item::Impl(parse_quote!(
                        impl std::convert::TryInto<#enum_name_ident> for #ty {
                            type Error = ();
                            fn try_into(self) -> Result<#enum_name_ident, Self::Error> {
                                #parse_match
                            }
                        }
                    )));
                }
                for (packet_idx, packet_entry) in tf_device.packets.iter().enumerate() {
                    let packet_name = packet_entry.name.to_case(Case::UpperCamel);
                    let packet_type = if let Some(ty) = TfPacketType::try_parse_type(packet_entry.r#type) {
                        ty
                    } else {
                        println!("Unknown Packet type: {}", packet_entry.r#type);
                        continue;
                    };
                    let doc_list = packet_entry.doc.downcast::<PyList>()?;
                    let doc = doc_list.get_item(1)?.downcast::<PyDict>()?;
                    println!("Packet: {packet_name}");
                    let doc_de = doc.get_item("de")?.map(|v| v.to_string()).unwrap_or_default();
                    //println!("Doc: {}", doc_de);
                    let mut in_fields = Vec::new();
                    let mut out_fields = Vec::new();
                    for element_entry in &packet_entry.elements {
                        let element_tuple = element_entry.downcast::<PyTuple>()?;

                        let element_name = element_tuple.get_item(0)?.to_string();
                        let element_name_rust = element_name.to_case(Case::Camel);
                        let transfer_type_str = element_tuple.get_item(1)?.downcast::<PyString>()?.to_str()?;
                        let transfer_type = TfValueType::try_parse_type(transfer_type_str);
                        let repeat_count = usize::extract(element_tuple.get_item(2)?)?;
                        let direction_str = element_tuple.get_item(3)?.downcast::<PyString>()?.to_str()?;
                        let (constant_group, unit) = if element_tuple.len() > 4 {
                            let details = element_tuple.get_item(4)?;
                            if let Ok(params) = details.downcast::<PyDict>() {
                                let unit = string_from_dict(params, "unit")?;
                                let constant_group = string_from_dict(params, "constant_group")?;
                                (constant_group, unit)
                            } else if let Ok(param_list) = details.downcast::<PyList>() {
                                (None, None)
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };
                        let fields = if direction_str == "in" {
                            &mut in_fields
                        } else if direction_str == "out" {
                            &mut out_fields
                        } else {
                            println!("Unknown direction: {direction_str}");
                            continue;
                        };
                        let ident = Some(Ident::new(&element_name_rust.to_case(Case::Snake), Span::call_site()));
                        let (ty, field_size): (Type, _) = if let Some(ty) = transfer_type {
                            (
                                if repeat_count > 1 && ty == TfValueType::String {
                                    parse_quote!([char;#repeat_count])
                                } else {
                                    let base_type = if let Some(constant_group) = constant_group {
                                        let constant_type_name =
                                            Some(Ident::new(&constant_group.to_case(Case::UpperCamel), Span::call_site()));
                                        parse_quote!(crate::byte_converter::ParsedOrRaw<#constant_type_name,#ty>)
                                    } else {
                                        ty.to_token_stream()
                                    };
                                    if repeat_count > 1 {
                                        parse_quote!([#base_type;#repeat_count])
                                    } else {
                                        parse_quote!(#base_type)
                                    }
                                },
                                ty.bytecount(repeat_count),
                            )
                        } else {
                            println!(" ########### Unknown type: {}", transfer_type_str);
                            continue;
                        };

                        //let ty = if repeat_count > 1 { parse_quote!([#base_type;#repeat_count]) } else { parse_quote!(#base_type) };
                        fields.push((
                            Field {
                                attrs: vec![],
                                vis: Visibility::Public(Pub::default()),
                                mutability: FieldMutability::None,
                                ident,
                                colon_token: None,
                                ty,
                            },
                            field_size,
                        ));
                    }
                    if packet_type == TfPacketType::Function {
                        let (request_type, request_size): (Option<Type>, usize) = if in_fields.is_empty() {
                            (None, 0)
                        } else {
                            let name = format!("{packet_name}Request");
                            let struct_name: Ident = Ident::new(&name, Span::call_site());
                            let size = append_data_object(&mut items, &mut in_fields, &struct_name);
                            (Some(parse_quote!(#struct_name)), size)
                        };
                        let (response_type, response_line): (Type, Option<Stmt>) = if out_fields.is_empty() {
                            (parse_quote!(()), None)
                        } else {
                            let name = format!("{packet_name}Response");
                            let struct_name: Ident = Ident::new(&name, Span::call_site());
                            append_data_object(&mut items, &mut out_fields, &struct_name);
                            (
                                parse_quote!(#struct_name),
                                Some(Stmt::Expr(parse_quote!(Ok(#struct_name::from_le_byte_slice(result.body()))), None)),
                            )
                        };
                        let function_name = Ident::new(&packet_entry.name.to_case(Case::Snake), Span::call_site());
                        let function_id = packet_idx as u8 + 1;
                        let mut function_statements = Vec::new();
                        if request_type.is_some() {
                            function_statements.push(parse_quote!(let mut payload = [0; #request_size];));
                            function_statements.push(parse_quote!(request.write_to_slice(&mut payload);))
                        } else {
                            function_statements.push(parse_quote!(let payload = [0; #request_size];));
                        }

                        if let Some(response_line) = response_line {
                            function_statements.push(parse_quote!(let result = self.device.get(#function_id, &payload).await?;));
                            function_statements.push(response_line);
                        } else {
                            function_statements
                                .push(parse_quote!(self.device.set(#function_id, &payload,Some(Duration::from_secs(2))).await?;));
                            function_statements.push(Stmt::Expr(parse_quote!(Ok(())), None));
                        }
                        let function_block = Block { brace_token: Default::default(), stmts: function_statements };
                        if let Some(request_type) = request_type {
                            device_impl.items.push(parse_quote!(
                                pub async fn #function_name(&mut self, request: #request_type) -> Result<#response_type, TinkerforgeError>
                                    #function_block
                            ));
                        } else {
                            device_impl.items.push(parse_quote!(
                                pub async fn #function_name(&mut self) -> Result<#response_type, TinkerforgeError>
                                    #function_block
                            ));
                        }
                    } else if packet_type == TfPacketType::Callback {
                        if !out_fields.is_empty() {
                            let struct_name: Ident = Ident::new(&format!("{packet_name}Callback"), Span::call_site());
                            append_data_object(&mut items, &mut out_fields, &struct_name);
                        }
                    }
                }
                items.push(Item::Impl(device_impl));
                bindings_content.push(Item::Mod(ItemMod {
                    attrs: vec![],
                    vis: Visibility::Public(Default::default()),
                    unsafety: None,
                    mod_token: Default::default(),
                    ident: Ident::new(&package_name, Span::call_site()),
                    content: Some((Default::default(), items)),
                    semi: None,
                }));
            }
        }
    }

    device_parse_arms.push(parse_quote!(_ => Err(())));

    bindings_content.push(Item::Enum(parse_quote!(
        #[derive(Copy,Clone,Eq,PartialEq,Debug)]
        pub enum DeviceIdentifier{
            #device_variants
        }
    )));
    let name_match = match_self(device_name_arms);
    bindings_content.push(Item::Impl(parse_quote!(
        impl DeviceIdentifier {
            fn name(self) -> &'static str {
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
    fs::write(dest_path, unparse(&file))?;

    Ok(())
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
       impl FromByteSlice for #struct_name {
       fn from_le_byte_slice(bytes: &[u8]) -> Self
               #read_fields
       fn bytes_expected() -> usize {
         #total_size
       }
    }));
    let write_fields = Block { brace_token: Default::default(), stmts: writer_statements };
    items.push(parse_quote!(
         impl ToBytes for #struct_name {
            fn write_to_slice(self, target: &mut [u8])
                #write_fields
        }
    ));
    offset
}

fn bytes_expected_expr(ty: &Type) -> Expr {
    static_method_call(ty, parse_quote!(bytes_expected), parse_quote!(()))
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
                parse_quote!(#type_path::#bracketed::#method#args)
            } else {
                parse_quote!(#ident::#bracketed::#method#args)
            };
        }
    }
    if let Type::Array(_) = ty {
        parse_quote!(<#ty>::#method#args)
    } else {
        parse_quote!(#ty::#method#args)
    }
}

fn string_from_dict<'a>(dict: &'a PyDict, key: &str) -> PyResult<Option<&'a str>> {
    if let Some(entry) = dict.get_item(key)? {
        Ok(Some(entry.downcast::<PyString>()?.to_str()?))
    } else {
        Ok(None)
    }
}
