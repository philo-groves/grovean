pub mod memory_map;
pub mod frame_allocator;

pub fn init() {
    memory_map::init();
    frame_allocator::init();
}
