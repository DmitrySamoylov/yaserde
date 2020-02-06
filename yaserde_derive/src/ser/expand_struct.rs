use attribute::*;
use field_type::*;
use proc_macro2::TokenStream;
use std::collections::BTreeMap;
use std::string::ToString;
use syn::spanned::Spanned;
use syn::DataStruct;
use syn::Ident;

use ser::element::*;

pub fn serialize(
  data_struct: &DataStruct,
  name: &Ident,
  root: &str,
  namespaces: &BTreeMap<String, String>,
) -> TokenStream {
  let build_attributes: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| {
      let field_attrs = YaSerdeAttribute::parse(&field.attrs);
      if !field_attrs.attribute {
        return None;
      }

      let label = &field.ident;

      let label_name = build_label_name(&field, &field_attrs);

      get_field_type(field).and_then(|f| match f {
        FieldType::FieldTypeString
        | FieldType::FieldTypeBool
        | FieldType::FieldTypeI8
        | FieldType::FieldTypeU8
        | FieldType::FieldTypeI16
        | FieldType::FieldTypeU16
        | FieldType::FieldTypeI32
        | FieldType::FieldTypeU32
        | FieldType::FieldTypeI64
        | FieldType::FieldTypeU64
        | FieldType::FieldTypeF32
        | FieldType::FieldTypeF64 => {
          if let Some(ref d) = field_attrs.default {
            let default_function = Ident::new(&d, field.span());
            Some(quote! {
              let struct_start_event =
                if self.#label != #default_function() {
                  struct_start_event.attr(#label_name, &*{
                    use std::mem;
                    unsafe {
                      let content = format!("{}", self.#label);
                      let ret : &'static str = mem::transmute(&content as &str);
                      mem::forget(content);
                      ret
                    }
                  })
                } else {
                  struct_start_event
                };
            })
          } else {
            Some(quote! {
              let struct_start_event = struct_start_event.attr(#label_name, &*{
                use std::mem;
                unsafe {
                  let content = format!("{}", self.#label);
                  let ret : &'static str = mem::transmute(&content as &str);
                  mem::forget(content);
                  ret
                }
              });
            })
          }
        }
        FieldType::FieldTypeOption { data_type } => match *data_type {
          FieldType::FieldTypeString => {
            if let Some(ref d) = field_attrs.default {
              let default_function = Ident::new(&d, field.span());
              Some(quote! {
                let struct_start_event =
                  if self.#label != #default_function() {
                    if let Some(ref value) = self.#label {
                      struct_start_event.attr(#label_name, &value)
                    } else {
                      struct_start_event
                    }
                  } else {
                    struct_start_event
                  };
              })
            } else {
              Some(quote! {
                let struct_start_event =
                  if let Some(ref value) = self.#label {
                    struct_start_event.attr(#label_name, &value)
                  } else {
                    struct_start_event
                  };
              })
            }
          }
          FieldType::FieldTypeBool
          | FieldType::FieldTypeI8
          | FieldType::FieldTypeU8
          | FieldType::FieldTypeI16
          | FieldType::FieldTypeU16
          | FieldType::FieldTypeI32
          | FieldType::FieldTypeU32
          | FieldType::FieldTypeI64
          | FieldType::FieldTypeU64
          | FieldType::FieldTypeF32
          | FieldType::FieldTypeF64 => {
            if let Some(ref d) = field_attrs.default {
              let default_function = Ident::new(&d, field.span());
              Some(quote! {
                let struct_start_event =
                  if self.#label != #default_function() {
                    if let Some(ref value) = self.#label {
                      struct_start_event.attr(#label_name, &*{
                        use std::mem;
                        unsafe {
                          let content = format!("{}", value);
                          let ret : &'static str = mem::transmute(&content as &str);
                          mem::forget(content);
                          ret
                        }
                      })
                    } else {
                      struct_start_event
                    }
                  } else {
                    struct_start_event
                  };
              })
            } else {
              Some(quote! {
                let struct_start_event =
                  if let Some(ref value) = self.#label {
                    struct_start_event.attr(#label_name, &*{
                      use std::mem;
                      unsafe {
                        let content = format!("{}", value);
                        let ret : &'static str = mem::transmute(&content as &str);
                        mem::forget(content);
                        ret
                      }
                    })
                  } else {
                    struct_start_event
                  };
              })
            }
          }
          FieldType::FieldTypeVec { .. } => {
            let item_ident = Ident::new("yas_item", field.span());
            let inner = enclose_formatted_characters(&item_ident, label_name);

            if let Some(ref d) = field_attrs.default {
              let default_function = Ident::new(&d, field.span());

              Some(quote! {
                if self.#label != #default_function() {
                  if let Some(ref yas_list) = self.#label {
                    for yas_item in yas_list.iter() {
                      #inner
                    }
                  }
                }
              })
            } else {
              Some(quote! {
                for yas_item in &self.#label {
                  #inner
                }
              })
            }
          }
          _ => unimplemented!(),
        },
        FieldType::FieldTypeStruct { .. } => {
          if let Some(ref d) = field_attrs.default {
            let default_function = Ident::new(&d, field.span());
            Some(quote! {
              let struct_start_event =
                if self.#label != #default_function() {
                  struct_start_event.attr(#label_name, &*{
                    use std::mem;
                    match yaserde::ser::to_string_content(&self.#label) {
                      Ok(value) => {
                        unsafe {
                          let ret : &'static str = mem::transmute(&value as &str);
                          mem::forget(value);
                          ret
                        }
                      },
                      Err(msg) => return Err("Unable to serialize content".to_owned()),
                    }
                  })
                } else {
                  struct_start_event
                };
            })
          } else {
            Some(quote! {
              let struct_start_event = struct_start_event.attr(#label_name, &*{
                use std::mem;
                match yaserde::ser::to_string_content(&self.#label) {
                  Ok(value) => {
                    unsafe {
                      let ret : &'static str = mem::transmute(&value as &str);
                      mem::forget(value);
                      ret
                    }
                  },
                  Err(msg) => return Err("Unable to serialize content".to_owned()),
                }
              });
            })
          }
        }
        _ => None,
      })
    })
    .filter_map(|x| x)
    .collect();

  let add_namespaces: TokenStream = namespaces
    .iter()
    .map(|(prefix, namespace)| {
      Some(quote!(
        .ns(#prefix, #namespace)
      ))
    })
    .filter_map(|x| x)
    .collect();

  let struct_inspector: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| {
      let field_attrs = YaSerdeAttribute::parse(&field.attrs);
      if field_attrs.attribute {
        return None;
      }

      let label = &field.ident;
      if field_attrs.text {
        return Some(quote!(
          let data_event = XmlEvent::characters(&self.#label);
          let _ret = writer.write(data_event);
        ));
      }

      let label_name = build_label_name(&field, &field_attrs);

      get_field_type(field).and_then(|f| match f {
        FieldType::FieldTypeString
        | FieldType::FieldTypeBool
        | FieldType::FieldTypeI8
        | FieldType::FieldTypeU8
        | FieldType::FieldTypeI16
        | FieldType::FieldTypeU16
        | FieldType::FieldTypeI32
        | FieldType::FieldTypeU32
        | FieldType::FieldTypeI64
        | FieldType::FieldTypeU64
        | FieldType::FieldTypeF32
        | FieldType::FieldTypeF64 => serialize_element(label, label_name, &field_attrs.default),
        FieldType::FieldTypeOption { data_type } => match *data_type {
          FieldType::FieldTypeString
          | FieldType::FieldTypeBool
          | FieldType::FieldTypeI8
          | FieldType::FieldTypeU8
          | FieldType::FieldTypeI16
          | FieldType::FieldTypeU16
          | FieldType::FieldTypeI32
          | FieldType::FieldTypeU32
          | FieldType::FieldTypeI64
          | FieldType::FieldTypeU64
          | FieldType::FieldTypeF32
          | FieldType::FieldTypeF64 => {
            let item_ident = Ident::new("yas_item", field.span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            if let Some(ref d) = field_attrs.default {
              let default_function = Ident::new(&d, field.span());

              Some(quote! {
                if self.#label != #default_function() {
                  if let Some(ref yas_item) = self.#label {
                    #inner
                  }
                }
              })
            } else {
              Some(quote! {
                if let Some(ref yas_item) = self.#label {
                  #inner
                }
              })
            }
          }
          FieldType::FieldTypeVec { .. } => {
            let item_ident = Ident::new("yas_item", field.span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            if let Some(ref d) = field_attrs.default {
              let default_function = Ident::new(&d, field.span());

              Some(quote! {
                if self.#label != #default_function() {
                  if let Some(ref yas_items) = &self.#label {
                    for yas_item in yas_items.iter() {
                      #inner
                    }
                  }
                }
              })
            } else {
              Some(quote! {
                if let Some(ref yas_items) = &self.#label {
                  for yas_item in yas_items.iter() {
                    #inner
                  }
                }
              })
            }
          }
          FieldType::FieldTypeStruct { .. } => Some(if field_attrs.flatten {
            quote! {
              if let Some(ref item) = &self.#label {
                writer.set_start_event_name(None);
                writer.set_skip_start_end(true);
                item.serialize(writer)?;
              }
            }
          } else {
            quote! {
              if let Some(ref item) = &self.#label {
                writer.set_start_event_name(Some(#label_name.to_string()));
                writer.set_skip_start_end(false);
                item.serialize(writer)?;
              }
            }
          }),
          _ => unimplemented!(),
        },
        FieldType::FieldTypeStruct { .. } => Some(if field_attrs.flatten {
          quote! {
            writer.set_start_event_name(None);
            writer.set_skip_start_end(true);
            self.#label.serialize(writer)?;
          }
        } else {
          quote! {
            writer.set_start_event_name(Some(#label_name.to_string()));
            writer.set_skip_start_end(false);
            self.#label.serialize(writer)?;
          }
        }),
        FieldType::FieldTypeVec { data_type } => match *data_type {
          FieldType::FieldTypeString => {
            let item_ident = Ident::new("yas_item", field.span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              for yas_item in &self.#label {
                #inner
              }
            })
          }
          FieldType::FieldTypeBool
          | FieldType::FieldTypeI8
          | FieldType::FieldTypeU8
          | FieldType::FieldTypeI16
          | FieldType::FieldTypeU16
          | FieldType::FieldTypeI32
          | FieldType::FieldTypeU32
          | FieldType::FieldTypeI64
          | FieldType::FieldTypeU64
          | FieldType::FieldTypeF32
          | FieldType::FieldTypeF64 => {
            let item_ident = Ident::new("yas_item", field.span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              for yas_item in &self.#label {
                #inner
              }
            })
          }
          FieldType::FieldTypeOption { .. } => Some(quote! {
            for item in &self.#label {
              if let Some(value) = item {
                writer.set_start_event_name(None);
                writer.set_skip_start_end(false);
                value.serialize(writer)?;
              }
            }
          }),
          FieldType::FieldTypeStruct { .. } => Some(quote! {
            for item in &self.#label {
              writer.set_start_event_name(None);
              writer.set_skip_start_end(false);
              item.serialize(writer)?;
            }
          }),
          FieldType::FieldTypeVec { .. } => {
            unimplemented!();
          }
        },
      })
    })
    .filter_map(|x| x)
    .collect();

  quote! {
    use xml::writer::XmlEvent;

    impl YaSerialize for #name {
      #[allow(unused_variables)]
      fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>)
        -> Result<(), String> {
        let skip = writer.skip_start_end();

        if !skip {
          if let Some(label) = writer.get_start_event_name() {
            let struct_start_event = XmlEvent::start_element(label.as_ref());
            #build_attributes
            let _ret = writer.write(struct_start_event);
          } else {
            let struct_start_event = XmlEvent::start_element(#root)#add_namespaces;
            #build_attributes
            let _ret = writer.write(struct_start_event);
          }
        }

        #struct_inspector

        if !skip {
          let struct_end_event = XmlEvent::end_element();
          let _ret = writer.write(struct_end_event);
        }

        Ok(())
      }
    }
  }
}

fn build_label_name(field: &syn::Field, field_attrs: &YaSerdeAttribute) -> String {
  format!(
    "{}{}",
    field_attrs
      .prefix
      .clone()
      .map_or("".to_string(), |prefix| prefix + ":"),
    field_attrs
      .rename
      .clone()
      .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
  )
}
