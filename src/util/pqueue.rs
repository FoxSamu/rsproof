use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use crate::fmt::DisplayNamed;

/// A priority queue, which prioritises based on weight. That is, elements with the lowest weights
/// come out of the queue first. The queue is implemented by a binary heap.
pub struct PQueue<E, W>
where
W : Ord + Copy {
    heap: Vec<(E, W)>
}

/// Default implementation of a [PQueue].
impl<E, W> PQueue<E, W>
where
W : Ord + Copy {

    /// Creates a new priority queue.
    pub fn new() -> Self {
        Self {
            heap: Vec::new()
        }
    }

    /// Creates a new priority queue from the elements in the given collection or iterator,
    /// by weighing them using a specific scale function. The scale function assigns a weight
    /// to each element in the collection.
    /// 
    /// The total operation takes `O(N)` if there are `N` elements in the iterator.
    pub fn assoc<I, F>(iter: I, mut scale: F) -> Self
    where
    I : IntoIterator<Item = E>,
    F : FnMut(&mut E) -> W {
        Self::from_iter(iter.into_iter().map(|mut e| {
            let w = scale(&mut e);
            (e, w)
        }))
    }

    /// Reassociates elements using the given scale function. Unlike [PQueue::assoc], the scale
    /// function of [PQueue::reassoc] also receives the old weight of each element.
    /// 
    /// The total operation takes `O(N)` if there are `N` elements in the queue.
    pub fn reassoc<F>(&mut self, mut scale: F)
    where
    F : FnMut(&mut E, W) -> W {
        let mut heap = std::mem::take(&mut self.heap);
        heap = heap.into_iter().map(|(mut e, mut w)| {
            w = scale(&mut e, w);
            (e, w)
        }).collect();
        self.heap = heap;

        self.restore();
    }

    /// Returns whether the queue is empty, that is, whether it contains no elements.
    /// 
    /// This operation runs in `O(1)`.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Returns the amount of elements in the queue.
    /// 
    /// This operation runs in `O(1)`.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Retrieves, but does not remove, the first element in the queue and its weight.
    /// When the queue is empty, [None] is returned.
    /// 
    /// This operation runs in `O(1)`.
    pub fn peek(&self) -> Option<(&E, &W)> {
        self.heap.first().map(|it| (&it.0, &it.1))
    }

    /// Retreves, but does not remove, the first element in the queue, ignoring its weight.
    /// When the queue is empty, [None] is returned.
    /// 
    /// This operation runs in `O(1)`.
    pub fn peek_elem(&self) -> Option<&E> {
        self.heap.first().map(|it| &it.0)
    }

    /// Inserts a new element into the queue using the given weight.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn insert(&mut self, elem: E, weight: W) {
        let i = self.heap.len();

        // Add element at end
        self.heap.push((elem, weight));

        // Sift up
        self.upheap(i);
    }

    /// Removes the first element in the queue, and its weight.
    /// When the queue is empty, [None] is returned.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn poll(&mut self) -> Option<(E, W)> {
        // Extract last element
        let mut elem = self.heap.pop()?;

        let first = if let Some(r) = self.heap.first_mut() {
            r
        } else {
            // Queue is empty now, so the element we removed was the only
            // element that was there
            return Some(elem);
        };

        // Swap last element with first element, so we now have extracted
        // the first element and the last element is in place of the first
        std::mem::swap(first, &mut elem);

        // Sift element down
        self.downheap(0);

        Some(elem)
    }

    /// Removes the first element in the queue, ignoring its weight.
    /// When the queue is empty, [None] is returned.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn poll_elem(&mut self) -> Option<E> {
        self.poll().map(|it| it.0)
    }

    /// Removes the first element in the queue while simultaneously inserting
    /// another element. Returns the removed element and its weight.
    /// When the queue is empty, [None] is returned and only the new element
    /// is inserted.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn poll_insert(&mut self, elem: E, weight: W) -> Option<(E, W)> {
        // New element
        let mut elem = (elem, weight);

        let first = if let Some(r) = self.heap.first_mut() {
            r
        } else {
            // Queue is empty, so all we need to do is insert the element
            self.heap.push(elem);
            return None;
        };

        // Swap out the first item in the queue with the new element
        std::mem::swap(first, &mut elem);

        // Sift element down
        self.downheap(0);

        Some(elem)
    }

    /// Removes the first element in the queue while simultaneously inserting
    /// another element. Returns the removed element, without its weight.
    /// When the queue is empty, [None] is returned and only the new element
    /// is inserted.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn poll_elem_insert(&mut self, elem: E, weight: W) -> Option<E> {
        self.poll_insert(elem, weight).map(|it| it.0)
    }

    /// Removes all elements from this queue.
    /// 
    /// This operation runs in `O(1)`.
    pub fn clear(&mut self) {
        self.heap.clear();
    }

    fn upheap(&mut self, mut i: usize) {
        let mut p = parent(i);

        while i != 0 && self.heap[p].1 > self.heap[i].1 {
            self.heap.swap(p, i);

            i = p;
            p = parent(i);
        }
    }

    fn downheap(&mut self, mut i: usize) {
        loop {
            let l = left(i);
            let r = right(i);

            let left = self.heap.get(l);
            let right = self.heap.get(r);
            let cur = &self.heap[i];

            match (left, right) {
                (Some(l_elem), Some(r_elem)) => {
                    if l_elem.1 >= cur.1 && r_elem.1 >= cur.1 {
                        // Heap property is restored
                        break;
                    }

                    if l_elem.1 < r_elem.1 {
                        self.heap.swap(l, i);
                        i = l;
                    } else {
                        self.heap.swap(r, i);
                        i = r;
                    }
                },

                (None, Some(r_elem)) => {
                    if r_elem.1 < cur.1 {
                        self.heap.swap(r, i);
                        i = r;
                    } else {
                        // Heap property is restored
                        break;
                    }
                },

                (Some(l_elem), None) => {
                    if l_elem.1 < cur.1 {
                        self.heap.swap(l, i);
                        i = l;
                    } else {
                        // Heap property is restored
                        break;
                    }
                },

                (None, None) => {
                    // We're in a leaf node now
                    break
                }
            }
        }
    }

    /// When all hope is lost, this function restores the heap property brutally. It runs in
    /// `O(N)` for a heap with `N` elements.
    fn restore(&mut self) {
        if self.len() <= 1 {
            return;
        }

        let mut i = self.heap.len() / 2;

        while i > 0 {
            self.downheap(i);
            i -= 1;
        }

        self.downheap(0);
    }


    /// Returns an iterator that iterates the **elements and weights** in this queue in sorted order.
    /// The creation of the iterator takes `O(N)` over a queue of `N` elements and the
    /// retrieval of an element from this iterator takes `O(log N)`. To retrieve an iterator
    /// in `O(1)`, use [PQueue::into_iter].
    pub fn iter<'lt>(&'lt self) -> Iter<'lt, E, W> {
        let q = PQueue {
            heap: self.heap.iter().map(|it| (it, it.1)).collect()
        };

        Iter { q }
    }


    /// Returns an iterator that iterates **only the elements** in this queue in sorted order.
    /// The creation of the iterator takes `O(N)` over a queue of `N` elements and the
    /// retrieval of an element from this iterator takes `O(log N)`. To retrieve an iterator
    /// in `O(1)`, use [PQueue::into_iter_elem].
    pub fn iter_elem<'lt>(&'lt self) -> ElemIter<'lt, E, W> {
        let q = PQueue {
            heap: self.heap.iter().map(|it| (&it.0, it.1)).collect()
        };

        ElemIter { q }
    }


    /// Returns an iterator that iterates **only the elements**
    /// of this queue. The priority queue is moved and consumed, allowing
    /// for a `O(1)` iterator setup. The retrieval of an element takes `O(log N)`
    /// for a queue of `N` elements.
    pub fn into_iter_elem(self) -> IntoElemIter<E, W> {
        IntoElemIter { q: self }
    }
}


