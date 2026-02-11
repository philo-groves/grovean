use limine::memory_map::{Entry, EntryType};
#[cfg(not(test))]
use limine::request::MemoryMapRequest;
use spin::Mutex;

const MAX_MEMORY_REGIONS: usize = 512;

#[cfg(not(test))]
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

static BOOT_MEMORY_MAP: Mutex<BootMemoryMap> = Mutex::new(BootMemoryMap::empty());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMapError {
    ResponseUnavailable,
    TooManyRegions,
    AddressOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    BootloaderReclaimable,
    ExecutableAndModules,
    Framebuffer,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub kind: MemoryRegionKind,
}

impl MemoryRegion {
    pub const fn empty() -> Self {
        Self {
            base: 0,
            length: 0,
            kind: MemoryRegionKind::Reserved,
        }
    }

    pub fn end(&self) -> Option<u64> {
        self.base.checked_add(self.length)
    }
}

pub struct BootMemoryMap {
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    len: usize,
    usable_memory_bytes: u64,
}

impl BootMemoryMap {
    pub const fn empty() -> Self {
        Self {
            regions: [MemoryRegion::empty(); MAX_MEMORY_REGIONS],
            len: 0,
            usable_memory_bytes: 0,
        }
    }

    pub fn from_limine_entries(entries: &[&Entry]) -> Result<Self, MemoryMapError> {
        let mut map = Self::empty();

        for entry in entries {
            if entry.length == 0 {
                continue;
            }

            let kind = memory_region_kind_from_limine(entry.entry_type);
            let normalized = MemoryRegion {
                base: entry.base,
                length: entry.length,
                kind,
            };

            let _ = normalized
                .base
                .checked_add(normalized.length)
                .ok_or(MemoryMapError::AddressOverflow)?;

            if map.try_merge_with_previous(normalized)? {
                continue;
            }

            if map.len >= MAX_MEMORY_REGIONS {
                return Err(MemoryMapError::TooManyRegions);
            }

            map.regions[map.len] = normalized;
            map.len += 1;
            if normalized.kind == MemoryRegionKind::Usable {
                map.usable_memory_bytes = map
                    .usable_memory_bytes
                    .checked_add(normalized.length)
                    .ok_or(MemoryMapError::AddressOverflow)?;
            }
        }

        Ok(map)
    }

    fn try_merge_with_previous(&mut self, next: MemoryRegion) -> Result<bool, MemoryMapError> {
        if self.len == 0 {
            return Ok(false);
        }

        let previous = &mut self.regions[self.len - 1];
        if previous.kind != next.kind || previous.end() != Some(next.base) {
            return Ok(false);
        }

        previous.length = previous
            .length
            .checked_add(next.length)
            .ok_or(MemoryMapError::AddressOverflow)?;
        if previous.kind == MemoryRegionKind::Usable {
            self.usable_memory_bytes = self
                .usable_memory_bytes
                .checked_add(next.length)
                .ok_or(MemoryMapError::AddressOverflow)?;
        }

        Ok(true)
    }

    pub fn regions(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }

    pub fn usable_memory_bytes(&self) -> u64 {
        self.usable_memory_bytes
    }
}

fn memory_region_kind_from_limine(entry_type: EntryType) -> MemoryRegionKind {
    if entry_type == EntryType::USABLE {
        MemoryRegionKind::Usable
    } else if entry_type == EntryType::RESERVED {
        MemoryRegionKind::Reserved
    } else if entry_type == EntryType::ACPI_RECLAIMABLE {
        MemoryRegionKind::AcpiReclaimable
    } else if entry_type == EntryType::ACPI_NVS {
        MemoryRegionKind::AcpiNvs
    } else if entry_type == EntryType::BAD_MEMORY {
        MemoryRegionKind::BadMemory
    } else if entry_type == EntryType::BOOTLOADER_RECLAIMABLE {
        MemoryRegionKind::BootloaderReclaimable
    } else if entry_type == EntryType::EXECUTABLE_AND_MODULES {
        MemoryRegionKind::ExecutableAndModules
    } else if entry_type == EntryType::FRAMEBUFFER {
        MemoryRegionKind::Framebuffer
    } else {
        MemoryRegionKind::Unknown
    }
}

pub fn init() {
    #[cfg(not(test))]
    {
        let response = MEMORY_MAP_REQUEST
            .get_response()
            .ok_or(MemoryMapError::ResponseUnavailable)
            .expect("limine memory map response is unavailable");

        let parsed = BootMemoryMap::from_limine_entries(response.entries())
            .expect("failed to normalize limine memory map");

        let mut boot_memory_map = BOOT_MEMORY_MAP.lock();
        *boot_memory_map = parsed;
    }
}

pub fn with_boot_memory_map<F, R>(f: F) -> R
where
    F: FnOnce(&BootMemoryMap) -> R,
{
    let boot_memory_map = BOOT_MEMORY_MAP.lock();
    f(&boot_memory_map)
}

#[cfg(test)]
mod tests {
    use kunit::kunit;

    use super::{BootMemoryMap, MemoryRegionKind};
    use limine::memory_map::{Entry, EntryType};

    #[kunit]
    fn normalizes_and_merges_adjacent_regions() {
        let entries = [
            Entry {
                base: 0x1000,
                length: 0x1000,
                entry_type: EntryType::USABLE,
            },
            Entry {
                base: 0x2000,
                length: 0x1000,
                entry_type: EntryType::USABLE,
            },
            Entry {
                base: 0x3000,
                length: 0x1000,
                entry_type: EntryType::RESERVED,
            },
        ];
        let refs = [&entries[0], &entries[1], &entries[2]];

        let map = BootMemoryMap::from_limine_entries(&refs).expect("map normalization should pass");
        let regions = map.regions();

        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].base, 0x1000);
        assert_eq!(regions[0].length, 0x2000);
        assert_eq!(regions[0].kind, MemoryRegionKind::Usable);
        assert_eq!(regions[1].base, 0x3000);
        assert_eq!(regions[1].length, 0x1000);
        assert_eq!(regions[1].kind, MemoryRegionKind::Reserved);
    }

    #[kunit]
    fn skips_zero_sized_entries() {
        let entries = [
            Entry {
                base: 0x0,
                length: 0,
                entry_type: EntryType::USABLE,
            },
            Entry {
                base: 0x1000,
                length: 0x1000,
                entry_type: EntryType::USABLE,
            },
        ];
        let refs = [&entries[0], &entries[1]];

        let map = BootMemoryMap::from_limine_entries(&refs).expect("map normalization should pass");
        let regions = map.regions();

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].base, 0x1000);
        assert_eq!(regions[0].length, 0x1000);
    }

    #[kunit]
    fn tracks_total_usable_memory_bytes() {
        let entries = [
            Entry {
                base: 0x1000,
                length: 0x2000,
                entry_type: EntryType::USABLE,
            },
            Entry {
                base: 0x4000,
                length: 0x1000,
                entry_type: EntryType::RESERVED,
            },
            Entry {
                base: 0x5000,
                length: 0x3000,
                entry_type: EntryType::USABLE,
            },
        ];
        let refs = [&entries[0], &entries[1], &entries[2]];

        let map = BootMemoryMap::from_limine_entries(&refs).expect("map normalization should pass");

        assert_eq!(map.usable_memory_bytes(), 0x5000);
    }
}
