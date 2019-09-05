extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index};

#[proc_macro_derive(CardParse, attributes(location))]
pub fn derive_heap_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: HeapSize` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();


    let interior = create_parsing(&input.data);


    let expanded = quote! {
        // The generated impl.
        impl #impl_generics CardParse for #name #ty_generics #where_clause {
            fn cardparse(s: &str) -> Result<#name, failure::Error> {
                let lines: Vec<&str> = s.lines().collect();
                Ok( #name {
                    #interior
                } )
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

// Add a bound `T: HeapSize` to every type parameter T.
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

// Generate an expression to sum up the heap size of each field.
fn create_parsing(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     0 + self.x.heap_size() + self.y.heap_size() + self.z.heap_size()
                    //
                    // but using fully qualified function call syntax.
                    //
                    // We take some care to use the span of each `syn::Field` as
                    // the span of the corresponding `heap_size_of_children`
                    // call. This way if one of the field types does not
                    // implement `HeapSize` then the compiler's error message
                    // underlines which field it is. An example is shown in the
                    // readme of the parent directory.
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        if let Some(attr) = f.attrs.iter().filter(|f| f.path.is_ident("location")).next() {
                            let parsed = ParseAttrs::new(attr);
                            let line = parsed.line;
                            let start = parsed.start;
                            let end = parsed.end;
                            quote!{
                                #name: String::from(lines.get(#line-1).and_then(|s| s.get(#start - 1 .. #end)).unwrap()),
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