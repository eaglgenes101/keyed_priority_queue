use crate::mediator::MediatorIndex;
use std::fmt::Debug;

/// Wrapper around usize that can be used only as index of `BinaryHeap`
/// Mostly needed to statically check that
/// Heap is not indexed by any other collection index
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct HeapIndex(pub(crate) usize);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct HeapEntry<TPriority> {
    pub(crate) outer_pos: MediatorIndex,
    pub(crate) priority: TPriority,
}

impl<TPriority> HeapEntry<TPriority> {
    // For usings as HeapEntry::as_pair instead of closures in map

    #[inline(always)]
    pub(crate) fn conv_pair(self) -> (MediatorIndex, TPriority) {
        (self.outer_pos, self.priority)
    }

    #[inline(always)]
    pub(crate) fn priority_ref(&self) -> &TPriority {
        &self.priority
    }

    #[inline(always)]
    pub(crate) fn to_outer(&self) -> MediatorIndex {
        self.outer_pos
    }
}

// Default implementations

impl<TPriority: Debug> Debug for HeapEntry<TPriority> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{{outer: {:?}, priority: {:?}}}",
            &self.outer_pos, &self.priority
        )
    }
}

pub trait EditableHeap<TPriority: Ord> {
    fn from_entries_vec(heap_base: Vec<HeapEntry<TPriority>>) -> Self;

    fn reserve(&mut self, additional: usize);

    /// Puts outer index and priority in queue
    /// outer_pos is assumed to be unique but not validated
    /// because validation too expensive
    /// Calls change_handler for every move of old values
    fn push<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        outer_pos: MediatorIndex,
        priority: TPriority,
        change_handler: TChangeHandler,
    );

    /// Removes item at position and returns it
    /// Time complexity - O(log n) swaps and change_handler calls
    fn remove<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        change_handler: TChangeHandler,
    ) -> Option<(MediatorIndex, TPriority)>;

    fn data(&self) -> &[HeapEntry<TPriority>];

    fn len(&self) -> HeapIndex {
        HeapIndex(self.data().len())
    }

    fn is_empty(&self) -> bool {
        self.data().is_empty()
    }

    // Changes outer index for element and return old index
    fn change_outer_pos(&mut self, outer_pos: MediatorIndex, position: HeapIndex) -> MediatorIndex;

    /// Changes priority of queue item
    /// Returns old priority
    fn change_priority<TChangeHandler: std::ops::FnMut(MediatorIndex, HeapIndex)>(
        &mut self,
        position: HeapIndex,
        updated: TPriority,
        change_handler: TChangeHandler,
    ) -> TPriority;

    fn most_prioritized_idx(&self) -> Option<(MediatorIndex, HeapIndex)>;

    fn clear(&mut self);
}
