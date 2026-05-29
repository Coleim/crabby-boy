pub struct AudioBuffer {
    data: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    count: usize, // nombre d'éléments dans le buffer
    capacity: usize,
}

impl AudioBuffer {
    pub fn new(capacity: usize) -> Self {
        AudioBuffer {
            data: vec![0.0; capacity],
            write_pos: 0,
            read_pos: 0,
            count: 0,
            capacity: capacity,
        }
    }
    pub fn empty(&self) -> bool {
        self.count == 0
    }

    pub fn push(&mut self, sample: f32) {
        if self.count < self.capacity {
            self.data[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            self.count += 1;
        }
    }

    pub fn pop(&mut self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }

        let val = self.data[self.read_pos];
        self.read_pos = (self.read_pos + 1) % self.capacity;
        self.count -= 1;
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_many(buf: &mut AudioBuffer, samples: &[f32]) {
        for &s in samples {
            buf.push(s);
        }
    }

    fn pop_all(buf: &mut AudioBuffer, n: usize) -> Vec<f32> {
        (0..n).map(|_| buf.pop()).collect()
    }

    #[test]
    fn new_buffer_is_empty() {
        let mut buf = AudioBuffer::new(4);
        assert_eq!(buf.count, 0);
        // Popping an empty buffer must return silence, not panic
        assert_eq!(buf.pop(), 0.0);
    }

    #[test]
    fn new_buffer_respects_capacity() {
        let buf = AudioBuffer::new(8);
        assert_eq!(buf.capacity, 8);
        assert_eq!(buf.data.len(), 8);
    }

    #[test]
    fn fifo_order_is_preserved() {
        let mut buf = AudioBuffer::new(4);
        push_many(&mut buf, &[1.0, 2.0, 3.0]);
        assert_eq!(pop_all(&mut buf, 3), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn single_push_pop_roundtrip() {
        let mut buf = AudioBuffer::new(4);
        buf.push(0.42);
        assert_eq!(buf.pop(), 0.42);
    }

    #[test]
    fn push_beyond_capacity_is_silently_dropped() {
        let mut buf = AudioBuffer::new(3);
        push_many(&mut buf, &[1.0, 2.0, 3.0, 99.0]); // 4th sample must be dropped
        assert_eq!(buf.count, 3);
        // Original three samples arrive intact; 99.0 never entered
        assert_eq!(pop_all(&mut buf, 3), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn count_never_exceeds_capacity() {
        let mut buf = AudioBuffer::new(2);
        for i in 0..10 {
            buf.push(i as f32);
            assert!(buf.count <= buf.capacity);
        }
    }

    #[test]
    fn pop_on_empty_returns_silence() {
        let mut buf = AudioBuffer::new(4);
        assert_eq!(buf.pop(), 0.0);
    }

    #[test]
    fn pop_more_than_pushed_returns_silence_for_extras() {
        let mut buf = AudioBuffer::new(4);
        push_many(&mut buf, &[1.0, 2.0]);
        assert_eq!(buf.pop(), 1.0);
        assert_eq!(buf.pop(), 2.0);
        assert_eq!(buf.pop(), 0.0); // underflow → silence
        assert_eq!(buf.pop(), 0.0); // still silence
    }

    #[test]
    fn ring_wraps_correctly_after_drain_and_refill() {
        let mut buf = AudioBuffer::new(4);
        push_many(&mut buf, &[1.0, 2.0, 3.0, 4.0]);
        // Drain half so read_pos advances into the middle
        assert_eq!(buf.pop(), 1.0);
        assert_eq!(buf.pop(), 2.0);
        // Fill again — write_pos must wrap around the ring
        push_many(&mut buf, &[5.0, 6.0]);
        assert_eq!(pop_all(&mut buf, 4), vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn multiple_wrap_around_cycles_stay_correct() {
        let mut buf = AudioBuffer::new(3);
        for cycle in 0..5_u32 {
            let a = (cycle * 10 + 1) as f32;
            let b = (cycle * 10 + 2) as f32;
            let c = (cycle * 10 + 3) as f32;
            push_many(&mut buf, &[a, b, c]);
            assert_eq!(pop_all(&mut buf, 3), vec![a, b, c]);
            assert_eq!(buf.count, 0);
        }
    }

    #[test]
    fn capacity_one_works_as_single_slot() {
        let mut buf = AudioBuffer::new(1);
        buf.push(7.0);
        buf.push(8.0); // must be dropped — buffer is full
        assert_eq!(buf.pop(), 7.0);
        assert_eq!(buf.pop(), 0.0); // empty again
    }

    #[test]
    fn count_tracks_pushes_and_pops_accurately() {
        let mut buf = AudioBuffer::new(8);
        for i in 1..=5 {
            buf.push(i as f32);
            assert_eq!(buf.count, i);
        }
        for i in (0..=4).rev() {
            buf.pop();
            assert_eq!(buf.count, i);
        }
    }

    #[test]
    fn extreme_float_values_are_stored_faithfully() {
        let mut buf = AudioBuffer::new(4);
        let values = [f32::MAX, f32::MIN, f32::INFINITY, f32::NEG_INFINITY];
        push_many(&mut buf, &values);
        let out = pop_all(&mut buf, 4);
        for (expected, got) in values.iter().zip(out.iter()) {
            // Use bit-level comparison so ±Inf round-trips correctly
            assert_eq!(expected.to_bits(), got.to_bits());
        }
    }
}
