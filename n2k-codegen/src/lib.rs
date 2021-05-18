#[macro_use]
extern crate serde_derive;

use heck::*;
use log::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::{collections::HashSet, path::Path};
use std::{fs::File, str::FromStr};
use std::{io::Write, path::PathBuf};

mod canboatxml;
mod keywords;

use canboatxml::*;

pub struct N2kCodeGenOpts {
    pub pgns_xml: String,
    pub pgns: HashSet<u32>,
    pub output: PathBuf,
    /// Whether to generate a crate, and its name. Generates a module otherwise
    pub generate_crate: Option<String>,
}

pub fn codegen(opts: N2kCodeGenOpts) {
    let dest_path = opts.output.join("src");
    std::fs::create_dir_all(&dest_path).ok();

    let my_str = std::fs::read_to_string(&opts.pgns_xml).unwrap();
    let content: PgnsFile = serde_xml_rs::from_str(&my_str).unwrap();

    // create Cargo.toml
    if let Some(crate_name) = opts.generate_crate.as_ref() {
        std::fs::write(
            opts.output.join("Cargo.toml"),
            format!(
                include_str!("../includes/Cargo.toml"),
                crateName = crate_name
            ),
        )
        .unwrap();
    }

    // create types.rs
    std::fs::write(
        dest_path.join("types.rs"),
        include_str!("../includes/types.rs"),
    )
    .unwrap();

    let lib_path = if opts.generate_crate.is_some() {
        dest_path.join("lib.rs")
    } else {
        dest_path.join("mod.rs")
    };

    let mut lib_file = File::create(&lib_path).unwrap();
    writeln!(lib_file, "mod messages;").unwrap();
    writeln!(lib_file, "mod types;").unwrap();

    // PGNs enum with all PGNs
    writeln!(lib_file, "mod pgns;").unwrap();
    writeln!(lib_file, "pub use pgns::Pgns;").unwrap();
    let pgns_file = codegen_pgns_enum(&content);
    let pgns_file_path = dest_path.join("pgns.rs");
    std::fs::write(pgns_file_path, pgns_file.to_string()).unwrap();

    // PGN enum with variants
    writeln!(lib_file, "mod pgn;").unwrap();
    writeln!(lib_file, "pub use pgn::Pgn;").unwrap();
    let pgns_file = codegen_pgns_variant_enum(&content, &opts.pgns);
    let pgns_file_path = dest_path.join("pgn.rs");
    std::fs::write(pgns_file_path, pgns_file.to_string()).unwrap();

    // PGN registry implementation
    writeln!(lib_file, "mod registry;").unwrap();
    writeln!(lib_file, "pub use registry::PgnRegistry;").unwrap();
    let pgns_file = codegen_pgns_registry_impl(&content, &opts.pgns);
    let pgns_file_path = dest_path.join("registry.rs");
    std::fs::write(pgns_file_path, pgns_file.to_string()).unwrap();

    std::fs::create_dir_all(dest_path.join("messages")).ok();
    let gen_lib_path = dest_path.join("messages/mod.rs");
    let mut gen_lib_file = File::create(&gen_lib_path).unwrap();

    content
        .pgns
        .pgn_infos
        .iter()
        .filter(|info| opts.pgns.contains(&info.pgn))
        .for_each(|info| codegen_pgn(&mut lib_file, &mut gen_lib_file, &dest_path, info));

    log::info!("Running rustfmt...");
    let _ = std::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(&opts.output)
        .status()
        .unwrap();
    if opts.generate_crate.is_some() {
        let _ = std::process::Command::new("cargo")
            .arg("check")
            .current_dir(&opts.output)
            .status()
            .unwrap();
    }
}