/// Implementation for [PQueue]s whose elements are [Weighted], i.e. they have a default weight.
/// A few additional methods provide the option to insert elements by their default weight.
impl<E, W> PQueue<E, W>
where
E : Weighted<W>,
W : Ord + Copy {
    /// Creates a new priority queue from the elements in the given collection or iterator,
    /// by using their default weights given by [Weighted].
    pub fn assoc_elem<I, F>(iter: I) -> Self
    where
    I : IntoIterator<Item = E> {
        Self::from_iter(iter.into_iter().map(|e| {
            let w = e.weight();
            (e, w)
        }))
    }

    /// Reassociates elements by their default weights given by [Weighted].
    /// 
    /// The total operation takes `O(N)` if there are `N` elements in the queue.
    pub fn reassoc_elem<F>(&mut self) {
        self.reassoc_modify(|e| e);
    }

    /// Reassociates elements by their default weights given by [Weighted], after
    /// modifying each element using a modifier function.
    /// 
    /// The total operation takes `O(N)` if there are `N` elements in the queue.
    pub fn reassoc_modify<F>(&mut self, mut modifier: F)
    where
    F : FnMut(E) -> E {
        let mut heap = std::mem::take(&mut self.heap);
        heap = heap.into_iter().map(|(mut e, _)| {
            e = modifier(e);

            let w = e.weight();
            (e, w)
        }).collect();
        self.heap = heap;

        self.restore();
    }

    /// Inserts a new element into the queue using the default weight
    /// of the [Weighted] element.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn insert_elem(&mut self, elem: E) {
        let w = elem.weight();
        self.insert(elem, w);
    }

    /// Removes the first element in the queue while simultaneously inserting
    /// another element. Returns the removed element and its weight.
    /// When the queue is empty, [None] is returned and only the new element
    /// is inserted.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn poll_insert_elem(&mut self, elem: E) -> Option<(E, W)> {
        let w = elem.weight();
        self.poll_insert(elem, w)
    }

    /// Removes the first element in the queue while simultaneously inserting
    /// another element. Returns the removed element, without its weight.
    /// When the queue is empty, [None] is returned and only the new element
    /// is inserted.
    /// 
    /// This operation runs in `O(log N)` in a queue of size `N`.
    pub fn poll_elem_insert_elem(&mut self, elem: E) -> Option<E> {
        let w = elem.weight();
        self.poll_elem_insert(elem, w)
    }
}


