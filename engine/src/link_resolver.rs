// Resolves Links between objects

use object::{
    Key,
    types::{
        link::Link,
        list::{List, ListBuilder},
        map::{Map, MapBuilder},
    },
};
use server::LinkResolution;
use std::collections::HashSet;

/// Manages link resolution
pub struct LinkResolver<'a, T: storage::Store> {
    /// The max number of resolutions that can occur
    max_resolution_depth: u8,
    /// Links that have already been visited, used for cycle detection
    visited: HashSet<Key>,
    /// The storage medium used
    storage: &'a T,
}

impl<'a, T: storage::Store> LinkResolver<'a, T> {
    /// Creates a new LinkResolver
    pub fn new(link_resolutions: LinkResolution, storage: &'a T) -> Self {
        Self {
            max_resolution_depth: link_resolutions.max_resolution_depth(),
            visited: HashSet::new(),
            storage,
        }
    }

    /// Resolves the links of an object and returns a new Object with the Links resolved
    /// It requires a reference to the storage to resolve the links. If max resolutions is
    /// reached or a cycle is detected the object as is will be returend
    // TODO: should cycles cause an error?
    pub fn resolve(&mut self, object: object::Object) -> storage::StoreResult {
        // The resolution depth so far
        // This cannot be a u8 since then it could never be greater than max_resolution_depth
        let resolution_depth = 0u16;

        self.resolve_links(object, resolution_depth)
    }

    fn resolve_links(
        &mut self,
        object: object::Object,
        resolution_depth: u16,
    ) -> storage::StoreResult {
        if resolution_depth > self.max_resolution_depth as u16 {
            return Ok(object);
        }
        let resolution_depth = resolution_depth + 1;

        match object.kind() {
            object::ObjectKind::List => self.resolve_list_links(object, resolution_depth),
            object::ObjectKind::Map => self.resolve_map_links(object, resolution_depth),
            object::ObjectKind::Link => self.resolve_link(object, resolution_depth),
            // Int, Text, Null cannot contain links
            _ => Ok(object),
        }
    }

    /// Resolve links for a List
    fn resolve_list_links(
        &mut self,
        object: object::Object,
        resolution_depth: u16,
    ) -> storage::StoreResult {
        // This is safe since the only way a list could get stored is if it is valid
        // if something happend that made it not valid then we have a bigger problem
        let list = unsafe { List::from_object_unchecked(object) };
        let mut builder = ListBuilder::new(list.len());

        for object in list {
            let object = self.resolve_links(object, resolution_depth)?;
            builder.add_item_no_increment(object);
        }

        Ok(builder.build().into())
    }

    /// Resolve links for a Map
    fn resolve_map_links(
        &mut self,
        object: object::Object,
        resolution_depth: u16,
    ) -> storage::StoreResult {
        // This is safe since the only way a list could get stored is if it is valid
        // if something happend that made it not valid then we have a bigger problem
        let map = unsafe { Map::from_object_unchecked(object) };
        let mut builder = MapBuilder::new(map.num_fields());

        for (field_name, object) in map {
            builder.add_field_no_increment(
                field_name.as_ref(),
                self.resolve_links(object, resolution_depth)?,
            );
        }

        Ok(builder.build().into())
    }

    /// Resolve links for a Link
    fn resolve_link(
        &mut self,
        object: object::Object,
        resolution_depth: u16,
    ) -> storage::StoreResult {
        // This is safe since the only way a link could get stored is if it is valid
        // if something happend that made it not valid then we have a bigger problem
        let link = unsafe { Link::from_object_unchecked(object) };
        let key = link.into();

        // I know there are a lot of .into()'s but they are zero cost... at least I think
        if self.visited.contains(&key) {
            // cycle detected early return
            let link: Link = key.into();
            return Ok(link.into());
        }

        let object = self.storage.retrieve(&key)?;
        self.visited.insert(key);

        // the object retrieve may also have links that need to be resolved
        self.resolve_links(object, resolution_depth)
    }
}
