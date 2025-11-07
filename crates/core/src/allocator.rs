/// An extremely simple (bare minimum) heap allocator
#[used]
#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();