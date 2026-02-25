use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct IdQueue {
    queue: BinaryHeap<Reverse<u32>>,
}

impl IdQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn from_vec(queue: Vec<u32>) -> Self {
        Self {
            queue: queue.into_iter().map(Reverse).collect(),
        }
    }

    pub fn front(&self) -> Option<u32> {
        self.queue.peek().map(|Reverse(id)| *id)
    }

    pub fn enqueue(&mut self, id: u32) {
        self.queue.push(Reverse(id))
    }

    pub fn sorted_enqueue(&mut self, id: u32) {
        self.enqueue(id);
    }

    pub fn dequeue(&mut self) -> Option<u32> {
        self.queue.pop().map(|Reverse(id)| id)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn size(&self) -> u32 {
        self.queue.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::IdQueue;

    #[test]
    fn dequeue_returns_smallest_id_first() {
        let mut queue = IdQueue::new();
        queue.sorted_enqueue(5);
        queue.sorted_enqueue(1);
        queue.sorted_enqueue(3);

        assert_eq!(queue.front(), Some(1));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), Some(5));
        assert_eq!(queue.dequeue(), None);
    }
}
