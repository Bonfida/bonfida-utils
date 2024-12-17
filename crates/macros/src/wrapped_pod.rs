use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Type, TypeSlice};

pub fn process(mut ast: syn::DeriveInput, is_mut: bool) -> TokenStream {
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
            let number_of_fields = named.len();
            let mut lengths = Vec::with_capacity(number_of_fields);
            let mut field_idents = Vec::with_capacity(number_of_fields);
            let mut cast_to_bytes_statements = Vec::with_capacity(number_of_fields);
            let mut cast_from_bytes_statements = Vec::with_capacity(number_of_fields);
            let mut split_statements = Vec::with_capacity(number_of_fields);

            let mut try_cast_from_bytes_statements = Vec::with_capacity(number_of_fields);
            let mut try_split_statements = Vec::with_capacity(number_of_fields);

            let split_ident = if is_mut {
                Ident::new("split_at_mut", struct_ident.span())
            } else {
                Ident::new("split_at", struct_ident.span())
            };

            for (i, n) in named.into_iter().enumerate() {
                let is_last = i + 1 == number_of_fields;
                let ident = n.ident.clone().unwrap();
                if let Type::Reference(t) = n.ty.clone() {
                    match *t.elem {
                        Type::Slice(TypeSlice { elem, .. }) => {
                            if is_mut {
                                cast_from_bytes_statements
                                    .push(quote!(bytemuck::cast_slice_mut::<u8, _>(#ident)));
                                try_cast_from_bytes_statements
                                    .push(quote!(bytemuck::try_cast_slice_mut::<u8, #elem>(#ident).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, "Cast error"))?));
                            } else {
                                cast_from_bytes_statements
                                    .push(quote!(bytemuck::cast_slice::<u8, _>(#ident)));
                                try_cast_from_bytes_statements
                                    .push(quote!(bytemuck::try_cast_slice::<u8, _>(#ident).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, "Cast error"))?));
                            }
                            if is_last {
                                cast_to_bytes_statements
                                    .push(quote!(bytemuck::cast_slice::<_, u8>(self.#ident)));
                                split_statements.push(quote!(let #ident = buffer;));
                                try_split_statements.push(quote!(let #ident = buffer;));

                                lengths
                                    .push(quote!(self.#ident.len() * std::mem::size_of::<#elem>()));
                            } else {
                                let len_ident = Ident::new(&format!("{ident}_len"), ident.span());
                                split_statements.push(quote! {
                                    let (#len_ident, buffer) = buffer.#split_ident(8);
                                    let #len_ident: &u64 = bytemuck::from_bytes(#len_ident);
                                    let (#ident, buffer) = buffer.#split_ident(*#len_ident as usize);
                                });
                                try_split_statements.push(quote! {
                                    if buffer.len() < 8 {
                                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Buffer too short"))
                                    }
                                    let (#len_ident, buffer) = buffer.#split_ident(8);
                                    let #len_ident: &u64 = bytemuck::from_bytes(#len_ident);

                                    if buffer.len() < *#len_ident as usize {
                                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Buffer too short"))
                                    }

                                    let (#ident, buffer) = buffer.#split_ident(*#len_ident as usize);
                                });
                                cast_to_bytes_statements
                                    .push(quote!(((self.#ident.len() * std::mem::size_of::<#elem>()) as u64).to_le_bytes()));
                                cast_to_bytes_statements
                                    .push(quote!(bytemuck::cast_slice::<_, u8>(self.#ident)));

                                lengths.push(
                                    quote!(self.#ident.len() * std::mem::size_of::<#elem>() + 8),
                                );
                            }
                        }
                        Type::Path(p)
                            if p.path.get_ident().map(|s| s == "str").unwrap_or(false) =>
                        {
                            if is_mut {
                                cast_from_bytes_statements
                                    .push(quote!(std::str::from_utf8_mut(#ident).unwrap()));
                                try_cast_from_bytes_statements.push(quote!(
                                    std::str::from_utf8_mut(#ident).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?
                                ));
                            } else {
                                cast_from_bytes_statements
                                    .push(quote!(std::str::from_utf8(#ident).unwrap()));
                                try_cast_from_bytes_statements.push(quote!(
                                    std::str::from_utf8(#ident).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?
                                ));
                            }
                            if is_last {
                                cast_to_bytes_statements.push(quote!(self.#ident.as_bytes()));
                                split_statements.push(quote!(let #ident = buffer;));
                                try_split_statements.push(quote!(let #ident = buffer;));

                                lengths.push(quote!(self.#ident.len()));
                            } else {
                                let len_ident = Ident::new(&format!("{ident}_len"), ident.span());
                                split_statements.push(quote! {
                                    let (#len_ident, buffer) = buffer.#split_ident(8);
                                    let #len_ident: &u64 = bytemuck::from_bytes(#len_ident);
                                    let (#ident, buffer) = buffer.#split_ident(*#len_ident as usize);
                                });

                                try_split_statements.push(quote! {
                                    if buffer.len() < 8 {
                                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Buffer too short"))
                                    }
                                    let (#len_ident, buffer) = buffer.#split_ident(8);
                                    let #len_ident: &u64 = bytemuck::from_bytes(#len_ident);

                                    if buffer.len() < *#len_ident as usize {
                                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Buffer too short"))
                                    }

                                    let (#ident, buffer) = buffer.#split_ident(*#len_ident as usize);
                                });
                                cast_to_bytes_statements
                                    .push(quote!((self.#ident.len() as u64).to_le_bytes()));
                                cast_to_bytes_statements.push(quote!(self.#ident.as_bytes()));

                                lengths.push(quote!(self.#ident.len() + 8));
                            }
                        }
                        Type::Path(p) => {
                            let len = quote!(std::mem::size_of::<#p>());
                            cast_to_bytes_statements.push(quote!(bytemuck::bytes_of(self.#ident)));

                            if is_mut {
                                cast_from_bytes_statements
                                    .push(quote!(bytemuck::from_bytes_mut(#ident)));
                                try_cast_from_bytes_statements
                                    .push(quote!(bytemuck::try_from_bytes_mut(#ident).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, "From bytes error"))?));
                                split_statements.push(
                                    quote!(let (#ident, buffer) = buffer.split_at_mut(#len);),
                                );
                                try_split_statements.push(quote!(
                                       if buffer.len() < #len {
                                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Buffer too short"))
                                        }
                                    let (#ident, buffer) = buffer.split_at_mut(#len);
                                ));
                            } else {
                                cast_from_bytes_statements
                                    .push(quote!(bytemuck::from_bytes(#ident)));
                                try_cast_from_bytes_statements.push(
                                    quote!(bytemuck::try_from_bytes(#ident).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, "From bytes error"))?),
                                );
                                split_statements
                                    .push(quote!(let (#ident, buffer) = buffer.split_at(#len);));
                                try_split_statements.push(quote!(
                                    if buffer.len() < #len {
                                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Buffer too short"))
                                    }
                                    let (#ident, buffer) = buffer.split_at(#len);
                                ));
                            }
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
            let (target, buffer_type) = if is_mut {
                (quote!(WrappedPodMut), quote!(&'a mut [u8]))
            } else {
                (quote!(WrappedPod), quote!(&'a [u8]))
            };
            let t = quote!(
                impl<'a> #target<'a> for #struct_ident<'a> {
                    fn size(&self) -> usize {
                        #(#lengths)+*
                    }

                    fn export(&self, buffer: &mut Vec<u8>){
                        #(buffer.extend(#cast_to_bytes_statements);)*
                    }

                    fn from_bytes(buffer: #buffer_type) -> Self {
                        #(#split_statements)*
                        Self {#(#field_idents: #cast_from_bytes_statements),*}
                    }

                    fn try_from_bytes(buffer: #buffer_type) -> Result<Box<Self>, std::io::Error> {
                        #(#try_split_statements)*
                        let res = Self {#(#field_idents: #try_cast_from_bytes_statements),*};
                        Ok(Box::new(res))
                    }
                }
            );
            t
        }
        _ => unimplemented!(),
    }
}
