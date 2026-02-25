use std::{alloc::{Layout, alloc, realloc, dealloc}, ptr::NonNull};
// dense vec used for column lookup; no external map needed

use crate::ecs::{component::{Component, ComponentId, ComponentKind, ErasedComponent}, world::ComponentWrite};

use super::component::ComponentMeta;

pub const MAX_COMPONENTS: usize = 256;
pub const WORDS: usize = MAX_COMPONENTS / u64::BITS as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EntityId(pub(super) u32);

#[derive(Clone)]
pub struct ArchetypeMask {
    words: [u64; WORDS],
}

impl ArchetypeMask {
    pub fn from_ids(ids: &[ComponentId]) -> Self {
        if ids.len() > MAX_COMPONENTS {
            panic!("Too many components in archetype {}", ids.len());
        }

        let mut mask = ArchetypeMask { words: [0; WORDS] };
        for &id in ids {
            let word_index = (id.0 as usize) / u64::BITS as usize;
            let bit_index = (id.0 as usize) % u64::BITS as usize;
            mask.words[word_index] |= 1 << bit_index;
        }

        mask
    }

    pub fn contains(&self, other: &ArchetypeMask) -> bool {
        self.words.iter().zip(other.words.iter())
            .all(|(a, b)| (a & b) == *b)
    }
}

pub struct Column {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
    meta: ComponentMeta,
}

impl Column {
    pub fn new(
        capacity: usize, 
        id: ComponentId, 
        layout: Layout,
        kind: ComponentKind,
        drop_fn: Option<unsafe fn(*mut u8)>,
    ) -> Column {
        let meta = ComponentMeta {
            id,
            kind,
            layout,
            drop_fn,
        };

        if meta.layout.size() == 0 {
            Column {
                ptr: NonNull::dangling(),
                len: 0,
                capacity: usize::MAX,
                meta,
            }
        } else {
            let total_size = meta.layout.size() * capacity;
            let ptr_raw = unsafe { alloc(Layout::from_size_align(total_size, meta.layout.align()).unwrap()) };
            Column {
                ptr: NonNull::new(ptr_raw).unwrap(),
                len: 0,
                capacity,
                meta,
            }
        }
    }

    pub fn push(&mut self, component: &dyn Component) {
        if self.meta.layout.size() == 0 {
            self.len += 1;
            return;
        }

        if self.len >= self.capacity {
            let new_capacity = self.capacity.saturating_mul(2).max(1);
            let old_size = self.meta.layout.size() * self.capacity;
            let new_size = self.meta.layout.size() * new_capacity;
            let old_layout = Layout::from_size_align(old_size, self.meta.layout.align()).unwrap();
            let new_ptr = unsafe { realloc(self.ptr.as_ptr(), old_layout, new_size) };
            self.ptr = NonNull::new(new_ptr).unwrap();
            self.capacity = new_capacity;
        }

        let offset = self.len * self.meta.layout.size();
        unsafe {
            std::ptr::copy_nonoverlapping(
                component as *const _ as *const u8,
                self.ptr.as_ptr().add(offset),
                self.meta.layout.size(),
            );
        }
        self.len += 1;
    }

    pub fn push_erased(&mut self, component: &ErasedComponent) {
        if self.meta.layout.size() == 0 {
            self.len += 1;
            return;
        }

        if self.len >= self.capacity {
            let new_capacity = self.capacity.saturating_mul(2).max(1);
            let old_size = self.meta.layout.size() * self.capacity;
            let new_size = self.meta.layout.size() * new_capacity;
            let old_layout = Layout::from_size_align(old_size, self.meta.layout.align()).unwrap();
            let new_ptr = unsafe { realloc(self.ptr.as_ptr(), old_layout, new_size) };
            self.ptr = NonNull::new(new_ptr).unwrap();
            self.capacity = new_capacity;
        }

        let offset = self.len * self.meta.layout.size();
        unsafe {
            std::ptr::copy_nonoverlapping(
                component.data,
                self.ptr.as_ptr().add(offset),
                self.meta.layout.size(),
            );
        }
        self.len += 1;
    }
    
