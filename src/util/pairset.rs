use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::ptr::null;

struct SetPair<A, B>(A, B, (*const A, *const B))
where
A : Ord,
B : Ord;

impl<A, B> Borrow<(*const A, *const B)> for SetPair<A, B>
where
A : Ord,
B : Ord {
    fn borrow(&self) -> &(*const A, *const B) {
        &self.2
    }
}

impl<A, B> PartialEq for SetPair<A, B>
where
A : Ord,
B : Ord {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<A, B> Eq for SetPair<A, B>
where
A : Ord,
B : Ord {
}

impl<A, B> PartialOrd for SetPair<A, B>
where
A : Ord,
B : Ord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.1.partial_cmp(&other.1)
    }
}

impl<A, B> Ord for SetPair<A, B>
where
A : Ord,
B : Ord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0.cmp(&other.0) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.1.cmp(&other.1)
    }
}


fn pair<A, B>(pair: (A, B)) -> SetPair<A, B>
where
A : Ord,
B : Ord {
    let mut p = SetPair(pair.0, pair.1, (null(), null()));
    p.2.0 = &p.0;
    p.2.1 = &p.1;
    p
}


pub struct PairSet<A, B>
where
A : Ord,
B : Ord {
    set: BTreeSet<SetPair<A, B>>
}

impl<A, B> PairSet<A, B>
where
A : Ord,
B : Ord {
    pub fn new() -> Self {
        Self {
            set: BTreeSet::new()
        }
    }

    pub fn insert(&mut self, pair: (A, B)) -> bool {
        self.set.insert(pair)
    }
}