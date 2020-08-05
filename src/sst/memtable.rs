use std::collections::{BTreeMap, BTreeSet};

pub trait Memtable {
    type Key;
    type Value;
    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn set(&mut self, key: Self::Key, value: Self::Value) -> ();
    fn delete(&mut self, key: Self::Key) -> ();
    fn flush(
        &mut self,
    ) -> (
        Box<BTreeMap<Self::Key, Self::Value>>,
        Box<BTreeSet<Self::Key>>,
    );
}

pub mod default {
    use super::Memtable;
    use std::{
        collections::{BTreeMap, BTreeSet},
        hash::Hash,
    };

    pub struct HashMemtable<K, V> {
        underlying: BTreeMap<K, V>,
        tombstone: BTreeSet<K>,
    }
    impl<K: Hash + Eq + Ord, V> HashMemtable<K, V> {
        pub fn new() -> impl Memtable<Key = K, Value = V> {
            HashMemtable {
                underlying: BTreeMap::new(),
                tombstone: BTreeSet::new(),
            }
        }

        fn is_deleted(&self, key: &K) -> bool {
            self.tombstone.get(key).is_some()
        }

        fn with_check_tombstone<T>(
            &self,
            key: &K,
            deleted: impl Fn() -> T,
            not_deleted: impl Fn() -> T,
        ) -> T {
            if self.is_deleted(key) {
                deleted()
            } else {
                not_deleted()
            }
        }
    }
    impl<K: Hash + Eq + Ord, V> Memtable for HashMemtable<K, V> {
        type Key = K;
        type Value = V;

        fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
            self.with_check_tombstone(key, || None, || self.underlying.get(&key))
        }

        fn set(&mut self, key: Self::Key, value: Self::Value) -> () {
            self.tombstone.remove(&key);
            self.underlying.insert(key, value);
        }

        fn delete(&mut self, key: Self::Key) -> () {
            self.underlying.remove(&key);
            self.tombstone.insert(key);
        }

        fn flush(
            &mut self,
        ) -> (
            Box<BTreeMap<Self::Key, Self::Value>>,
            Box<BTreeSet<Self::Key>>,
        ) {
            // let contents = std::mem::replace(&mut self.underlying, BTreeMap::new());
            // let deleted = std::mem::replace(&mut self.tombstone, BTreeSet::new());
            // (Box::new(contents.iter()), Box::new(deleted.iter()))
            (
                Box::new(std::mem::replace(&mut self.underlying, BTreeMap::new())),
                Box::new(std::mem::replace(&mut self.tombstone, BTreeSet::new())),
            )
        }
    }
}