use crate::heap_traits::{EditableHeap, HeapEntry, HeapIndex};
use crate::mediator::MediatorIndex;
use std::cmp::{Ord, Ordering};
use std::fmt::Debug;
use std::vec::Vec;

/// Enum which determines which side the sibling node is on. The child node is on the other side.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum SiblingSide {
    Left,
    Right,
}

impl SiblingSide {
    fn flip(&mut self) {
        match self {
            SiblingSide::Left => *self = SiblingSide::Right,
            SiblingSide::Right => *self = SiblingSide::Left,
        };
    }

    fn as_bool(&self) -> bool {
        *self == SiblingSide::Right
    }
}

impl Default for SiblingSide {
    fn default() -> Self {
        SiblingSide::Left
    }
}

#[derive(Clone)]
pub struct WeakHeap<TPriority>
where
    TPriority: Ord,
{
    sides: Vec<SiblingSide>,
    data: Vec<HeapEntry<TPriority>>,
}

impl<TPriority: Ord> WeakHeap<TPriority> {
    fn distinguished_ancestor(&self, position: HeapIndex) -> HeapIndex {
        let HeapIndex(mut position) = position;
        while position > 0 {
            let binary_parent_pos = position / 2;
            let is_direct_child = (position % 2 == 0) == self.sides[binary_parent_pos].as_bool();
            if is_direct_child {
                return HeapIndex(binary_parent_pos);
            } else {
                // This binary parent is actually our sibling
                position = binary_parent_pos;
                // And we go through the loop to see what our sibling's parent is
            }
        }
        // If we got here, then we're the root
        HeapIndex(0)
    }

    fn next_sibling(&self, position: HeapIndex) -> HeapIndex {
        let HeapIndex(position) = position;
        HeapIndex(position * 2 + self.sides[position].as_bool() as usize)
    }

    fn first_child(&self, position: HeapIndex) -> HeapIndex {
        let HeapIndex(position) = position;
        HeapIndex(position * 2 + (!self.sides[position].as_bool()) as usize)
    }

    fn heapify_up<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        mut change_handler: TChangeHandler,
    ) {
        debug_assert!(position < self.len(), "Out of index in heapify_up");
        let HeapIndex(mut position) = position;
        while position > 0 {
            let HeapIndex(parent_pos) = self.distinguished_ancestor(HeapIndex(position));
            if self.data[parent_pos].priority >= self.data[position].priority {
                break;
            }
            self.data.swap(parent_pos, position);
            self.sides[position].flip();
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
        let HeapIndex(position) = position;
        let first_child_idx = self.first_child(HeapIndex(position));
        if first_child_idx.0 < self.data.len() {
            let max_child_idx = {
                let mut current_child_idx = first_child_idx;
                loop {
                    let cand_child_idx = self.next_sibling(current_child_idx);
                    if cand_child_idx.0 >= self.data.len() {
                        break current_child_idx;
                    } else {
                        current_child_idx = cand_child_idx;
                    }
                }
            };
            let mut current_child_idx = max_child_idx;
            while current_child_idx.0 > position {
                if self.data[position].priority < self.data[current_child_idx.0].priority {
                    self.data.swap(current_child_idx.0, position);
                    self.sides[current_child_idx.0].flip();
                    change_handler(self.data[current_child_idx.0].outer_pos, current_child_idx);
                }
                current_child_idx.0 /= 2;
            }
        }
        change_handler(self.data[position].outer_pos, HeapIndex(position));
    }

    /*
    fn format_recursive(
        &self,
        head: &str,
        i: usize,
        f: &mut std::fmt::Formatter,
    ) -> Result<(), std::fmt::Error>
    where
        TPriority: Debug,
    {
        writeln!(f, "{}{:?}", head, self.data[i])?;
        if i == 0 {
            if self.data.len() > 1 {
                self.format_recursive("    ", 1, f)?;
            }
        } else {
            let HeapIndex(next_child_ind) = self.first_child(HeapIndex(i));
            if next_child_ind < self.data.len() {
                let head_more = format!("{}    ", head);
                self.format_recursive(&head_more, next_child_ind, f)?;
            }
            let HeapIndex(next_sibling_ind) = self.next_sibling(HeapIndex(i));
            if next_sibling_ind < self.data.len() {
                self.format_recursive(head, next_sibling_ind, f)?;
            }
        }
        Result::Ok(())
    }
    */
}

