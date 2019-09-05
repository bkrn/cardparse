extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(CardParse, attributes(location))]
pub fn derive_heap_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

fn literal_int(lit: syn::Lit) -> usize {
    if let syn::Lit::Int(lint) = lit {
        lint.base10_parse().unwrap()
    } else {
        panic!{"Field attribute arg was not int literal"}
    }
}

struct ParseAttrs {
    line: usize,
    start: usize,
    end: usize,
}

impl ParseAttrs {
    fn new(attr: &syn::Attribute) -> Self {
        let mut line = 1;
        let mut start = 1;
        let mut end = 1;
        match attr.parse_meta() {
            Ok(syn::Meta::List(meta_list)) => {
                for meta in meta_list.nested {
                    match meta {
                        syn::NestedMeta::Meta(syn::Meta::NameValue(key_val)) => {
                            if key_val.path.is_ident("line") {
                                line = literal_int(key_val.lit);
                            } else if key_val.path.is_ident("start") {
                                start = literal_int(key_val.lit);
                            } else if key_val.path.is_ident("end") {
                                end = literal_int(key_val.lit);
                            } else {
                                panic!{"Could not parse key"};
                            }
                        }
                        _ => panic!{"Field meta wrong format"}
                    }
                }
            }
            _ => panic!{"Field meta wrong format"},
        }
        ParseAttrs{line,start,end}
    }
}

fn create_parsing(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        if let Some(attr) = f.attrs.iter().filter(|f| f.path.is_ident("location")).next() {
                            let parsed = ParseAttrs::new(attr);
                            let line = parsed.line;
                            let start = parsed.start;
                            let end = parsed.end;
                            quote!{
                                #name: String::from(lines.get(#line-1).and_then(|s| {
                                    let stop = if #end > s.chars().count() {s.chars().count()} else {#end};
                                    s.get(#start - 1 .. stop)
                                }).unwrap()),
                            }
                        } else {
                            quote!{#name: String::new(),}
                        }
                    });
                    quote! {
                        #( #recurse)*
                    }
                }
                _ => unimplemented!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}