#[inline]
const fn parent(i: usize) -> usize {
    i / 2
}

#[inline]
const fn left(i: usize) -> usize {
    i * 2 + 1
}

#[inline]
const fn right(i: usize) -> usize {
    i * 2 + 2
}



impl<E, W> FromIterator<(E, W)> for PQueue<E, W>
where
W : Ord + Copy {
    /// Creates a new queue with the elements of this iterator.
    fn from_iter<T: IntoIterator<Item = (E, W)>>(iter: T) -> Self {
        let mut new = Self {
            heap: iter.into_iter().collect()
        };

        new.restore();

        new
    }
}


/// Generated by [PQueue::into_iter].
pub struct IntoIter<E, W>
where
W : Ord + Copy {
    q: PQueue<E, W>
}

impl<E, W> Iterator for IntoIter<E, W>
where
W : Ord + Copy {
    type Item = (E, W);

    fn next(&mut self) -> Option<Self::Item> {
        self.q.poll()
    }
}

impl<E, W> IntoIterator for PQueue<E, W>
where
W : Ord + Copy {
    type Item = (E, W);

    type IntoIter = IntoIter<E, W>;

    /// Returns an iterator that iterates the **elements and their weights**
    /// of this queue. The priority queue is moved and consumed, allowing
    /// for a `O(1)` iterator setup. The retrieval of an element takes `O(log N)`
    /// for a queue of `N` elements.
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { q: self }
    }
}


pub struct Iter<'lt, E, W>
where
W : Ord + Copy {
    q: PQueue<&'lt (E, W), W>
}


impl<'lt, E, W> Iterator for Iter<'lt, E, W>
where
W : Ord + Copy {
    type Item = &'lt (E, W);

    fn next(&mut self) -> Option<Self::Item> {
        self.q.poll_elem()
    }
}


pub struct IntoElemIter<E, W>
where
W : Ord + Copy {
    q: PQueue<E, W>
}

impl<E, W> Iterator for IntoElemIter<E, W>
where
W : Ord + Copy {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        self.q.poll_elem()
    }
}


pub struct ElemIter<'lt, E, W>
where
W : Ord + Copy {
    q: PQueue<&'lt E, W>
}


impl<'lt, E, W> Iterator for ElemIter<'lt, E, W>
where
W : Ord + Copy {
    type Item = &'lt E;

    fn next(&mut self) -> Option<Self::Item> {
        self.q.poll_elem()
    }
}


impl<E, W> DisplayNamed for PQueue<E, W>
where
E : DisplayNamed,
W : DisplayNamed + Ord + Copy {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        DisplayNamed::fmt_named(&self.heap, f, names)
    }
}

