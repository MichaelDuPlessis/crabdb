// Resolves Links between objects

use object::types::{link::Link, list::List, map::Map};

// TODO: I personally think there are a lot of things that can be done better here.
// there are many memory allocations and I am still deciding if I like how
// this code is structured but if it works for now its fine.

/// Resolves the links of an object and returns a new Object with the Links resolved
/// It requires a reference to the storage to resolve the links
pub fn resolve_links<T: storage::Store>(
    object: object::Object,
    storage: &T,
) -> storage::StoreResult {
    match object.kind() {
        object::ObjectKind::List => resolve_list_links(object, storage),
        object::ObjectKind::Map => resolve_map_links(object, storage),
        object::ObjectKind::Link => resolve_link(object, storage),
        // Int, Text, Null cannot contain links
        _ => Ok(object),
    }
}

/// Resolve links for a List
fn resolve_list_links<T: storage::Store>(
    object: object::Object,
    storage: &T,
) -> storage::StoreResult {
    // This is safe since the only way a list could get stored is if it is valid
    // if something happend that made it not valid then we have a bigger problem
    let list = unsafe { List::from_object_unchecked(object) };
    let mut objects = Vec::with_capacity(list.len() as usize);

    for object in list {
        let object = resolve_links(object, storage)?;
        objects.push(object);
    }

    Ok(List::from(objects.as_slice()).into())
}

/// Resolve links for a Map
fn resolve_map_links<T: storage::Store>(
    object: object::Object,
    storage: &T,
) -> storage::StoreResult {
    // This is safe since the only way a list could get stored is if it is valid
    // if something happend that made it not valid then we have a bigger problem
    let map = unsafe { Map::from_object_unchecked(object) };
    let mut records = Vec::with_capacity(map.num_fields() as usize);

    for (field_name, object) in map {
        records.push((field_name, resolve_links(object, storage)?));
    }

    Ok(Map::from(records.as_slice()).into())
}

/// Resolve links for a Link
fn resolve_link<T: storage::Store>(object: object::Object, storage: &T) -> storage::StoreResult {
    // This is safe since the only way a link could get stored is if it is valid
    // if something happend that made it not valid then we have a bigger problem
    let link = unsafe { Link::from_object_unchecked(object) };
    let key = link.into();

    let object = storage.retrieve(key)?;
    // the object retrieve may also have links that need to be resolved
    resolve_links(object, storage)
}
