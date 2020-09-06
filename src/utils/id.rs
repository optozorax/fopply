use core::hash::Hash;
use std::collections::HashMap;

#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct IdGenerator<T: Hash + Eq> {
	storage: HashMap<T, u32>,
	counter: u32,
}

impl<T: Hash + Eq> IdGenerator<T> {
	pub fn get_or_add_id(&mut self, key: T) -> u32 {
		let storage = &mut self.storage;
		let counter = &mut self.counter;
		*storage.entry(key).or_insert_with(|| {
			*counter += 1; 
			*counter
		})
	}

	pub fn get_id(&self, key: &T) -> Option<u32> {
		self.storage.get(key).cloned()
	}

	pub fn get_key(&self, value: u32) -> Option<&T> {
		self.storage.iter().find(|(_, v)| **v == value).map(|(k, _)| k)
	}

	pub fn get_hash_map(&self) -> &HashMap<T, u32> {
		&self.storage
	}
}