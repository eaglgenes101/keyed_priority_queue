use crate::heap_traits::{EditableHeap, HeapEntry, HeapIndex};
use std::cmp::{Ord, Ordering};
use std::fmt::Debug;
use std::vec::Vec;

use crate::mediator::MediatorIndex;

#[derive(Clone)]
pub struct BinaryHeap<TPriority>
where
    TPriority: Ord,
{
    data: Vec<HeapEntry<TPriority>>,
}

impl<TPriority: Ord> BinaryHeap<TPriority> {
    fn heapify_up<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        mut change_handler: TChangeHandler,
    ) {
        debug_assert!(position.0 < self.data.len(), "Out of index in heapify_up");
        let HeapIndex(mut position) = position;
        while position > 0 {
            let parent_pos = (position - 1) / 2;
            if self.data[parent_pos].priority >= self.data[position].priority {
                break;
            }
            self.data.swap(parent_pos, position);
            change_handler(self.data[position].outer_pos, HeapIndex(position));
            position = parent_pos;
        }
        change_handler(self.data[position].outer_pos, HeapIndex(position));
    }

    fn heapify_down<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        mut change_handler: TChangeHandler,
    ) {
        debug_assert!(position < self.len(), "Out of index in heapify_down");
        let HeapIndex(mut position) = position;
        loop {
            let max_child_idx = {
                let child1 = position * 2 + 1;
                let child2 = child1 + 1;
                if child1 >= self.data.len() {
                    break;
                }
                if child2 < self.data.len()
                    && self.data[child1].priority <= self.data[child2].priority
                {
                    child2
                } else {
                    child1
                }
            };

            if self.data[position].priority >= self.data[max_child_idx].priority {
                break;
            }
            self.data.swap(position, max_child_idx);
            change_handler(self.data[position].outer_pos, HeapIndex(position));
            position = max_child_idx;
        }
        change_handler(self.data[position].outer_pos, HeapIndex(position));
    }
}

impl<TPriority: Ord> EditableHeap<TPriority> for BinaryHeap<TPriority> {
    fn from_entries_vec(heap_base: Vec<HeapEntry<TPriority>>) -> Self {
        let heapify_start = std::cmp::min(heap_base.len() / 2 + 2, heap_base.len());
        let mut heap = BinaryHeap { data: heap_base };
        for pos in (0..heapify_start).rev().map(HeapIndex) {
            heap.heapify_down(pos, |_, _| {});
        }

        heap
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }

    /// Puts outer index and priority in queue
    /// outer_pos is assumed to be unique but not validated
    /// because validation too expensive
    /// Calls change_handler for every move of old values
    fn push<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        outer_pos: MediatorIndex,
        priority: TPriority,
        change_handler: TChangeHandler,
    ) {
        self.data.push(HeapEntry {
            outer_pos,
            priority,
        });
        self.heapify_up(HeapIndex(self.data.len() - 1), change_handler);
    }

    /// Removes item at position and returns it
    /// Time complexity - O(log n) swaps and change_handler calls
    fn remove<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        change_handler: TChangeHandler,
    ) -> Option<(MediatorIndex, TPriority)> {
        if position >= self.len() {
            return None;
        }
        if position.0 + 1 == self.len().0 {
            let result = self.data.pop().expect("At least 1 item");
            return Some(result.conv_pair());
        }

        let result = self.data.swap_remove(position.0);
        self.heapify_down(position, change_handler);
        Some(result.conv_pair())
    }

    #[inline]
    fn data(&self) -> &[HeapEntry<TPriority>] {
        &self.data
    }

    // Changes outer index for element and return old index
    fn change_outer_pos(&mut self, outer_pos: MediatorIndex, position: HeapIndex) -> MediatorIndex {
        debug_assert!(position < self.len(), "Out of index during changing key");

        let old_pos = self.data[position.0].outer_pos;
        self.data[position.0].outer_pos = outer_pos;
        old_pos
    }

    /// Changes priority of queue item
    /// Returns old priority
    fn change_priority<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        updated: TPriority,
        change_handler: TChangeHandler,
    ) -> TPriority {
        debug_assert!(
            position < self.len(),
            "Out of index during changing priority"
        );

        let old = std::mem::replace(&mut self.data[position.0].priority, updated);
        match old.cmp(&self.data[position.0].priority) {
            Ordering::Less => {
                self.heapify_up(position, change_handler);
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                self.heapify_down(position, change_handler);
            }
        }
        old
    }

    fn most_prioritized_idx(&self) -> Option<(MediatorIndex, HeapIndex)> {
        self.data.get(0).map(|x| (x.outer_pos, HeapIndex(0)))
    }

    #[inline]
    fn clear(&mut self) {
        self.data.clear();
    }
}