    pub fn component_meta(&self) -> &ComponentMeta {
        &self.meta
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub fn as_ptr_mut(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        if self.meta.layout.size() == 0 {
            return;
        }

        for i in 0..self.len {
            if let Some(drop_fn) = self.meta.drop_fn {
                unsafe {
                    drop_fn(self.ptr.as_ptr().add(i * self.meta.layout.size()));
                }
            }
        }
        unsafe {
            dealloc(self.ptr.as_ptr(), Layout::from_size_align(self.meta.layout.size() * self.capacity, self.meta.layout.align()).unwrap());
        }
    }
}

pub struct Archetype {
    mask: ArchetypeMask,
    columns: Vec<Column>,
    entities: Vec<EntityId>,
    column_map: Vec<Option<usize>>,
}

impl Archetype {
    pub fn new(
        mask: ArchetypeMask,
        columns: Vec<Column>,
    ) -> Archetype {
        let mut map = vec![None; MAX_COMPONENTS];
        for (i, col) in columns.iter().enumerate() {
            let id = col.meta.id.0 as usize;
            if id >= MAX_COMPONENTS {
                panic!("Component id {} exceeds MAX_COMPONENTS", id);
            }
            map[id] = Some(i);
        }

        Archetype {
            mask,
            columns,
            entities: vec![],
            column_map: map,
        }
    }

    pub fn has_components(&self, ids: &[ComponentId]) -> bool {
        let mask = ArchetypeMask::from_ids(ids);
        self.mask.contains(&mask)
    }

    pub fn get_column_with_component(&mut self, id: ComponentId) -> Option<&Column> {
        let id = id.0 as usize;
        if id >= MAX_COMPONENTS { return None }
        self.column_map[id].map(|idx| &self.columns[idx])
    }

    pub fn get_column_with_component_mut(&mut self, id: ComponentId) -> Option<&mut Column> {
        let id = id.0 as usize;
        if id >= MAX_COMPONENTS { return None }
        match self.column_map[id] {
            Some(idx) => Some(&mut self.columns[idx]),
            None => None,
        }
    }

    pub fn column_index(&self, id: ComponentId) -> Option<usize> {
        let id = id.0 as usize;
        if id >= MAX_COMPONENTS { return None }
        self.column_map[id]
    }
    
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn columns_mut(&mut self) -> &mut [Column] {
        &mut self.columns
    }

    pub unsafe fn apply_write(&mut self, write: &ComponentWrite, row: usize) {
        let col_idx = match self.column_map.get(write.component_id.0 as usize).copied().flatten() {
            Some(idx) => idx,
            None => panic!("Component {} not found in archetype", write.component_id.0),
        };

        let column = &mut self.columns[col_idx];

        if row >= column.capacity {
            panic!("Row {} out of capacity {}", row, column.capacity);
        }

        match column.meta.kind {
            ComponentKind::Pod => {
                let dest_ptr = unsafe {
                    column.ptr.as_ptr().add(row * column.meta.layout.size())
                };
                assert_eq!(write.bytes.len(), column.meta.layout.size());
                unsafe {
                    std::ptr::copy_nonoverlapping(write.bytes.as_ptr(), dest_ptr, write.bytes.len());
                }
            }
            ComponentKind::Extern => {
                assert_eq!(write.bytes.len(), std::mem::size_of::<usize>());
                let mut ptr_bytes = [0u8; std::mem::size_of::<usize>()];
                ptr_bytes.copy_from_slice(&write.bytes[..]);
                let boxed_ptr = usize::from_ne_bytes(ptr_bytes) as *mut u8;

                if row < column.len {
                    if let Some(drop_fn) = column.meta.drop_fn {
                        unsafe {
                            drop_fn(column.ptr.as_ptr().add(row * column.meta.layout.size()));
                        }
                    }
                }

                unsafe {
                    let dest_ptr = column.ptr.as_ptr().add(row * column.meta.layout.size());
                    std::ptr::write(dest_ptr as *mut *mut u8, boxed_ptr);
                }
            }
        }

        if row >= column.len {
            column.len = row + 1;
        }
    }

    pub(super) fn add_entity(&mut self, id: EntityId) -> usize {
        let row = self.entities.len();
        self.entities.push(id);
        row
    }
}