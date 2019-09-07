extern crate proc_macro;

use proc_macro2;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(CardParse, attributes(location))]
pub fn derive_card_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let interior = create_parsing(&input.data);

    let expanded = quote! {
        impl #impl_generics CardParse for #name #ty_generics #where_clause {
            fn cardparse(s: &str) -> Result<#name, crate::ParseError> {
                let lines: Vec<&str> = s.lines().collect();
                Ok( #name {
                    #interior
                } )
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(CardParse));
        }
    }
    generics
}

// Extract a Some(usize) from a syn::Lit or None
fn literal_int(lit: &syn::Lit) -> Option<usize> {
    if let syn::Lit::Int(lint) = lit {
        lint.base10_parse().ok()
    } else {
        None
    }
}

/// Either get the integer valeu associated with a
/// field attribute or fail compilation
macro_rules! get_int_arg {
    ($name:expr, $meta:ident) => {{
        let tmp = literal_int(&$meta.lit);
        if tmp.is_none() {
            panic! {
                "Field '{}' had '{}' value of '{:?}' but it should have been an int literal",
                path_to_string(&$meta.path),
                $name,
                $meta.lit,
            }
        }
        tmp
    }};
}

/// ParseAttrs holds the field level attributes used to derive the
/// CardParse trait
struct ParseAttrs {
    field: String,
    line: usize,
    start: usize,
    end: Option<usize>,
    max: Option<usize>,
}

impl ParseAttrs {
    /// Create a new ParseAttrs from a syn::Attribute
    fn new(field: String, attr: &syn::Attribute) -> Self {
        let mut line = None;
        let mut start = None;
        let mut end = None;
        let mut max = None;
        for meta in attr_meta_list(attr) {
            match AttrKey::new(&meta.path) {
                AttrKey::Line => {
                    line = get_int_arg!("line", meta);
                }
                AttrKey::Start => {
                    start = get_int_arg!("start", meta);
                }
                AttrKey::End => {
                    end = get_int_arg!("end", meta);
                }
                AttrKey::Max => {
                    max = get_int_arg!("max", meta);
                }
                _ => panic! {
                    "Key '{}' in field '{}' is not recognized",
                    path_to_string(&meta.path),
                    path_to_string(&attr.path),
                },
            }
        }
        ParseAttrs {
            field,
            line: line.expect("Line attribute required"),
            start: start.expect("Start attribute required"),
            end,
            max,
        }
    }
}

/// Transition a syn::Attribute to a list of syn::MetaNameValue failing
/// to compile if the types are wrong
fn attr_meta_list(attr: &syn::Attribute) -> Vec<syn::MetaNameValue> {
    if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
        meta_list
            .nested
            .into_iter()
            .map(|m| {
                if let syn::NestedMeta::Meta(syn::Meta::NameValue(key_val)) = m {
                    key_val
                } else {
                    panic!("Field meta wrong format, requires key=value");
                }
            })
            .collect()
    } else {
        panic!("Field meta wrong format");
    }
}

/// Extract a string from a syn::Path if it's an ident
fn path_to_string(p: &syn::Path) -> String {
    p.get_ident().map(|i| i.to_string()).unwrap_or_default()
}

/// Field attribute keys
enum AttrKey {
    Line,
    Start,
    End,
    Max,
    Unrecognized,
}

impl AttrKey {
    /// Convert a path to an AttrKey enum
    fn new(path: &syn::Path) -> Self {
        if path.is_ident("line") {
            AttrKey::Line
        } else if path.is_ident("start") {
            AttrKey::Start
        } else if path.is_ident("end") {
            AttrKey::End
        } else if path.is_ident("max") {
            AttrKey::Max
        } else {
            AttrKey::Unrecognized
        }
    }
}

/// Extract fields from struct data, failing if types are wong
fn unwrap_data_to_fields(data: &Data) -> &syn::FieldsNamed {
    match data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields,
            _ => unimplemented!("CardParse only implmented for named fields"),
        },
        _ => unimplemented!("CardParse only implmented for structs"),
    }
}

/// Convert a field attribute and name into the line
/// creating that field's value from the source string
fn field_instantiation(args: &ParseAttrs) -> proc_macro2::TokenStream {
    let name = syn::Ident::new(args.field.as_str(), proc_macro2::Span::call_site());
    match args {
        ParseAttrs {
            start,
            line,
            end: None,
            max: None,
            ..
        } => quote! {
            #name: String::from(lines.get(#line-1).and_then(|s| {
                s.get(#start - 1 .. )
            }).unwrap()),
        },
        ParseAttrs {
            start,
            line,
            end: Some(end),
            max: None,
            ..
        } => quote! {
            #name: String::from(lines.get(#line-1).and_then(|s| {
                s.get(#start - 1 .. #end)
            }).unwrap()),
        },
        ParseAttrs {
            start,
            line,
            end: None,
            max: Some(max),
            ..
        } => quote! {
            #name: String::from(lines.get(#line-1).and_then(|s| {
                s.get(#start - 1 .. if #max > s.len() {s.len()} else {#max} )
            }).unwrap()),
        },
        ParseAttrs {
            end: Some(_),
            max: Some(_),
            ..
        } => {
            panic! {"Cardparse field attributes may not contain 'max' and 'end' keys at same time"}
        }
    }
}

/// Track the structure of a derived CardParse trait
/// for validity to allow compile time failure
/// of invalid configurations
#[derive(Default)]
struct ParsingStructure {
    lines: HashMap<usize, Vec<ParseAttrs>>,
}

impl ParsingStructure {
    fn insert(&mut self, field: ParseAttrs) {
        let line_ix = field.line;
        let attrs = self.lines.remove(&line_ix).unwrap_or_default();
        let mut new_fields = Vec::new();
        let mut new_field = Some(field);
        for field in attrs {
            if let Some(field_ref) = new_field.as_ref() {
                if field.start > field_ref.end.unwrap_or_default() {
                    new_fields.push(new_field.take().unwrap())
                } else if field.end.is_none() || field.end.map(|e| e > field_ref.start).unwrap() {
                    panic! {"Fields '{}' and '{}' have overlapping locations", field.field, field_ref.field}
                }
            }
            new_fields.push(field);
        }
        if let Some(new_field) = new_field {
            new_fields.push(new_field);
        }
        self.lines.insert(line_ix, new_fields);
    }
}

/// Create the interior code of the parsing method
/// for a strings CardParse implementation
fn create_parsing(data: &Data) -> proc_macro2::TokenStream {
    let mut structure = ParsingStructure::default();
    let fields = unwrap_data_to_fields(data);
    let recurse = fields.named.iter().map(|f| {
        let name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
        if let Some(attr) = f
            .attrs
            .iter()
            .filter(|f| f.path.is_ident("location"))
            .next()
        {
            let parsed_attrs = ParseAttrs::new(name, attr);
            let init_expr = field_instantiation(&parsed_attrs);
            structure.insert(parsed_attrs);
            init_expr
        } else {
            quote! {#name: String::new(),}
        }
    });
    quote! {
        #( #recurse)*
    }
}
