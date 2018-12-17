use std::cmp::{Ord, Ordering::*};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use nohash_hasher::{BuildNoHashHasher, IntMap};

pub struct PerfectHasher<C, H> {
    // Key is the Id
    alloted: IntMap<u32, C>,
    hasher: H,
}

impl<C, H> PerfectHasher<C, H>
where
    H: Hasher,
    C: Hash + Ord,
{
    pub fn new(hasher: H) -> Self {
        PerfectHasher {
            alloted: IntMap::default(),
            hasher,
        }
    }

    pub fn with_capacity(capacity: u32, hasher: H) -> Self {
        PerfectHasher {
            alloted: HashMap::with_capacity_and_hasher(
                capacity as usize,
                BuildNoHashHasher::default(),
            ),
            hasher,
        }
    }

    pub fn unique_id(&mut self, content: C) -> u32 {
        content.hash(&mut self.hasher);
        let mut hash = self.hasher.finish() as u32;

        let mut comparison = Equal;

        loop {
            let entry = self
                .alloted
                .entry(hash)
                .and_modify(|cached| comparison = (*cached).cmp(&content));

            match comparison {
                Equal => {
                    entry.or_insert(content);
                    return hash;
                }
                Less => hash += 1,
                Greater => hash -= 1,
            }
        }
    }

    pub fn dissociate(&mut self, id: u32) {
        self.alloted.remove(&id);
    }
}
