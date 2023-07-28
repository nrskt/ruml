use quote::quote;

use super::{Entity, EntityType};

pub fn file_parser(file: syn::File) -> Vec<Entity> {
    let mut entities: Vec<Entity> = vec![];

    // First pass: create entities for structs
    for item in &file.items {
        if let syn::Item::Struct(item) = item {
            entities.push(struct_parser(item.clone()));
        }
    }

    // Second pass: add methods to entities
    for item in &file.items {
        if let syn::Item::Impl(impl_item) = item {
            if impl_item.trait_.is_some() {
                continue; // Skip trait implementations for now
            }
            let struct_name = if let syn::Type::Path(type_path) = *impl_item.self_ty.clone() {
                type_path.path.segments.last().unwrap().ident.to_string()
            } else {
                continue;
            };
            let methods = impl_parser(impl_item.clone());
            if let Some(entity) = entities.iter_mut().find(|e| e.name == struct_name) {
                entity.fields.extend(methods);
            }
        }
    }

    entities
}

fn struct_parser(item: syn::ItemStruct) -> Entity {
    let name = item.ident.to_string();
    let fields = fields_parser(item.fields);
    Entity {
        entity_type: EntityType::Struct,
        name,
        fields,
    }
}

fn fields_parser(item: syn::Fields) -> Vec<Entity> {
    match item {
        syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
            fields.into_iter().map(field_parser).collect()
        }
        _ => vec![],
    }
}

fn impl_parser(impl_item: syn::ItemImpl) -> Vec<Entity> {

    impl_item.items.into_iter().filter_map(|item| {
        if let syn::ImplItem::Method(method) = item {
            let method_name = method.sig.ident.to_string();
            let parameters = method.sig.inputs.into_iter().filter_map(|input| {
                match input {
                    syn::FnArg::Typed(pat_type) => {
                        let parameter_name = match *pat_type.pat {
                            syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                            _ => return None,
                        };
                        let parameter_type = pat_type.ty;
                        let parameter_type_string = quote!(#parameter_type).to_string();
                        Some(Entity {
                            entity_type: EntityType::Parameter(parameter_name),
                            name: parameter_type_string,
                            fields: vec![],
                        })
                    }
                    _ => None,
                }
            }).collect();
            Some(Entity {
                entity_type: EntityType::Method(method_name),
                name: method.sig.ident.to_string(),
                fields: parameters,
            })
        } else {
            None
        }
    }).collect()
}


fn field_parser(field: syn::Field) -> Entity {
    let name = field
        .ident
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| "".to_string());

    if has_dependencies(&type_parser(field.ty.clone())) {
        let fields = make_dependencies(&type_parser(field.ty.clone()));
        return Entity::new(
            EntityType::Field(name),
            &type_parser(field.ty),
            vec![fields],
        );
    }
    Entity::new(EntityType::Field(name), &type_parser(field.ty), Vec::new())
}

fn type_parser(type_: syn::Type) -> String {
    let type_name = quote!(#type_);
    type_name
        .to_string()
        .chars()
        .filter(|c| *c != ' ')
        .collect()
}

fn has_dependencies(type_name: &str) -> bool {
    let cnt = type_name
        .chars()
        .filter(|x| (*x == ',') || (*x == '<') || (*x == '>'))
        .count();
    if cnt != 0 {
        return true;
    }
    false
}

fn make_dependencies(type_name: &str) -> Entity {
    let dependencies: Vec<&str> = type_name
        .split(|x| (x == ',') || (x == '<') || (x == '>'))
        .collect();
    let dependencies = dependencies
        .into_iter()
        .map(|x| x.to_string())
        .filter(|x| x != "")
        .map(|x| x.replace(" ", ""))
        .collect::<Vec<String>>();
    let dependencies = dependencies
        .into_iter()
        .map(|x| Entity::new(EntityType::Field("".to_string()), &x, Vec::new()))
        .collect::<Vec<Entity>>();
    Entity::new(EntityType::Struct, type_name, dependencies)
}
