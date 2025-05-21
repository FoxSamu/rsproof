pub struct Max<N> where N : Ord {
    pub result: Option<N>
}

impl<N> FromIterator<N> for Max<N> where N : Ord {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Self { result: iter.into_iter().max() }
    }
}


pub struct Min<N> where N : Ord {
    pub result: Option<N>
}

impl<N> FromIterator<N> for Min<N> where N : Ord {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Self { result: iter.into_iter().min() }
    }
}