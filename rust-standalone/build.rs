use std::{collections::HashMap, default::Default, env, env::current_dir, fs, path};

use convert_case::{Case, Casing};
use prettyplease::unparse;
use proc_macro2::{Span, TokenStream};
use pyo3::{
    exceptions::PyTypeError,
    prelude::PyModule,
    types::{PyDict, PyInt, PyList, PyString, PyTuple},
    FromPyObject, PyAny, PyErr, PyResult, Python,
};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Comma, PathSep, Pub},
    Arm, Block, Expr, ExprMatch, Field, FieldMutability, FieldValue, File, Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemMod, ItemTrait,
    Lit, Path, PathArguments, PathSegment, Stmt, TraitItem, TraitItemFn, Type, TypePath, Variant, Visibility,
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
    packets: Vec<&'a PyAny>,
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
    //#[pyo3(item)]
    //since_firmware: [u8; 3],
}

#[derive(Debug, FromPyObject)]
struct PacketElementTypeListEntry<'a> {
    #[pyo3(item)]
    name: &'a str,
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
    fn try_parse_type(value: &str) -> Option<Self> {
        if value == "uint8" {
            Some(Self::U8)
        } else if value == "uint32" {
            Some(Self::U32)
        } else if value == "uint16" {
            Some(Self::U16)
        } else if value == "uint64" {
            Some(Self::U64)
        } else if value == "int8" {
            Some(Self::I8)
        } else if value == "int16" {
            Some(Self::I16)
        } else if value == "int32" {
            Some(Self::I32)
        } else if value == "int64" {
            Some(Self::I64)
        } else if value == "bool" {
            Some(Self::Bool)
        } else if value == "char" {
            Some(Self::Char)
        } else if value == "string" {
            Some(Self::String)
        } else if value == "float" {
            Some(Self::Float)
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
            TfValueType::Bool => string.to_lowercase().parse::<bool>()?.into_token_stream(),
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
            TfValueType::Float => string.parse::<i32>()?.into_token_stream(),
            TfValueType::U64 => string.parse::<u64>()?.into_token_stream(),
            TfValueType::I64 => string.parse::<i64>()?.into_token_stream(),
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
            TfValueType::Float => array_length * 4,
            TfValueType::U64 => array_length * 8,
            TfValueType::I64 => array_length * 8,
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
    let root_dir = find_generators(buf.parent());
    let path = root_dir.map(|p| p.join("configs")).unwrap();
    println!("Reading from directory {path:?}");
    let dir = path.read_dir().expect("Cannot read directory");
    let initializer_code = format!(
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
    generators_dir = os.path.split(os.path.realpath({path:?}))[0]

    if sys.hexversion < 0x3050000:
        generators_module = importlib.machinery.SourceFileLoader('generators', os.path.join(generators_dir, '__init__.py')).load_module()
    else:
        generators_spec = importlib.util.spec_from_file_location('generators', os.path.join(generators_dir, '__init__.py'))
        generators_module = importlib.util.module_from_spec(generators_spec)

        generators_spec.loader.exec_module(generators_module)

    sys.modules['generators'] = generators_module

if 'generators' not in sys.modules:
    create_generators_module()

from generators import common"
    );
    PyModule::from_code(py, &initializer_code, "generators.rs", "initializer")?;
    let mut bindings_content = Vec::new();
    let mut device_variants: Punctuated<Variant, Comma> = Default::default();
    let mut device_encode_arms = Vec::new();
    let mut device_parse_arms = Vec::new();
    let mut device_name_arms = Vec::new();

    let found_modules = dir
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            if let Some(filename) = entry.path().file_name().and_then(|f| f.to_str()) {
                if filename.ends_with(".py") {
                    let content = fs::read_to_string(entry.path()).ok()?;
                    let module_name = &filename[0..filename.len() - 3];
                    let module = PyModule::from_code(py, &content, filename, module_name).ok()?;
                    Some((module_name.to_string().into_boxed_str(), module))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();
    let mut common_items = Vec::new();
    let mut features = HashMap::<_, (Vec<_>, Vec<_>)>::new();
    for (_, module) in found_modules.iter() {
        if let Ok(com_struct) = module.getattr("common_constant_groups") {
            for common_constant_group_entry_dict in
                com_struct.iter().expect("common_constant_groups is not a list").map(|e| e.expect("cannot access list entry"))
            {
                let feature = String::extract(common_constant_group_entry_dict.get_item("feature").expect("Missing feature"))
                    .expect("Feature is not a string")
                    .into_boxed_str();
                let common_constant_group_entry = ConstantGroupEntry::extract(common_constant_group_entry_dict)
                    .expect("Cannot parse common_constant_groups of {module_name}");
                process_constant_group(&mut features.entry(feature).or_default().0, common_constant_group_entry);
            }
        }
        if let Ok(com_packets) = module.getattr("common_packets") {
            for com_packet in com_packets.iter().expect("Packets are no list").map(|p| p.expect("Cannot decode packet")) {
                let feature = String::extract(com_packet.get_item("feature").expect("Missing feature")).expect("Feature is not a string");
                let feature_data = features.entry(feature.into_boxed_str()).or_default();
                if com_packet.get_item("is_virtual").ok().map(bool::extract).and_then(<PyResult<bool>>::ok).unwrap_or(false) {
                    continue;
                }
                let packet_data = PacketEntry::extract(com_packet).expect("Cannot unpack packet entry");
                let function_id: u8 = com_packet
                    .get_item("function_id")
                    .expect("Missing function id")
                    .downcast::<PyInt>()
                    .expect("Function_ID is not a number")
                    .extract()
                    .expect("Function id is not u8");
                feature_data.1.push((function_id, packet_data));
            }
        }
    }
    let mut feature_trait_impls = HashMap::<_, (Path, Vec<ImplItemFn>)>::new();
    for (feature_name, (mut constants, packets)) in features {
        let trait_name = create_ident(&feature_name.as_ref().to_case(Case::UpperCamel));
        let feature_package_ident = create_ident(&feature_name.as_ref().to_case(Case::Snake));
        let base_package_path = parse_quote!(crate::bindings::common::#feature_package_ident);
        println!("{trait_name} {feature_name}");
        let mut trait_helper_structs = Vec::new();
        trait_helper_structs.append(&mut constants);
        let mut trait_items = Vec::<TraitItem>::new();

        let mut item_impls = Vec::with_capacity(packets.len());
        for (function_id, packet_entry) in packets {
            let mut function = generate_packet_element_item(&mut trait_helper_structs, function_id, &packet_entry, &base_package_path)?;

            let mut attrs = function.attrs.clone();
            attrs.push(parse_quote!(#[allow(async_fn_in_trait)]));
            trait_items.push(TraitItem::Fn(TraitItemFn { attrs, sig: function.sig.clone(), default: None, semi_token: None }));
            function.vis = Visibility::Inherited;
            item_impls.push(function);
        }
        feature_trait_impls.insert(feature_name, (parse_quote!(#base_package_path::#trait_name), item_impls));
        trait_helper_structs.push(Item::Trait(ItemTrait {
            attrs: vec![],
            vis: Visibility::Public(Default::default()),
            unsafety: None,
            auto_token: None,
            restriction: None,
            trait_token: Default::default(),
            ident: trait_name,
            generics: Default::default(),
            colon_token: None,
            supertraits: Default::default(),
            brace_token: Default::default(),
            items: trait_items,
        }));
        common_items.push(Item::Mod(ItemMod {
            attrs: vec![],
            vis: Visibility::Public(Default::default()),
            unsafety: None,
            mod_token: Default::default(),
            ident: feature_package_ident,
            content: Some((Default::default(), trait_helper_structs)),
            semi: None,
        }));
    }
    println!("Features: ");
    for feature_name in feature_trait_impls.keys() {
        println!("- {feature_name}");
    }
    bindings_content.push(Item::Mod(ItemMod {
        attrs: vec![],
        vis: Visibility::Public(Default::default()),
        unsafety: None,
        mod_token: Default::default(),
        ident: create_ident("common"),
        content: Some((Default::default(), common_items)),
        semi: None,
    }));

    for (module_name, module) in found_modules {
        if let Ok(com_struct) = module.getattr("com") {
            let tf_device = TfDevice::extract(com_struct)?;
            let raw_package_name = tf_device.name;
            println!("Python file: {:?}: {raw_package_name}", module_name);

            if raw_package_name == "Unknown" {
                // probleme mit doppelten einträgen in der config
                continue;
            }

            let package_name = raw_package_name.to_case(Case::Snake);
            let package_ident = create_ident(&package_name);
            let package_path = parse_quote!(crate::bindings::#package_ident);
            let device_struct_name = Ident::new(
                &format!("{}{}", raw_package_name.to_case(Case::UpperCamel), tf_device.category.to_case(Case::UpperCamel)),
                Span::call_site(),
            );
            let value = tf_device.device_identifier;
            if value > 0 {
                let value = value as u16;
                device_variants.push(parse_quote!(#device_struct_name));
                device_encode_arms.push(parse_quote!(DeviceIdentifier::#device_struct_name =>#value));
                device_parse_arms.push(parse_quote!(#value => Ok(DeviceIdentifier::#device_struct_name)));
                device_name_arms.push(parse_quote!(DeviceIdentifier::#device_struct_name =>#raw_package_name));
            }

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
            //let api_version = tf_device.api_version;
            /*
            let av1 = api_version[0];
            let av2 = api_version[1];
            let av3 = api_version[2];*/
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
            for group in tf_device.constant_groups {
                process_constant_group(&mut items, group);
            }
            let mut function_id: u8 = 0;
            for packet_entry_any in tf_device.packets.iter() {
                if packet_entry_any.get_item("openhab_doc").ok().map(bool::extract).and_then(<PyResult<bool>>::ok).unwrap_or(false) {
                    continue;
                }
                let packet_entry = PacketEntry::extract(packet_entry_any).expect("Cannot unpack packet entry");
                function_id = if let Some(fid) =
                    packet_entry_any.get_item("function_id").ok().map(|fid| fid.downcast::<PyInt>().expect("Function_ID is not a number"))
                {
                    fid.extract().expect("Cannot extract function id")
                } else {
                    function_id + 1
                };

                let function = generate_packet_element_item(&mut items, function_id, &packet_entry, &package_path)?;
                device_impl.items.push(ImplItem::Fn(function));
            }
            items.push(Item::Impl(device_impl));
            for feature_name in tf_device.features {
                if let Some((path, impls)) = feature_trait_impls.get(feature_name) {
                    let mut feature_impl: ItemImpl = parse_quote!(
                        impl #path for #device_struct_name{
                        }
                    );
                    for item_fn in impls {
                        feature_impl.items.push(ImplItem::Fn(item_fn.clone()));
                    }
                    items.push(Item::Impl(feature_impl));
                } else {
                    panic!("Feature {feature_name} not defined");
                }
            }
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
    fs::write(dest_path, unparse(&file))?;

    Ok(())
}

fn find_generators(path: Option<&path::Path>) -> Option<&path::Path> {
    if let Some(p) = path {
        if p.ends_with("generators") {
            Some(p)
        } else {
            find_generators(p.parent())
        }
    } else {
        None
    }
}

fn generate_packet_element_item(
    items: &mut Vec<Item>,
    function_id: u8,
    packet_entry: &PacketEntry,
    base_path: &Path,
) -> Result<ImplItemFn, PyErr> {
    let packet_name = packet_entry.name.to_case(Case::UpperCamel);
    let packet_type = TfPacketType::try_parse_type(packet_entry.r#type).expect("Unknown Packet type");
    let doc_list = packet_entry.doc.downcast::<PyList>()?;
    let doc = doc_list.get_item(1)?.downcast::<PyDict>()?;

    println!("Packet: {packet_name}");
    let doc_de = doc.get_item("de")?.map(|v| v.to_string()).unwrap_or_default();

    let (mut in_fields, mut out_fields) = parse_packet_elements(&packet_entry, base_path)?;
    Ok(if packet_type == TfPacketType::Function {
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
        let function_name = create_ident(&packet_entry.name.to_case(Case::Snake));
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
    } else if packet_type == TfPacketType::Callback {
        let function_name = create_ident(&format!("{}_stream", packet_entry.name.to_case(Case::Snake)));
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
    } else {
        panic!("Invalid packet type")
    })
}

fn parse_packet_elements(packet_entry: &PacketEntry, base_path: &Path) -> Result<(Vec<(Field, usize)>, Vec<(Field, usize)>), PyErr> {
    let mut in_fields = Vec::new();
    let mut out_fields = Vec::new();
    for element_entry in &packet_entry.elements {
        let element_tuple = element_entry.downcast::<PyTuple>()?;

        let element_name = element_tuple.get_item(0)?.to_string();
        let element_name_rust = element_name.to_case(Case::Camel);
        let transfer_type_str = element_tuple.get_item(1)?.downcast::<PyString>().expect("Type of element is not a string").to_str()?;
        let transfer_type = TfValueType::try_parse_type(transfer_type_str);
        let repeat_count = usize::extract(element_tuple.get_item(2)?)?;
        let direction_str = element_tuple.get_item(3)?.downcast::<PyString>().expect("direction of element is not a string").to_str()?;
        let subelements = if element_tuple.len() > 4 {
            let details = element_tuple.get_item(4)?;
            if let Ok(params) = details.downcast::<PyDict>() {
                let unit = string_from_dict(params, "unit").expect("Cannot parse unit of element");
                let constant_group = string_from_dict(params, "constant_group")?;
                vec![(constant_group, create_ident(&element_name_rust.to_case(Case::Snake)), unit)]
            } else if let Ok(param_list) = details.downcast::<PyList>() {
                if param_list.len() == repeat_count {
                    param_list
                        .into_iter()
                        .map(|e| e.downcast::<PyDict>().expect("List entry is not a dict"))
                        .map(|e| {
                            let name =
                                string_from_dict(e, "name").expect("Error extracting name").expect("No name attribute in list entry");
                            let constant_group = string_from_dict(e, "constant_group").expect("Error extracting constant group");
                            let unit = string_from_dict(e, "unit").expect("Error extracting unit");
                            let element_name_rust = format!("{element_name} {}", name).to_case(Case::Snake);
                            (constant_group, create_ident(&element_name_rust), unit)
                        })
                        .collect()
                } else {
                    panic!("List length not matching count")
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        let (fields, wrap_enum) = if direction_str == "in" {
            (&mut in_fields, true)
        } else if direction_str == "out" {
            (&mut out_fields, false)
        } else {
            panic!("Unknown direction: {direction_str}");
        };
        let ident = create_ident(&element_name_rust.to_case(Case::Snake));
        let (create_fields, field_size): (Box<[(Type, Ident)]>, _) = if let Some(ty) = transfer_type {
            if repeat_count > 1 && ty == TfValueType::String {
                (vec![(parse_quote!([char;#repeat_count]), parse_quote!(#ident))].into(), ty.bytecount(repeat_count))
            } else {
                let base_type = ty.to_token_stream();
                let found_types: Box<[(Type, Ident)]> = if subelements.is_empty() {
                    vec![(parse_quote!(#base_type), parse_quote!(#ident))].into()
                } else {
                    subelements
                        .into_iter()
                        .map(|(constant_group, ident, unit)| {
                            if let Some(constant_group) = constant_group {
                                let constant_type_name = Some(create_ident(&constant_group.to_case(Case::UpperCamel)));
                                (
                                    if wrap_enum {
                                        parse_quote!(#base_path::#constant_type_name)
                                    } else {
                                        parse_quote!(crate::byte_converter::ParsedOrRaw<#base_path::#constant_type_name,#ty>)
                                    },
                                    parse_quote!(#ident),
                                )
                            } else {
                                (parse_quote!(#base_type), parse_quote!( #ident))
                            }
                        })
                        .collect()
                };
                if found_types.len() == 1 && repeat_count > 1 {
                    if let [(base_type, ident)] = found_types.as_ref() {
                        (vec![(parse_quote!([#base_type;#repeat_count]), ident.clone())].into(), ty.bytecount(repeat_count))
                    } else {
                        panic!("Invalid");
                    }
                } else if found_types.len() == repeat_count {
                    (found_types, ty.bytecount(1))
                } else {
                    panic!("Count mismatch");
                }
            }
        } else {
            panic!(" ########### Unknown type: {}", transfer_type_str);
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
    Ok((in_fields, out_fields))
}

fn process_constant_group(items: &mut Vec<Item>, group: ConstantGroupEntry) {
    let camel_name = group.name.to_case(Case::UpperCamel);
    println!("Constant group: {}", group.name);
    let ty = if let Some(ty) = TfValueType::try_parse_type(group.r#type) {
        ty
    } else {
        return;
    };
    let enum_name_ident = create_ident(&camel_name);
    let mut variants: Punctuated<Variant, Comma> = Default::default();
    let mut encode_arms = vec![];
    let mut parse_arms = vec![];
    for (name, value) in group.constants {
        let variant_ident = create_ident(&name.to_case(Case::UpperCamel));
        variants.push(parse_quote!(#variant_ident));
        let value = ty.parse_token_value(value).expect("Cannot parse python value {value:?}");
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

fn string_from_dict<'a>(dict: &'a PyDict, key: &str) -> PyResult<Option<&'a str>> {
    if let Some(entry) = dict.get_item(key)? {
        if let Ok(string_value) = entry.downcast::<PyString>() {
            Ok(Some(string_value.to_str()?))
        } else {
            if entry.is_none() {
                Ok(None)
            } else {
                panic!("Invalid String value: {entry:?}")
            }
        }
    } else {
        Ok(None)
    }
}