/// Generate an implementation of the PgnRegistry trait to be used by the n2k embedded_hal_can library
fn codegen_pgns_registry_impl(pgns_file: &PgnsFile, pgns: &HashSet<u32>) -> TokenStream {
    let mut is_fast_packet = vec![];

    for pgn_id in pgns {
        let pgns: Vec<_> = pgns_file
            .pgns
            .pgn_infos
            .iter()
            .filter(|pgn| pgn.pgn == *pgn_id)
            .collect();

        if pgns.is_empty() {
            continue;
        }

        if pgns.len() > 1 {
            panic!(
                "PGNs with more than one variation not supported yet ({})",
                pgn_id
            )
        }

        let pgn = pgns[0];

        if pgn.xtype == "Fast" {
            let pgn_id = TokenStream::from_str(&pgn_id.to_string()).unwrap();
            is_fast_packet.push(quote! {
                #pgn_id
            })
        }
    }

    quote! {
        pub struct PgnRegistry;
        impl n2k::PgnRegistry for PgnRegistry {
            type Message = crate::Pgn;
            type Error = crate::types::N2kError;

            // fn is_known(pgn: u32) -> bool;
            fn is_fast_packet(pgn: u32) -> bool {
                matches!(pgn, #(#is_fast_packet)|*)
            }

            fn build_message(pgn: u32, data: &[u8]) -> Result<Self::Message, Self::Error> {
                crate::Pgn::try_from_bytes(pgn, data)
            }
        }
    }
}

fn codegen_pgns_variant_enum(pgns_file: &PgnsFile, pgns: &HashSet<u32>) -> TokenStream {
    let mut variants = vec![];
    let mut match_arms = vec![];
    for pgn_id in pgns {
        // A PGN can map to multiple variants
        let names: Vec<_> = pgns_file
            .pgns
            .pgn_infos
            .iter()
            .filter(|pgn| pgn.pgn == *pgn_id)
            .map(|v| type_name(&v.id))
            .collect();

        if names.is_empty() {
            continue;
        }

        if names.len() > 1 {
            panic!(
                "PGNs with more than one variation not supported yet ({})",
                pgn_id
            )
        }

        let variant_name = Ident::new(&names[0], Span::call_site());
        variants.push(quote! {
            #variant_name(crate::#variant_name)
        });

        match_arms.push(quote! {
            #pgn_id => Pgn::#variant_name(crate::#variant_name::try_from(bytes)?)
        });
    }
    quote! {
        use crate::types::*;
        use core::convert::TryFrom;

        #[derive(Debug)]
        pub enum Pgn {
            #(#variants),*
        }

        impl Pgn {
            pub fn try_from_bytes(pgn: u32, bytes: &[u8]) -> Result<Pgn, N2kError> {
                Ok(match pgn {
                    #(#match_arms),*,
                    pgn => return Err(N2kError::UnknownPgn(pgn))
                })
            }
        }
    }
}

fn codegen_pgns_enum(pgns: &PgnsFile) -> TokenStream {
    let mut enum_fields = vec![];
    let mut enum_match_arms = vec![];
    let mut names_seen = HashSet::new();
    let pgn_ids: HashSet<_> = pgns.pgns.pgn_infos.iter().map(|v| v.pgn).collect();

    for pgn_id in &pgn_ids {
        let names: Vec<_> = pgns
            .pgns
            .pgn_infos
            .iter()
            .filter(|pgn| pgn.pgn == *pgn_id)
            .map(|v| type_name(&v.id))
            .collect();
        // Give PGNs without a name a field value
        let name = Ident::new(&names.join("_"), Span::call_site());
        if names_seen.contains(&name) {
            continue;
        }
        names_seen.insert(name.to_owned());

        let value = TokenStream::from_str(&pgn_id.to_string()).unwrap();

        enum_fields.push(quote! {
            #name
        });

        enum_match_arms.push(quote! {
            #value => Self::#name
        });
    }

    quote! {
        #![allow(non_camel_case_types)]
        use crate::types::*;

        #[derive(Eq, PartialEq, Debug)]
        pub enum Pgns {
            #(#enum_fields),*
        }

        impl core::convert::TryFrom<u32> for Pgns {
            type Error = N2kError;

            #[inline(always)]
            fn try_from(pgn: u32) -> Result<Self, Self::Error> {
                Ok(match pgn {
                    #(#enum_match_arms),*,
                    v => return Err(N2kError::UnknownPgn(v))
                })
            }
        }
    }
}

fn codegen_pgn(lib_file: &mut File, gen_lib_file: &mut File, path: &Path, pgninfo: &PgnInfo) {
    let struct_name = Ident::new(&type_name(&pgninfo.id), Span::call_site());
    let module_name = pgninfo.id.to_snake_case();

    info!("generating PGN {} / {}", pgninfo.pgn, pgninfo.id);

    writeln!(gen_lib_file, "pub mod {};", module_name).unwrap();
    writeln!(
        lib_file,
        "pub use messages::{}::{};",
        module_name, struct_name
    )
    .unwrap();

    let name = format!("messages/{}.rs", &module_name);
    let current_file_path = path.join(&name);
    let mut message_file = File::create(&current_file_path).unwrap();

    let header = quote! {
        use bitvec::prelude::*;
        use crate::types::*;
    };
    writeln!(message_file, "{}", header).unwrap();

    let size = pgninfo.length;
    let struct_ = quote! {
        pub struct #struct_name {
            raw: [u8; #size],
        }
    };
    let definition_str = format!("// {}", serde_json::to_string(&pgninfo).unwrap());
    writeln!(message_file, "{}", definition_str).unwrap();
    writeln!(message_file, "{}", struct_).unwrap();

    let pgn_id = TokenStream::from_str(&pgninfo.pgn.to_string()).unwrap();
    let try_from = quote! {
        impl core::convert::TryFrom<&[u8]> for #struct_name {
          type Error = N2kError;

            #[inline(always)]
            fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
                if payload.len() < #size {
                    return Err(N2kError::InvalidPayloadSize { expected: #size, actual: payload.len(), pgn: #pgn_id });
                }
                let mut raw = [0u8; #size];
                raw.copy_from_slice(&payload[..#size]);
                Ok(Self { raw })
            }
        }
    };

    writeln!(message_file, "{}", try_from).unwrap();

    // Codegen Enums
    for field in &pgninfo.fields.fields {
        if !field.enum_values.enum_values.is_empty() {
            writeln!(
                message_file,
                "{}",
                codegen_enum(&pgninfo, &field, &field.enum_values)
            )
            .unwrap();
        }
    }

    let impl_tokens = codegen_impl(pgninfo);
    writeln!(message_file, "{}", impl_tokens).unwrap();
}

fn codegen_enum(pgninfo: &PgnInfo, field: &Field, values: &EnumValues) -> TokenStream {
    let enum_int_type = decode_unsigned_int_type_for_bit_length(field.bit_length).0;
    let enum_type_name = lookup_table_type(&field);
    let mut enum_fields = vec![];
    let mut enum_match_arms = vec![];
    // Amazingly, the pgns.xml encodes some enum values as binary, others as decimal.
    // Try to guess if it is in binary if all the values contain only 1 or 0.
    let is_binary = values
        .enum_values
        .iter()
        .all(|v| v.value.chars().all(|b| b == '0' || b == '1'));
    for value in &values.enum_values {
        let variant_name = Ident::new(&type_name(&value.name), Span::call_site());
        let decoded_value = if is_binary {
            usize::from_str_radix(&value.value, 2).unwrap().to_string()
        } else {
            value.value.to_owned()
        };
        let value = TokenStream::from_str(&decoded_value).unwrap();
        enum_fields.push(quote! {
          #variant_name
        });

        enum_match_arms.push(quote! {
            #value => Self::#variant_name
        });
    }

    // TODO: impl Into<>
    quote! {
       #[derive(Debug)]
       pub enum #enum_type_name {
           #(#enum_fields),*,
           Other(#enum_int_type)
       }

        impl core::convert::From<#enum_int_type> for #enum_type_name {
            #[inline(always)]
            fn from(value: #enum_int_type) -> Self {
                match value {
                    #(#enum_match_arms),*,
                    v => Self::Other(v)
                }
            }
        }
    }
}

fn codegen_impl(pgninfo: &PgnInfo) -> TokenStream {
    let struct_name_str = type_name(&pgninfo.id);
    let struct_name = Ident::new(&struct_name_str, Span::call_site());
    let (getters, fields) = codegen_getters(pgninfo);

    let field_debugs: Vec<_> = fields
        .iter()
        .map(|v| {
            let ident = Ident::new(&v, Span::call_site());
            quote! {
                .field(#v, &self.#ident())
            }
        })
        .collect();
    quote! {
        impl #struct_name {
            #getters
        }

        impl core::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(#struct_name_str)
                #(#field_debugs)*
                .finish()
            }
        }
    }
}

fn codegen_getters(pgninfo: &PgnInfo) -> (TokenStream, Vec<String>) {
    let mut getters = vec![];
    let mut generated_fields = vec![];
    // let getters = vec![];
    for field in &pgninfo.fields.fields {
        if field.id == "reserved" {
            continue;
        }
        let field_name = Ident::new(&field_name(&field.id), Span::call_site());
        let field_name_raw = Ident::new(&format!("{}_raw", field_name), Span::call_site());

        getters.push(codegen_raw_get_impl(field, &field_name_raw));
        // If a non-raw getter is available, use that as the main interpretation of it
        if let Some(get) = codegen_get_impl(&pgninfo, field, &field_name_raw, &field_name) {
            generated_fields.push(field_name.to_string());
            getters.push(get);
        } else {
            generated_fields.push(field_name_raw.to_string());
        }
    }

    (
        quote! {
            #(#getters)*
        },
        generated_fields,
    )
}

fn codegen_raw_get_impl(field: &Field, field_name: &Ident) -> TokenStream {
    let (rust_type_raw, is_slice) = decode_unsigned_int_type_for_bit_length(field.bit_length);

    let bit_offset = field.bit_offset;
    let bit_length = field.bit_length;
    let bit_end = bit_offset + bit_length;

    let bits = quote! {
        self.raw.view_bits::<Lsb0>()[#bit_offset .. #bit_end]
    };
    if !is_slice {
        if field.signed {
            let signed_type = decode_signed_int_type_for_bit_length(field.bit_length);
            quote! {
                pub fn #field_name(&self) -> #signed_type {
                    let value = #bits.load_be::<#rust_type_raw>();
                    #signed_type::from_ne_bytes(value.to_ne_bytes())
                }
            }
        } else {
            quote! {
                pub fn #field_name(&self) -> #rust_type_raw {
                    #bits.load_be::<#rust_type_raw>()
                }
            }
        }
    } else {
        quote! {
            pub fn #field_name<'a>(&'a self) -> #rust_type_raw {
                #bits.as_raw_slice()
            }
        }
    }
}

