mod memory;
mod sleddb;

pub use sleddb::SledDb;
pub use memory::MemTable;
use crate::{KvError, Kvpair, Value};

/// This is abstract of storage.
pub trait Storage {
    /// Get a value of the key from HashTable
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    /// Update a value of the key from HashTable, and return the old value
    fn set(&self, table: &str, key: impl Into<String>, value: impl Into<Value>) -> Result<Option<Value>, KvError>;
    /// Check if the key contains in the HashTable
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;
    /// Delete a key from HashTable
    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    /// Return all kv pairs from HashTable (bad trait)
    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError>;
    /// Return kv pairs' Iterator of HashTable
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item=Kvpair>>, KvError>;
}

/// Provide Storage iterator so that implements of traits only need to provide their iterator
/// to the StorageIter, and then they make sure that the type passed out by next() implements Into.
pub struct StorageIter<T> {
    data: T,
}

impl<T> StorageIter<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Iterator for StorageIter<T>
    where T: Iterator,
          T::Item: Into<Kvpair>,
{
    type Item = Kvpair;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|v| v.into())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, tempfile};
    use super::*;


    #[test]
    fn sleddb_basic_interface_should_work() {
        let dir = tempdir().unwrap();
        let store = SledDb::new(dir);
        test_basic_interface(store);
    }

    #[test]
    fn sleddb_get_all_should_work() {
        let dir = tempdir().unwrap();
        let store = SledDb::new(dir);
        test_get_all(store);
    }

    #[test]
    fn sleddb_iter_should_work() {
        let dir = tempdir().unwrap();
        let store = SledDb::new(dir);
        test_get_iter(store);
    }

    #[test]
    fn memtable_basic_interface_should_work() {
        let store = MemTable::new();
        test_basic_interface(store);
    }

    #[test]
    fn memtable_get_all_should_work() {
        let store = MemTable::new();
        test_get_all(store);
    }

    #[test]
    fn memtable_iter_should_work() {
        let store = MemTable::new();
        test_get_iter(store);
    }

    fn test_basic_interface(store: impl Storage) {
        // The first set creates the table, inserts the key and returns None.
        let v = store.set("t1", "hello", "world");
        assert!(v.unwrap().is_none());
        // Setting the same key again updates it and returns the previous value.
        let v1 = store.set("t1", "hello", "world1");
        assert_eq!(v1, Ok(Some("world".into())));

        // get The key that exists gets the latest value
        let v = store.get("t1", "hello");
        assert_eq!(v, Ok(Some("world1".into())));

        // get The key that is not exists gets None
        assert_eq!(Ok(None), store.get("t1", "hello1"));
        assert!(store.get("t2", "hello1").unwrap().is_none());

        // contains returns true if the key exists, otherwise false
        assert_eq!(store.contains("t1", "hello"), Ok(true));
        assert_eq!(store.contains("t1", "hello1"), Ok(false));
        assert_eq!(store.contains("t2", "hello"), Ok(false));

        // del the existing key to return the previous value
        let v = store.del("t1", "hello");
        assert_eq!(v, Ok(Some("world1".into())));

        // del non-existent key or table Returns None
        assert_eq!(Ok(None), store.del("t1", "hello1"));
        assert_eq!(Ok(None), store.del("t2", "hello"));
    }

    fn test_get_all(store: impl Storage) {
        store.set("t2", "k1", "v1").unwrap();
        store.set("t2", "k2", "v2").unwrap();
        let mut data = store.get_all("t2").unwrap();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            data,
            vec![
                Kvpair::new("k1", "v1".into()),
                Kvpair::new("k2", "v2".into()),
            ]
        )
    }

    #[allow(dead_code)]
    fn test_get_iter(store: impl Storage) {
        store.set("t2", "k1", "v1").unwrap();
        store.set("t2", "k2", "v2").unwrap();
        let mut data: Vec<_> = store.get_iter("t2").unwrap().collect();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            data,
            vec![
                Kvpair::new("k1", "v1".into()),
                Kvpair::new("k2", "v2".into()),
            ]
        )
    }
}