use std::collections::VecDeque;

struct Queue<T> {
    values: VecDeque<T>
}

impl <T: Default> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue { values: VecDeque::new() }
    }

    pub fn enqueue(&mut self, val: T) {
        self.values.push_back(val);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.values.pop_front()
    }

    pub fn requeue(&mut self, val: T) {
        self.values.push_front(val);
    }

    pub fn peek(&self) -> Option<&T> {
        self.values.front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a new queue
    fn nq<T: Default>() -> Queue<T> {
        Queue::new()
    }

    #[test]
    fn test_new_queue_is_empty() {
        let q: Queue<i32> = nq();
        assert!(q.peek().is_none(), "New queue should be empty");
    }

    #[test]
    fn test_enqueue_single_element() {
        let mut q = nq();
        q.enqueue(42);
        assert_eq!(q.peek(), Some(&42), "Peek should return the enqueued element");
    }

    #[test]
    fn test_enqueue_dequeue_single_element() {
        let mut q = nq();
        q.enqueue(42);
        assert_eq!(q.dequeue(), Some(42), "Dequeue should return the enqueued element");
        assert!(q.peek().is_none(), "Queue should be empty after dequeue");
    }

    #[test]
    fn test_enqueue_multiple_elements() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.peek(), Some(&1), "Peek should return first element");
    }

    #[test]
    fn test_dequeue_multiple_elements() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.dequeue(), Some(1), "First dequeue should return 1");
        assert_eq!(q.dequeue(), Some(2), "Second dequeue should return 2");
        assert_eq!(q.dequeue(), Some(3), "Third dequeue should return 3");
        assert_eq!(q.dequeue(), None, "Dequeue on empty queue should return None");
    }

    #[test]
    fn test_requeue_single_element() {
        let mut q = nq();
        q.enqueue(1);
        q.dequeue();
        q.requeue(2);
        assert_eq!(q.peek(), Some(&2), "Requeue should place element at front");
        assert_eq!(q.dequeue(), Some(2), "Dequeue should return requeued element");
    }

    #[test]
    fn test_requeue_after_enqueue() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.requeue(0);
        assert_eq!(q.peek(), Some(&0), "Requeue should place element at front");
        assert_eq!(q.dequeue(), Some(0), "First dequeue should return requeued element");
        assert_eq!(q.dequeue(), Some(1), "Second dequeue should return original first element");
    }

    #[test]
    fn test_peek_empty_queue() {
        let q: Queue<i32> = nq();
        assert_eq!(q.peek(), None, "Peek on empty queue should return None");
    }

    #[test]
    fn test_dequeue_empty_queue() {
        let mut q: Queue<i32> = nq();
        assert_eq!(q.dequeue(), None, "Dequeue on empty queue should return None");
    }

    #[test]
    fn test_queue_with_strings() {
        let mut q: Queue<String> = nq();
        q.enqueue(String::from("hello"));
        q.enqueue(String::from("world"));
        assert_eq!(q.peek(), Some(&String::from("hello")), "Peek should return first string");
        assert_eq!(q.dequeue(), Some(String::from("hello")), "Dequeue should return first string");
        assert_eq!(q.peek(), Some(&String::from("world")), "Peek should return second string");
    }

    #[test]
    fn test_multiple_operations() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        assert_eq!(q.dequeue(), Some(1), "First dequeue should return 1");
        q.requeue(0);
        q.enqueue(3);
        assert_eq!(q.peek(), Some(&0), "Peek should return requeued element");
        assert_eq!(q.dequeue(), Some(0), "Dequeue should return requeued element");
        assert_eq!(q.dequeue(), Some(2), "Next dequeue should return 2");
        assert_eq!(q.dequeue(), Some(3), "Next dequeue should return 3");
        assert_eq!(q.dequeue(), None, "Queue should be empty");
    }

    #[test]
    fn test_requeue_empty_queue() {
        let mut q = nq();
        q.requeue(42);
        assert_eq!(q.peek(), Some(&42), "Requeue on empty queue should work");
        assert_eq!(q.dequeue(), Some(42), "Dequeue should return requeued element");
        assert_eq!(q.dequeue(), None, "Queue should be empty after dequeue");
    }
}