fn codegen_get_impl(
    pgninfo: &PgnInfo,
    field: &Field,
    field_name_raw: &Ident,
    field_name: &Ident,
) -> Option<TokenStream> {
    let rust_type = field.to_rust_type();

    Some(if field.is_string() {
        // string
        quote! {
            pub fn #field_name<'a>(&'a self) -> Result<#rust_type, core::str::Utf8Error> {
                core::str::from_utf8(self.#field_name_raw())
            }
        }
    } else if field.is_enum() {
        // lookup table
        quote! {
            pub fn #field_name(&self) -> #rust_type {
                self.#field_name_raw().into()
            }
        }
    } else if field.is_float() {
        let resolution = TokenStream::from_str(&field.resolution.to_string()).unwrap();
        // float
        quote! {
            pub fn #field_name(&self) -> #rust_type {
                (self.#field_name_raw() as #rust_type) * (#resolution as #rust_type)
            }
        }
    } else {
        info!(
            "unhandled non-raw field {:?} for pgn {}",
            field, pgninfo.pgn
        );
        return None;
    })
}

impl Field {
    pub fn is_float(&self) -> bool {
        (self.resolution - 1.0).abs() > f32::EPSILON && self.resolution != 0.0
    }

    pub fn is_string(&self) -> bool {
        self.n2k_type == "ASCII text"
    }