impl<E, W> Debug for PQueue<E, W>
where
E : Debug,
W : Debug + Ord + Copy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.heap, f)
    }
}

impl<E, W> PartialEq for PQueue<E, W>
where
E : PartialEq,
W : Ord + Copy { // Ord => PartialEq
    fn eq(&self, other: &Self) -> bool {
        // The final queue order may be the same while
        // the heaps have a different structure, so we need
        // to sort the elements first.
        let sv = self.iter().collect::<Vec<_>>();
        let ov = other.iter().collect::<Vec<_>>();
        sv == ov
    }
}

impl<E, W> Eq for PQueue<E, W>
where
E : Eq,
W : Ord + Copy { // Ord => Eq
}

impl<E, W> PartialOrd for PQueue<E, W>
where
E : PartialOrd,
W : Ord + Copy { // Ord => PartialOrd
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // See comment in PartialEq implementation
        let sv = self.iter().collect::<Vec<_>>();
        let ov = other.iter().collect::<Vec<_>>();
        sv.partial_cmp(&ov)
    }
}

impl<E, W> Ord for PQueue<E, W>
where
E : Ord,
W : Ord + Copy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // See comment in PartialEq implementation
        let sv = self.iter().collect::<Vec<_>>();
        let ov = other.iter().collect::<Vec<_>>();
        sv.cmp(&ov)
    }
}

impl<E, W> Clone for PQueue<E, W>
where
E : Clone,
W : Clone + Ord + Copy {
    fn clone(&self) -> Self {
        // Cloning the heap should preserve the heap property
        Self { heap: self.heap.clone() }
    }
}

impl<E, W> Hash for PQueue<E, W>
where
E : Hash,
W : Hash + Ord + Copy {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // See comment in PartialEq implementation.
        let sv = self.iter().collect::<Vec<_>>();
        sv.hash(state);
    }
}

impl<E, W> Default for PQueue<E, W>
where
W : Ord + Copy {
    fn default() -> Self {
        // Default is an empty queue
        Self { heap: vec![] }
    }
}


/// A trait for elements of a [PQueue] that can assign themselves a weight.
/// The [PQueue] will use this in functions like [PQueue::insert_elem] to
/// determine the weight of the element. The weight will be computed once
/// and the queue will then keep the element under that weight.
pub trait Weighted<W> where W : Ord + Copy {
    /// Computes the weight of this value. When an element gets inserted into
    /// a [PQueue], this function is called once, after which the element stays
    /// in the queue with the returned weight.
    fn weight(&self) -> W;
}

impl<'lt, E, W> Weighted<W> for &'lt E where W : Ord + Copy, E : Weighted<W> {
    fn weight(&self) -> W {
        Weighted::weight(*self)
    }
}

impl<E, W> Weighted<W> for Box<E> where W : Ord + Copy, E : Weighted<W> {
    fn weight(&self) -> W {
        Weighted::weight(self.as_ref())
    }
}

impl<E, W> Weighted<W> for Rc<E> where W : Ord + Copy, E : Weighted<W> {
    fn weight(&self) -> W {
        Weighted::weight(self.as_ref())
    }
}

impl Weighted<usize> for usize {
    fn weight(&self) -> usize {
        *self
    }
}

impl Weighted<u128> for u128 {
    fn weight(&self) -> u128 {
        *self
    }
}

impl Weighted<u64> for u64 {
    fn weight(&self) -> u64 {
        *self
    }
}

impl Weighted<u32> for u32 {
    fn weight(&self) -> u32 {
        *self
    }
}

impl Weighted<u16> for u16 {
    fn weight(&self) -> u16 {
        *self
    }
}

impl Weighted<u8> for u8 {
    fn weight(&self) -> u8 {
        *self
    }
}

impl Weighted<isize> for isize {
    fn weight(&self) -> isize {
        *self
    }
}

impl Weighted<i128> for i128 {
    fn weight(&self) -> i128 {
        *self
    }
}

impl Weighted<i64> for i64 {
    fn weight(&self) -> i64 {
        *self
    }
}

impl Weighted<i32> for i32 {
    fn weight(&self) -> i32 {
        *self
    }
}

impl Weighted<i16> for i16 {
    fn weight(&self) -> i16 {
        *self
    }
}

impl Weighted<i8> for i8 {
    fn weight(&self) -> i8 {
        *self
    }
}
