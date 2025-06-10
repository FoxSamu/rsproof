use rsplib::util::pqueue::PQueue;

#[test]
fn random_1() {
    let queue = PQueue::assoc([3, 1, 2, 4, 0], |it| *it);
    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    
    assert_eq!(vec, vec![0, 1, 2, 3, 4]);
}

#[test]
fn empty() {
    let mut queue = PQueue::<u64, u64>::new();

    assert_eq!(None, queue.poll());
}

#[test]
fn poll() {
    let mut queue = PQueue::assoc([81, 1, 2, 3], |it| *it);

    assert_eq!(Some((1, 1)), queue.poll());
    assert_eq!(Some((2, 2)), queue.poll());
    assert_eq!(Some((3, 3)), queue.poll());
    assert_eq!(Some((81, 81)), queue.poll());
}

#[test]
fn peek() {
    let queue = PQueue::assoc([81, 1, 2, 3], |it| *it);

    assert_eq!(Some((&1, &1)), queue.peek());
    assert_eq!(Some((&1, &1)), queue.peek());
    assert_eq!(Some((&1, &1)), queue.peek());
}

#[test]
fn poll_integrity() {
    let mut queue = PQueue::assoc([3, 1, 2, 4, 0], |it| *it);

    assert_eq!(Some((0, 0)), queue.poll());

    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    assert_eq!(vec, vec![1, 2, 3, 4]);
}

#[test]
fn peek_integrity() {
    let queue = PQueue::assoc([3, 1, 2, 4, 0], |it| *it);

    assert_eq!(Some((&0, &0)), queue.peek());

    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    assert_eq!(vec, vec![0, 1, 2, 3, 4]);
}

#[test]
fn insert() {
    let mut queue = PQueue::assoc([3, 1, 2, 4], |it| *it);

    queue.insert(0, 0);

    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    assert_eq!(vec, vec![0, 1, 2, 3, 4]);
}

#[test]
fn insert_heavy_weight() {
    let mut queue = PQueue::assoc([3, 1, 2, 4], |it| *it);

    queue.insert(0, 19);

    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    assert_eq!(vec, vec![1, 2, 3, 4, 0]);
}

#[test]
fn insert_mid_weight() {
    let mut queue = PQueue::assoc([3, 1, 5, 4], |it| *it);

    queue.insert(0, 2);

    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    assert_eq!(vec, vec![1, 0, 3, 4, 5]);
}

#[test]
fn clear() {
    let mut queue = PQueue::assoc([3, 1, 5, 4], |it| *it);

    queue.clear();

    let vec = queue.into_iter().map(|it| it.0).collect::<Vec<_>>();
    assert_eq!(vec, vec![]);
}