    pub fn is_enum(&self) -> bool {
        !self.enum_values.enum_values.is_empty()
    }

    pub fn to_rust_type(&self) -> Option<TokenStream> {
        Some(match self.n2k_type.as_str() {
            "Binary data" => decode_unsigned_int_type_for_bit_length(self.bit_length).0,
            "Lookup table" => lookup_table_type(&self),
            "Manufacturer code" => quote! {u16},
            "ASCII text" => quote! {&'a str},
            "Date" => return None,
            "Time" => return None,
            "ASCII or UNICODE string starting with length and control byte" => return None,
            "ASCII string starting with length byte" => return None,
            "String with start/stop byte" => return None,
            "Bitfield" => return None,
            "Latitude"
            | "IEEE Float"
            | "Longitude"
            | "Temperature"
            | "Pressure (hires)"
            | "Temperature (hires)" => decode_float_type_for_bit_length(self.bit_length),
            "Decimal encoded number" => decode_unsigned_int_type_for_bit_length(self.bit_length).0,

            "" => {
                if self.is_float() {
                    decode_float_type_for_bit_length(self.bit_length)
                } else {
                    decode_unsigned_int_type_for_bit_length(self.bit_length).0
                }
            }
            "Integer" => {
                if !self.is_float() {
                    decode_unsigned_int_type_for_bit_length(self.bit_length).0
                } else {
                    eprintln!("resolution = {:#?}", self.resolution);
                    eprintln!("bit_length = {:#?}", self.bit_length);
                    unimplemented!()
                }
            }
            x => panic!("unhandled N2K type {}", x),
        })
    }
}

fn lookup_table_type(field: &Field) -> TokenStream {
    let name = format_ident!("{}", field.id.to_camel_case());
    quote! { #name }
}

fn decode_unsigned_int_type_for_bit_length(bit_length: usize) -> (TokenStream, bool) {
    if bit_length > 64 || bit_length == 0 {
        (quote! { &'a [u8] }, true)
    } else {
        (
            match bit_length {
                _a if _a > 32 && _a <= 64 => quote! { u64 },
                _a if _a > 16 && _a <= 32 => quote! { u32 },
                _a if _a > 8 && _a <= 16 => quote! { u16 },
                _a if _a > 1 && _a <= 8 => quote! { u8 },
                _a if _a == 1 => quote! { bool },
                a => panic!("unhandled bit length {}", a),
            },
            false,
        )
    }
}

fn decode_signed_int_type_for_bit_length(bit_length: usize) -> TokenStream {
    match bit_length {
        _a if _a > 32 && _a <= 64 => quote! { i64 },
        _a if _a > 16 && _a <= 32 => quote! { i32 },
        _a if _a > 8 && _a <= 16 => quote! { i16 },
        _a if _a > 1 && _a <= 8 => quote! { i8 },
        _a if _a == 1 => quote! { bool },
        a => panic!("unhandled bit length {}", a),
    }
}

fn decode_float_type_for_bit_length(bit_length: usize) -> TokenStream {
    match bit_length {
        _a if _a > 16 && _a < 33 => quote! { f32 },
        _a if (8..17).contains(&_a) => quote! { f32 },
        _a if _a < 8 => quote! { f32 },
        _ => quote! { &'a [u8]},
    }
}

fn type_name(x: &str) -> String {
    if keywords::is_keyword(x) || !x.starts_with(|c: char| c.is_ascii_alphabetic()) {
        format!("X{}", x.to_camel_case())
    } else {
        x.to_camel_case()
    }
}

fn field_name(x: &str) -> String {
    if keywords::is_keyword(x) || !x.starts_with(|c: char| c.is_ascii_alphabetic()) {
        format!("x{}", x.to_snake_case())
    } else {
        x.to_snake_case()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        env_logger::init();
    }
}
