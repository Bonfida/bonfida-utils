use proc_macro2::TokenStream;
use quote::quote;
use syn::{Type, TypeArray, TypeSlice, Variant};

pub fn process(mut ast: syn::DeriveInput) -> TokenStream {
    let struct_ident = ast.ident;
    match &mut ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields:
                syn::Fields::Named(syn::FieldsNamed {
                    brace_token: _,
                    named,
                }),
            ..
        }) => {
            let mut lengths = Vec::with_capacity(named.len());
            let mut field_idents = Vec::with_capacity(named.len());
            let mut cast_to_bytes_statements = Vec::with_capacity(named.len());
            let mut cast_from_bytes_statements = Vec::with_capacity(named.len());
            let mut split_statements = Vec::with_capacity(named.len());
            let mut types = Vec::with_capacity(named.len());

            for n in named.into_iter() {
                let ident = n.ident.clone().unwrap();
                if let Type::Reference(t) = n.ty.clone() {
                    match *t.elem {
                        Type::Slice(TypeSlice { elem, .. }) => {
                            lengths.push(quote!(self.#ident.len() * std::mem::size_of::<#elem>()));
                            cast_to_bytes_statements
                                .push(quote!(bytemuck::cast_slice::<_, u8>(self.#ident)));
                            cast_from_bytes_statements
                                .push(quote!(bytemuck::cast_slice::<u8, _>(#ident)));
                            split_statements.push(quote!(let #ident = buffer;));
                            types.push(quote!(#elem));
                        }
                        Type::Path(p) => {
                            let len = quote!(std::mem::size_of::<#p>());
                            eprintln!("{:?}", quote!(#p));
                            cast_to_bytes_statements.push(quote!(bytemuck::bytes_of(self.#ident)));
                            cast_from_bytes_statements.push(quote!(bytemuck::from_bytes(#ident)));
                            split_statements
                                .push(quote!(let (#ident, buffer) = buffer.split_at(#len);));
                            types.push(quote!(#p));
                            lengths.push(len);
                        }
                        e => panic!("Unsupported type : {:?}", e),
                    }
                } else {
                    panic!("{}", line!())
                }
                field_idents.push(ident);
            }
            lengths.push(quote!(0));
            let t = quote!(
                impl<'a> InstructionParams<'a> for #struct_ident<'a> {
                    fn size(&self) -> usize {
                        #(#lengths)+*
                    }

                    fn write_instruction_data(&self, buffer: &mut Vec<u8>){
                        #(buffer.extend(#cast_to_bytes_statements);)*
                    }

                    fn parse_instruction_data(buffer: &'a [u8]) -> Self {
                        #(#split_statements)*
                        Self {#(#field_idents: #cast_from_bytes_statements),*}
                    }
                }
            );
            t
        }
        _ => unimplemented!(),
    }
}
