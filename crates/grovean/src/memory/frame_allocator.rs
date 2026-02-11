use spin::Mutex;

use crate::memory::memory_map::{self, MemoryRegion, MemoryRegionKind};

pub const FRAME_SIZE_BYTES: u64 = 4096;
const MAX_FRAME_REGIONS: usize = 512;
const MAX_RECYCLED_FRAMES: usize = 512;

static FRAME_ALLOCATOR: Mutex<FrameAllocatorState> = Mutex::new(FrameAllocatorState::Uninitialized);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameAllocatorError {
    Uninitialized,
    OutOfMemory,
    InvalidFrameAddress,
    InvalidFrameCount,
    AddressOverflow,
    TooManyRegions,
    InvalidReserveRange,
    FreeListFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    start_address: u64,
}

impl PhysFrame {
    pub fn from_start_address(start_address: u64) -> Result<Self, FrameAllocatorError> {
        if start_address % FRAME_SIZE_BYTES != 0 {
            return Err(FrameAllocatorError::InvalidFrameAddress);
        }

        Ok(Self { start_address })
    }

    pub fn start_address(self) -> u64 {
        self.start_address
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameAllocatorStats {
    pub total_frames: u64,
    pub free_frames: u64,
    pub used_frames: u64,
}

trait FrameAllocatorBackend {
    fn alloc_frame(&mut self) -> Result<PhysFrame, FrameAllocatorError>;
    fn free_frame(&mut self, frame: PhysFrame) -> Result<(), FrameAllocatorError>;
    fn alloc_contiguous(&mut self, count: usize) -> Result<PhysFrame, FrameAllocatorError>;
    fn reserve_range(&mut self, base: u64, length: u64) -> Result<(), FrameAllocatorError>;
    fn stats(&self) -> FrameAllocatorStats;
}

enum FrameAllocatorState {
    Uninitialized,
    Active(ActiveBackend),
}

enum ActiveBackend {
    Cursor(CursorFrameAllocator),
}

impl ActiveBackend {
    fn alloc_frame(&mut self) -> Result<PhysFrame, FrameAllocatorError> {
        match self {
            ActiveBackend::Cursor(backend) => backend.alloc_frame(),
        }
    }

    fn free_frame(&mut self, frame: PhysFrame) -> Result<(), FrameAllocatorError> {
        match self {
            ActiveBackend::Cursor(backend) => backend.free_frame(frame),
        }
    }

    fn alloc_contiguous(&mut self, count: usize) -> Result<PhysFrame, FrameAllocatorError> {
        match self {
            ActiveBackend::Cursor(backend) => backend.alloc_contiguous(count),
        }
    }

    fn reserve_range(&mut self, base: u64, length: u64) -> Result<(), FrameAllocatorError> {
        match self {
            ActiveBackend::Cursor(backend) => backend.reserve_range(base, length),
        }
    }

    fn stats(&self) -> FrameAllocatorStats {
        match self {
            ActiveBackend::Cursor(backend) => backend.stats(),
        }
    }
}

#[derive(Clone, Copy)]
struct FrameRegion {
    start: u64,
    next: u64,
    end: u64,
}

impl FrameRegion {
    const fn empty() -> Self {
        Self {
            start: 0,
            next: 0,
            end: 0,
        }
    }

    fn frame_count(&self) -> u64 {
        (self.end - self.start) / FRAME_SIZE_BYTES
    }

    fn overlaps(&self, other_start: u64, other_end: u64) -> bool {
        self.start < other_end && other_start < self.end
    }
}

struct CursorFrameAllocator {
    regions: [FrameRegion; MAX_FRAME_REGIONS],
    region_len: usize,
    region_cursor: usize,
    recycled_frames: [u64; MAX_RECYCLED_FRAMES],
    recycled_len: usize,
    total_frames: u64,
    free_frames: u64,
    used_frames: u64,
}

impl CursorFrameAllocator {
    const fn empty() -> Self {
        Self {
            regions: [FrameRegion::empty(); MAX_FRAME_REGIONS],
            region_len: 0,
            region_cursor: 0,
            recycled_frames: [0; MAX_RECYCLED_FRAMES],
            recycled_len: 0,
            total_frames: 0,
            free_frames: 0,
            used_frames: 0,
        }
    }

    #[cfg(test)]
    fn from_memory_regions(regions: &[MemoryRegion]) -> Result<Self, FrameAllocatorError> {
        let mut allocator = Self::empty();
        allocator.initialize_from_memory_regions(regions)?;
        Ok(allocator)
    }

    fn initialize_from_memory_regions(
        &mut self,
        regions: &[MemoryRegion],
    ) -> Result<(), FrameAllocatorError> {
        self.region_len = 0;
        self.region_cursor = 0;
        self.recycled_len = 0;
        self.total_frames = 0;
        self.free_frames = 0;
        self.used_frames = 0;

        for region in regions {
            if region.kind != MemoryRegionKind::Usable || region.length == 0 {
                continue;
            }

            let end = region
                .base
                .checked_add(region.length)
                .ok_or(FrameAllocatorError::AddressOverflow)?;

            let start_aligned = align_up(region.base, FRAME_SIZE_BYTES)?;
            let end_aligned = align_down(end, FRAME_SIZE_BYTES);

            if start_aligned >= end_aligned {
                continue;
            }

            if self.region_len >= MAX_FRAME_REGIONS {
                return Err(FrameAllocatorError::TooManyRegions);
            }

            let frame_region = FrameRegion {
                start: start_aligned,
                next: start_aligned,
                end: end_aligned,
            };
            self.total_frames = self
                .total_frames
                .checked_add(frame_region.frame_count())
                .ok_or(FrameAllocatorError::AddressOverflow)?;

            self.regions[self.region_len] = frame_region;
            self.region_len += 1;
        }

        self.free_frames = self.total_frames;

        Ok(())
    }

    fn contains_frame_address(&self, address: u64) -> bool {
        if address % FRAME_SIZE_BYTES != 0 {
            return false;
        }

        for region in &self.regions[..self.region_len] {
            if address >= region.start && address < region.end {
                return true;
            }
        }

        false
    }

    fn pop_recycled(&mut self) -> Option<u64> {
        if self.recycled_len == 0 {
            return None;
        }

        self.recycled_len -= 1;
        Some(self.recycled_frames[self.recycled_len])
    }

    fn push_recycled(&mut self, address: u64) -> Result<(), FrameAllocatorError> {
        if self.recycled_len >= MAX_RECYCLED_FRAMES {
            return Err(FrameAllocatorError::FreeListFull);
        }

        self.recycled_frames[self.recycled_len] = address;
        self.recycled_len += 1;
        Ok(())
    }

    fn recompute_total_frames(&mut self) -> Result<(), FrameAllocatorError> {
        let mut total = 0u64;

        for region in &self.regions[..self.region_len] {
            total = total
                .checked_add(region.frame_count())
                .ok_or(FrameAllocatorError::AddressOverflow)?;
        }

        self.total_frames = total;
        self.free_frames = self.total_frames.saturating_sub(self.used_frames);
        Ok(())
    }
}

impl FrameAllocatorBackend for CursorFrameAllocator {
    fn alloc_frame(&mut self) -> Result<PhysFrame, FrameAllocatorError> {
        if let Some(address) = self.pop_recycled() {
            self.used_frames += 1;
            self.free_frames = self.free_frames.saturating_sub(1);
            return PhysFrame::from_start_address(address);
        }

        while self.region_cursor < self.region_len {
            let region = &mut self.regions[self.region_cursor];
            if region.next < region.end {
                let address = region.next;
                region.next = region
                    .next
                    .checked_add(FRAME_SIZE_BYTES)
                    .ok_or(FrameAllocatorError::AddressOverflow)?;

                self.used_frames += 1;
                self.free_frames = self.free_frames.saturating_sub(1);
                return PhysFrame::from_start_address(address);
            }

            self.region_cursor += 1;
        }

        Err(FrameAllocatorError::OutOfMemory)
    }

    fn free_frame(&mut self, frame: PhysFrame) -> Result<(), FrameAllocatorError> {
        let address = frame.start_address();
        if !self.contains_frame_address(address) {
            return Err(FrameAllocatorError::InvalidFrameAddress);
        }
        if self.used_frames == 0 || self.free_frames >= self.total_frames {
            return Err(FrameAllocatorError::InvalidFrameAddress);
        }

        self.push_recycled(address)?;
        self.used_frames -= 1;
        self.free_frames += 1;
        Ok(())
    }

    fn alloc_contiguous(&mut self, count: usize) -> Result<PhysFrame, FrameAllocatorError> {
        if count == 0 {
            return Err(FrameAllocatorError::InvalidFrameCount);
        }

        let count_u64 = count as u64;
        let size_bytes = count_u64
            .checked_mul(FRAME_SIZE_BYTES)
            .ok_or(FrameAllocatorError::AddressOverflow)?;

        for idx in self.region_cursor..self.region_len {
            let region = &mut self.regions[idx];
            let candidate_end = region
                .next
                .checked_add(size_bytes)
                .ok_or(FrameAllocatorError::AddressOverflow)?;

            if candidate_end <= region.end {
                let start = region.next;
                region.next = candidate_end;
                self.region_cursor = idx;
                self.used_frames += count_u64;
                self.free_frames = self.free_frames.saturating_sub(count_u64);
                return PhysFrame::from_start_address(start);
            }

            if region.next == region.end && self.region_cursor == idx {
                self.region_cursor += 1;
            }
        }

        Err(FrameAllocatorError::OutOfMemory)
    }

    fn reserve_range(&mut self, base: u64, length: u64) -> Result<(), FrameAllocatorError> {
        if length == 0 {
            return Ok(());
        }

        if self.used_frames != 0 {
            return Err(FrameAllocatorError::InvalidReserveRange);
        }

        let reserve_end = base
            .checked_add(length)
            .ok_or(FrameAllocatorError::AddressOverflow)?;
        let reserve_start = align_down(base, FRAME_SIZE_BYTES);
        let reserve_end_aligned = align_up(reserve_end, FRAME_SIZE_BYTES)?;

        let mut next_regions = [FrameRegion::empty(); MAX_FRAME_REGIONS];
        let mut next_len = 0usize;

        for region in &self.regions[..self.region_len] {
            if !region.overlaps(reserve_start, reserve_end_aligned) {
                if next_len >= MAX_FRAME_REGIONS {
                    return Err(FrameAllocatorError::TooManyRegions);
                }

                next_regions[next_len] = *region;
                next_len += 1;
                continue;
            }

            if reserve_start > region.start {
                if next_len >= MAX_FRAME_REGIONS {
                    return Err(FrameAllocatorError::TooManyRegions);
                }

                let left_end = reserve_start.min(region.end);
                if region.start < left_end {
                    next_regions[next_len] = FrameRegion {
                        start: region.start,
                        next: region.start,
                        end: left_end,
                    };
                    next_len += 1;
                }
            }

            if reserve_end_aligned < region.end {
                if next_len >= MAX_FRAME_REGIONS {
                    return Err(FrameAllocatorError::TooManyRegions);
                }

                let right_start = reserve_end_aligned.max(region.start);
                if right_start < region.end {
                    next_regions[next_len] = FrameRegion {
                        start: right_start,
                        next: right_start,
                        end: region.end,
                    };
                    next_len += 1;
                }
            }
        }

        self.regions = next_regions;
        self.region_len = next_len;
        self.region_cursor = 0;
        self.recycled_len = 0;
        self.recompute_total_frames()?;

        Ok(())
    }

    fn stats(&self) -> FrameAllocatorStats {
        FrameAllocatorStats {
            total_frames: self.total_frames,
            free_frames: self.free_frames,
            used_frames: self.used_frames,
        }
    }
}

pub fn init() {
    memory_map::with_boot_memory_map(|map| {
        let mut allocator = FRAME_ALLOCATOR.lock();

        if matches!(&*allocator, FrameAllocatorState::Uninitialized) {
            *allocator =
                FrameAllocatorState::Active(ActiveBackend::Cursor(CursorFrameAllocator::empty()));
        }

        match &mut *allocator {
            FrameAllocatorState::Active(ActiveBackend::Cursor(backend)) => backend
                .initialize_from_memory_regions(map.regions())
                .expect("failed to initialize frame allocator"),
            FrameAllocatorState::Uninitialized => unreachable!(),
        }
    });
}

pub fn alloc_frame() -> Result<PhysFrame, FrameAllocatorError> {
    let mut allocator = FRAME_ALLOCATOR.lock();

    match &mut *allocator {
        FrameAllocatorState::Uninitialized => Err(FrameAllocatorError::Uninitialized),
        FrameAllocatorState::Active(backend) => backend.alloc_frame(),
    }
}

pub fn free_frame(frame: PhysFrame) -> Result<(), FrameAllocatorError> {
    let mut allocator = FRAME_ALLOCATOR.lock();

    match &mut *allocator {
        FrameAllocatorState::Uninitialized => Err(FrameAllocatorError::Uninitialized),
        FrameAllocatorState::Active(backend) => backend.free_frame(frame),
    }
}

pub fn alloc_contiguous(count: usize) -> Result<PhysFrame, FrameAllocatorError> {
    let mut allocator = FRAME_ALLOCATOR.lock();

    match &mut *allocator {
        FrameAllocatorState::Uninitialized => Err(FrameAllocatorError::Uninitialized),
        FrameAllocatorState::Active(backend) => backend.alloc_contiguous(count),
    }
}

pub fn reserve_range(base: u64, length: u64) -> Result<(), FrameAllocatorError> {
    let mut allocator = FRAME_ALLOCATOR.lock();

    match &mut *allocator {
        FrameAllocatorState::Uninitialized => Err(FrameAllocatorError::Uninitialized),
        FrameAllocatorState::Active(backend) => backend.reserve_range(base, length),
    }
}

pub fn with_stats<F, R>(f: F) -> Result<R, FrameAllocatorError>
where
    F: FnOnce(&FrameAllocatorStats) -> R,
{
    let allocator = FRAME_ALLOCATOR.lock();

    match &*allocator {
        FrameAllocatorState::Uninitialized => Err(FrameAllocatorError::Uninitialized),
        FrameAllocatorState::Active(backend) => {
            let stats = backend.stats();
            Ok(f(&stats))
        }
    }
}

const fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

fn align_up(value: u64, align: u64) -> Result<u64, FrameAllocatorError> {
    if value == 0 {
        return Ok(0);
    }

    let mask = align - 1;
    value
        .checked_add(mask)
        .map(|rounded| rounded & !mask)
        .ok_or(FrameAllocatorError::AddressOverflow)
}

#[cfg(test)]
mod tests {
    use kunit::kunit;

    use super::{
        CursorFrameAllocator, FrameAllocatorBackend, FrameAllocatorError, MemoryRegion,
        MemoryRegionKind,
    };

    fn usable_region(base: u64, length: u64) -> MemoryRegion {
        MemoryRegion {
            base,
            length,
            kind: MemoryRegionKind::Usable,
        }
    }

    fn reserved_region(base: u64, length: u64) -> MemoryRegion {
        MemoryRegion {
            base,
            length,
            kind: MemoryRegionKind::Reserved,
        }
    }

    #[kunit]
    fn allocates_all_frames_then_reports_out_of_memory() {
        let regions = [usable_region(0x1000, 0x3000)];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        let first = allocator
            .alloc_frame()
            .expect("first frame should allocate");
        let second = allocator
            .alloc_frame()
            .expect("second frame should allocate");
        let third = allocator
            .alloc_frame()
            .expect("third frame should allocate");

        assert_eq!(first.start_address(), 0x1000);
        assert_eq!(second.start_address(), 0x2000);
        assert_eq!(third.start_address(), 0x3000);
        assert_eq!(
            allocator.alloc_frame(),
            Err(FrameAllocatorError::OutOfMemory)
        );

        let stats = allocator.stats();
        assert_eq!(stats.total_frames, 3);
        assert_eq!(stats.used_frames, 3);
        assert_eq!(stats.free_frames, 0);
    }

    #[kunit]
    fn frees_and_reuses_frames() {
        let regions = [usable_region(0x8000, 0x2000)];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        let frame0 = allocator.alloc_frame().expect("frame0 should allocate");
        let frame1 = allocator.alloc_frame().expect("frame1 should allocate");
        allocator.free_frame(frame0).expect("free should pass");
        let recycled = allocator
            .alloc_frame()
            .expect("recycled frame should allocate");

        assert_eq!(frame0.start_address(), 0x8000);
        assert_eq!(frame1.start_address(), 0x9000);
        assert_eq!(recycled.start_address(), frame0.start_address());
    }

    #[kunit]
    fn ignores_non_usable_regions() {
        let regions = [
            reserved_region(0x0000, 0x5000),
            usable_region(0x8000, 0x1000),
            reserved_region(0x9000, 0x2000),
        ];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        let frame = allocator
            .alloc_frame()
            .expect("one usable frame should allocate");
        assert_eq!(frame.start_address(), 0x8000);
        assert_eq!(
            allocator.alloc_frame(),
            Err(FrameAllocatorError::OutOfMemory)
        );
    }

    #[kunit]
    fn rounds_regions_to_frame_boundaries() {
        let regions = [usable_region(0x1003, 0x3001)];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        let frame0 = allocator
            .alloc_frame()
            .expect("first aligned frame should allocate");
        let frame1 = allocator
            .alloc_frame()
            .expect("second aligned frame should allocate");

        assert_eq!(frame0.start_address(), 0x2000);
        assert_eq!(frame1.start_address(), 0x3000);
        assert_eq!(
            allocator.alloc_frame(),
            Err(FrameAllocatorError::OutOfMemory)
        );
    }

    #[kunit]
    fn allocates_contiguous_frame_ranges() {
        let regions = [usable_region(0x1000, 0x5000)];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        let start = allocator
            .alloc_contiguous(3)
            .expect("contiguous allocation should pass");
        let next = allocator
            .alloc_frame()
            .expect("remaining frame should allocate");

        assert_eq!(start.start_address(), 0x1000);
        assert_eq!(next.start_address(), 0x4000);
    }

    #[kunit]
    fn rejects_invalid_free_address() {
        let regions = [usable_region(0x1000, 0x2000)];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        let invalid = super::PhysFrame::from_start_address(0x7000).expect("alignment should pass");
        assert_eq!(
            allocator.free_frame(invalid),
            Err(FrameAllocatorError::InvalidFrameAddress)
        );
    }

    #[kunit]
    fn reserve_range_removes_frames_from_allocation_pool() {
        let regions = [usable_region(0x1000, 0x6000)];
        let mut allocator = CursorFrameAllocator::from_memory_regions(&regions)
            .expect("allocator init should pass");

        allocator
            .reserve_range(0x2000, 0x2000)
            .expect("reserve should pass");

        let frame0 = allocator.alloc_frame().expect("frame0 should allocate");
        let frame1 = allocator.alloc_frame().expect("frame1 should allocate");
        let frame2 = allocator.alloc_frame().expect("frame2 should allocate");
        let frame3 = allocator.alloc_frame().expect("frame3 should allocate");

        assert_eq!(frame0.start_address(), 0x1000);
        assert_eq!(frame1.start_address(), 0x4000);
        assert_eq!(frame2.start_address(), 0x5000);
        assert_eq!(frame3.start_address(), 0x6000);
        assert_eq!(
            allocator.alloc_frame(),
            Err(FrameAllocatorError::OutOfMemory)
        );
    }
}