impl<TPriority: Debug + Ord> Debug for BinaryHeap<TPriority> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.data.fmt(f)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::heap_traits::EditableHeap;
    use std::cmp::Reverse;
    use std::collections::{HashMap, HashSet};

    fn is_valid_heap<TP: Ord>(heap: &BinaryHeap<TP>) -> bool {
        for (i, current) in heap.data.iter().enumerate().skip(1) {
            let parent = &heap.data[(i - 1) / 2];
            if parent.priority < current.priority {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_heap_fill() {
        let items = [
            70, 50, 0, 1, 2, 4, 6, 7, 9, 72, 4, 4, 87, 78, 72, 6, 7, 9, 2, -50, -72, -50, -42, -1,
            -3, -13,
        ];
        let mut maximum = std::i32::MIN;
        let mut heap = <BinaryHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        assert!(heap.data().get(0).is_none());
        assert!(is_valid_heap(&heap), "Heap state is invalid");
        for (key, x) in items
            .iter()
            .enumerate()
            .map(|(i, &x)| (MediatorIndex(i), x))
        {
            if x > maximum {
                maximum = x;
            }
            heap.push(key, x, |_, _| {});
            assert!(
                is_valid_heap(&heap),
                "Heap state is invalid after pushing {}",
                x
            );
            assert!(heap.data().get(0).is_some());
            let heap_max = heap.data().get(0).unwrap().priority;
            assert_eq!(maximum, heap_max)
        }
    }

    #[test]
    fn test_change_logger() {
        let items = [
            2, 3, 21, 22, 25, 29, 36, 90, 89, 88, 87, 83, 48, 50, 52, 69, 65, 55, 73, 75, 76, -53,
            78, 81, -45, -41, 91, -34, -33, -31, -27, -22, -19, -8, -5, -3,
        ];
        let mut last_positions = HashMap::<MediatorIndex, HeapIndex>::new();
        let mut heap = <BinaryHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        let mut on_pos_change = |outer_pos: MediatorIndex, position: HeapIndex| {
            last_positions.insert(outer_pos, position);
        };
        for (i, &x) in items.iter().enumerate() {
            heap.push(MediatorIndex(i), x, &mut on_pos_change);
        }
        assert_eq!(heap.data().len(), last_positions.len());
        for i in 0..items.len() {
            let rem_idx = MediatorIndex(i);
            assert!(
                last_positions.contains_key(&rem_idx),
                "Not for all items change_handler called"
            );
            let position = last_positions[&rem_idx];
            assert_eq!(
                items[(heap.data().get(position.0).unwrap()).outer_pos.0],
                heap.data().get(position.0).unwrap().priority
            );
            assert_eq!((heap.data().get(position.0).unwrap()).outer_pos, rem_idx);
        }

        let mut removed = HashSet::<MediatorIndex>::new();
        loop {
            let mut on_pos_change = |key: MediatorIndex, position: HeapIndex| {
                last_positions.insert(key, position);
            };
            let popped = heap.remove(HeapIndex(0), &mut on_pos_change);
            if popped.is_none() {
                break;
            }
            let (key, _) = popped.unwrap();
            last_positions.remove(&key);
            removed.insert(key);
            assert_eq!(heap.data().len(), last_positions.len());
            for i in (0..items.len())
                .into_iter()
                .filter(|i| !removed.contains(&MediatorIndex(*i)))
            {
                let rem_idx = MediatorIndex(i);
                assert!(
                    last_positions.contains_key(&rem_idx),
                    "Not for all items change_handler called"
                );
                let position = last_positions[&rem_idx];
                assert_eq!(
                    items[(heap.data().get(position.0).unwrap()).outer_pos.0],
                    heap.data().get(position.0).unwrap().priority
                );
                assert_eq!((heap.data().get(position.0).unwrap()).outer_pos, rem_idx);
            }
        }
    }

    #[test]
    fn test_pop() {
        let items = [
            -16, 5, 11, -1, -34, -42, -5, -6, 25, -35, 11, 35, -2, 40, 42, 40, -45, -48, 48, -38,
            -28, -33, -31, 34, -18, 25, 16, -33, -11, -6, -35, -38, 35, -41, -38, 31, -38, -23, 26,
            44, 38, 11, -49, 30, 7, 13, 12, -4, -11, -24, -49, 26, 42, 46, -25, -22, -6, -42, 28,
            45, -47, 8, 8, 21, 49, -12, -5, -33, -37, 24, -3, -26, 6, -13, 16, -40, -14, -39, -26,
            12, -44, 47, 45, -41, -22, -11, 20, 43, -44, 24, 47, 40, 43, 9, 19, 12, -17, 30, -36,
            -50, 24, -2, 1, 1, 5, -19, 21, -38, 47, 34, -14, 12, -30, 24, -2, -32, -10, 40, 34, 2,
            -33, 9, -31, -3, -15, 28, 50, -37, 35, 19, 35, 13, -2, 46, 28, 35, -40, -19, -1, -33,
            -42, -35, -12, 19, 29, 10, -31, -4, -9, 24, 15, -27, 13, 20, 15, 19, -40, -41, 40, -25,
            45, -11, -7, -19, 11, -44, -37, 35, 2, -49, 11, -37, -14, 13, 41, 10, 3, 19, -32, -12,
            -12, 33, -26, -49, -45, 24, 47, -29, -25, -45, -36, 40, 24, -29, 15, 36, 0, 47, 3, -45,
        ];

        let mut heap = <BinaryHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        for (i, &x) in items.iter().enumerate() {
            heap.push(MediatorIndex(i), x, |_, _| {});
        }
        assert!(is_valid_heap(&heap), "Heap is invalid before pops");

        let mut sorted_items = items;
        sorted_items.sort_unstable_by_key(|&x| Reverse(x));
        for &x in sorted_items.iter() {
            let pop_res = heap.remove(HeapIndex(0), |_, _| {});
            assert!(pop_res.is_some());
            let (rem_idx, val) = pop_res.unwrap();
            assert_eq!(val, x);
            assert_eq!(items[rem_idx.0], val);
            assert!(is_valid_heap(&heap), "Heap is invalid after {}", x);
        }

        assert_eq!(heap.remove(HeapIndex(0), |_, _| {}), None);
    }

    #[test]
    fn test_change_priority() {
        let pairs = [
            (MediatorIndex(0), 0),
            (MediatorIndex(1), 1),
            (MediatorIndex(2), 2),
            (MediatorIndex(3), 3),
            (MediatorIndex(4), 4),
        ];

        let mut heap = <BinaryHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        for (key, priority) in pairs.iter().cloned() {
            heap.push(key, priority, |_, _| {});
        }
        assert!(is_valid_heap(&heap), "Invalid before change");
        heap.change_priority(HeapIndex(3), 10, |_, _| {});
        assert!(is_valid_heap(&heap), "Invalid after upping");
        heap.change_priority(HeapIndex(2), -10, |_, _| {});
        assert!(is_valid_heap(&heap), "Invalid after lowering");
    }

    #[test]
    fn test_clear() {
        let mut heap = <BinaryHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        for x in 0..5 {
            heap.push(MediatorIndex(x), x as i32, |_, _| {});
        }
        assert!(!heap.data().is_empty(), "Heap must be non empty");
        heap.data.clear();
        assert!(heap.data().is_empty(), "Heap must be empty");
        assert_eq!(heap.remove(HeapIndex(0), |_, _| {}), None);
    }

    #[test]
    fn test_change_change_outer_pos() {
        let mut heap = <BinaryHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        for x in 0..5 {
            heap.push(MediatorIndex(x), x as i32, |_, _| {});
        }
        assert_eq!(
            heap.data().get(0).map(|n| *n),
            Some(HeapEntry {
                outer_pos: MediatorIndex(4),
                priority: 4i32
            })
        );
        assert_eq!(
            heap.change_outer_pos(MediatorIndex(10), HeapIndex(0)),
            MediatorIndex(4)
        );
        assert_eq!(
            heap.data().get(0).map(|n| *n),
            Some(HeapEntry {
                outer_pos: MediatorIndex(10),
                priority: 4i32
            })
        );
    }
}