impl<TPriority: Ord> EditableHeap<TPriority> for WeakHeap<TPriority> {
    fn from_entries_vec(heap_base: Vec<HeapEntry<TPriority>>) -> Self {
        let heap_len = heap_base.len();
        let mut heap = WeakHeap {
            data: heap_base,
            sides: vec![SiblingSide::default(); heap_len],
        };
        let ignorant_distinguished_ancestor = |mut position| {
            while position > 0 {
                if position % 2 != 0 {
                    return position / 2;
                } else {
                    // This binary parent is actually our sibling
                    position /= 2;
                    // And we go through the loop to see what our sibling's parent is
                }
            }
            // If we got here, then we're the root
            0
        };
        for pos in (1..heap_len).rev() {
            let ancestor_pos = ignorant_distinguished_ancestor(pos);
            if heap.data[ancestor_pos].priority < heap.data[pos].priority {
                heap.data.swap(ancestor_pos, pos);
                heap.sides[pos].flip();
            }
        }

        heap
    }

    #[inline]
    fn reserve(&mut self, capacity: usize) {
        self.data.reserve(capacity)
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
        let new_index = self.data.len();
        self.data.push(HeapEntry {
            outer_pos,
            priority,
        });
        self.sides.push(SiblingSide::default());
        if new_index % 2 == 0 {
            self.sides[new_index / 2] = SiblingSide::default();
        }
        self.heapify_up(HeapIndex(new_index), change_handler);
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

        if HeapIndex(position.0 + 1) == self.len() {
            let result = self.data.pop().expect("At least 1 item");
            self.sides.pop();
            return Some(result.conv_pair());
        }

        let result = self.data.swap_remove(position.0);
        self.sides.pop();
        self.heapify_down(position, change_handler);
        Some(result.conv_pair())
    }

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

    #[inline]
    fn most_prioritized_idx(&self) -> Option<(MediatorIndex, HeapIndex)> {
        self.data.get(0).map(|x| (x.outer_pos, HeapIndex(0)))
    }

    fn clear(&mut self) {
        self.data.clear();
        self.sides.clear();
    }
}

impl<TPriority: Debug + Ord> Debug for WeakHeap<TPriority> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.data.fmt(f)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::cmp::Reverse;
    use std::collections::{HashMap, HashSet};

    fn is_valid_weak_heap<TP: Ord + Debug>(heap: &WeakHeap<TP>) -> bool {
        for (i, current) in heap.data.iter().enumerate().skip(1) {
            let heap_parent_ind = i / 2;
            if heap.first_child(HeapIndex(heap_parent_ind)) == HeapIndex(i) {
                let parent = &heap.data[heap_parent_ind];
                if parent.priority < current.priority {
                    println!(
                        "Heap condition violated at mediator index {}",
                        current.outer_pos.0
                    );
                    println!("{:?}", heap);
                    return false;
                }
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
        let mut heap = <WeakHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        assert!(heap.data().get(0).is_none());
        assert!(is_valid_weak_heap(&heap), "Heap state is invalid");
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
                is_valid_weak_heap(&heap),
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
        let mut heap = <WeakHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
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

        println!("{}", items.len());
        let mut heap = <WeakHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        for (i, &x) in items.iter().enumerate() {
            heap.push(MediatorIndex(i), x, |_, _| {});
        }
        assert!(is_valid_weak_heap(&heap), "Heap is invalid before pops");

        let mut sorted_items = items;
        sorted_items.sort_unstable_by_key(|&x| Reverse(x));
        for &x in sorted_items.iter() {
            let pop_res = heap.remove(HeapIndex(0), |_, _| {});
            assert!(pop_res.is_some());
            let (rem_idx, val) = pop_res.unwrap();
            assert_eq!(val, x);
            assert_eq!(items[rem_idx.0], val);
            assert!(is_valid_weak_heap(&heap), "Heap is invalid after {}", x);
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

        let mut heap = <WeakHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
        for (key, priority) in pairs.iter().cloned() {
            heap.push(key, priority, |_, _| {});
        }
        assert!(is_valid_weak_heap(&heap), "Invalid before change");
        heap.change_priority(HeapIndex(3), 10, |_, _| {});
        assert!(is_valid_weak_heap(&heap), "Invalid after upping");
        heap.change_priority(HeapIndex(2), -10, |_, _| {});
        assert!(is_valid_weak_heap(&heap), "Invalid after lowering");
    }

    #[test]
    fn test_clear() {
        let mut heap = <WeakHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
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
        let mut heap = <WeakHeap<i32> as EditableHeap<i32>>::from_entries_vec(Vec::new());
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
