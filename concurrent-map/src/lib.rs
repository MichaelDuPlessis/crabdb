use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::RwLock,
};

type Shard<K, V> = RwLock<HashMap<K, V>>;

/// A HashMap that can be shared between threads safely while still remaining effiecient
pub struct ConcurrentMap<K, V> {
    /// The shards in the hashmap
    shards: Vec<Shard<K, V>>,
}

impl<K, V> ConcurrentMap<K, V> {
    /// Creates a new concurrent hashmap with the specified number of shards
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);

        for _ in 0..num_shards {
            shards.push(RwLock::new(HashMap::default()));
        }

        Self { shards }
    }

    /// Gets the numbre of shards in the hashmap
    fn num_shards(&self) -> usize {
        self.shards.len()
    }
}

impl<K, V> ConcurrentMap<K, V>
where
    K: Hash + Eq,
{
    /// Gets the shard index that a key belongs to
    fn shard_index(&self, key: &K) -> usize {
        // TODO: Should the default hasher be used?
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.num_shards()
    }

    /// Gets the shard that a key belongs too
    fn shard(&self, key: &K) -> &Shard<K, V> {
        // getting the index that the key belongs in
        let shard_index = self.shard_index(&key);
        // getting shard
        let shard = unsafe { self.shards.get_unchecked(shard_index) };

        shard
    }

    /// Inserts a new element into the ConcurrentMap
    /// and returns the element was already there if one exists
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        // getting the shard
        let shard = self.shard(&key);
        // inserting the value
        let mut map = shard.write().unwrap();
        map.insert(key, value)
    }

    /// Reads an element from the map and returns None if no element is found
    /// The element retrieved is cloned
    pub fn get_cloned(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        // getting the shard
        let shard = self.shard(key);

        // getting the element
        let map = shard.read().unwrap();
        map.get(key).cloned()
    }

    /// Removes an element from the ConcurrentMap. Returns None if the element does not exist
    pub fn remove(&self, key: &K) -> Option<V> {
        // getting the shard
        let shard = self.shard(&key);
        // inserting the value
        let mut map = shard.write().unwrap();
        map.remove(key)
    }
}
