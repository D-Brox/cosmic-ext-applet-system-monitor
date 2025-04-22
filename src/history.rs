use std::iter::Chain;
use std::slice::Iter;

#[derive(Clone, Debug)]
pub struct History<T = u64> {
    data: Vec<T>,
    capacity: usize,
    insertion_index: usize,
}

impl<T: Default + Copy> History<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            data: vec![Default::default(); capacity],
            capacity,
            insertion_index: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn push(&mut self, x: T) {
        if self.capacity == 0 {
            return;
        }

        _ = std::mem::replace(&mut self.data[self.insertion_index], x);

        self.insertion_index = (self.insertion_index + 1) % self.capacity;
    }

    pub fn iter(&self) -> Chain<Iter<T>, Iter<T>> {
        let (a, b) = self.data.split_at(self.insertion_index);
        b.iter().chain(a.iter())
    }

    pub fn resize(&mut self, capacity: usize) {
        let capacity = capacity.max(1);
        if capacity == self.capacity {
            return;
        }
        if self.capacity != 0 {
            // Reverse front and back to make it contiguous
            self.data[self.insertion_index..].reverse();
            self.data[..self.insertion_index].reverse();
        }
        // Resize, fill with default and update other values
        self.data.resize_with(capacity, Default::default);
        self.data.reverse();

        self.capacity = capacity;
        self.insertion_index = 0;
    }
}
