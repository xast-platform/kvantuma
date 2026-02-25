use super::archetype::*;
use super::component::*;

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    next_entity: EntityId,
    entity_to_archetype: Vec<Option<(usize, usize)>>,
}

impl World {
    pub fn new() -> World {
        World::default()
    }
}

impl World {
    pub fn spawn(&mut self, components: impl ComponentsBundle) -> EntityId {
        let entity_id = self.next_entity;
        self.next_entity.0 += 1;
        let mut ids = vec![];

        components.for_each(&mut |comp| {
            let id = comp.id();
            ids.push(id);
        });

        ids.sort();

        if let Some((idx, archetype)) = self
            .archetypes
            .iter_mut()
            .enumerate()
            .find(|(_, a)| a.has_components(&ids)) 
        {
            components.for_each(&mut |comp| {
                let id = comp.id();
                let col = archetype
                    .get_column_with_component_mut(id)
                    .expect("Should have found column after bitset check");

                col.push(comp);
            });

            let row = archetype.add_entity(entity_id);
            self.ensure_entity_map_capacity(entity_id);
            self.entity_to_archetype[entity_id.0 as usize] = Some((idx, row));
        } else {
            let mask = ArchetypeMask::from_ids(&ids);
            let mut columns = vec![];
            components.for_each(&mut |comp| {
                let mut col = Column::new(64, comp.id(), comp.layout(), comp.kind(), comp.drop_fn());
                col.push(comp);
                columns.push(col);
            });

            let mut archetype = Archetype::new(mask, columns);

            let row = archetype.add_entity(entity_id);
            let idx = self.archetypes.len();

            self.archetypes.push(archetype);
            self.ensure_entity_map_capacity(entity_id);
            self.entity_to_archetype[entity_id.0 as usize] = Some((idx, row));
        }

        entity_id
    }

    pub fn spawn_erased(&mut self, components: &[ErasedComponent]) -> EntityId {
        let entity_id = self.next_entity;
        self.next_entity.0 += 1;

        let mut ids = components
            .iter()
            .map(|comp| comp.id)
            .collect::<Vec<_>>();

        ids.sort();

        if let Some((idx, archetype)) = self
            .archetypes
            .iter_mut()
            .enumerate()
            .find(|(_, a)| a.has_components(&ids)) 
        {
            components.iter().for_each(|comp| {
                let id = comp.id;
                let col = archetype
                    .get_column_with_component_mut(id)
                    .expect("Should have found column after bitset check");

                col.push_erased(comp);
            });
            let row = archetype.add_entity(entity_id);

            self.ensure_entity_map_capacity(entity_id);
            self.entity_to_archetype[entity_id.0 as usize] = Some((idx, row));            
        } else {
            let mask = ArchetypeMask::from_ids(&ids);
            let mut columns = vec![];
            components.iter().for_each(|comp| {
                let mut col = Column::new(64, comp.id, comp.layout, comp.kind, comp.drop_fn);
                col.push_erased(comp);
                columns.push(col);
            });

            let mut archetype = Archetype::new(mask, columns);

            let row = archetype.add_entity(entity_id);
            let idx = self.archetypes.len();
            
            self.archetypes.push(archetype);
            self.ensure_entity_map_capacity(entity_id);
            self.entity_to_archetype[entity_id.0 as usize] = Some((idx, row));  
        }

        entity_id
    }

    pub fn apply(&mut self, write: &ComponentWrite) {
        let (arch_idx, row) = self.entity_to_archetype[write.entity.0 as usize]
            .expect("Entity does not exist");
        unsafe {
            self.archetypes[arch_idx].apply_write(write, row);
        }
    }

    fn ensure_entity_map_capacity(&mut self, entity: EntityId) {
        if self.entity_to_archetype.len() <= entity.0 as usize {
            self.entity_to_archetype
                .resize(entity.0 as usize + 1, None);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Access {
    Read,
    Write,
}

pub const READ: Access = Access::Read;
pub const WRITE: Access = Access::Write;

impl World {
    pub fn query_erased<F>(&self, ids: &[ComponentId], mut f: F)
    where
        F: FnMut(EntityId, &[(*const u8, usize)]),
    {
        for archetype in &self.archetypes {
            if !archetype.has_components(&ids) { continue }

            let len = archetype.entities().len();
            let column_indices: Vec<usize> = ids
                .iter()
                .map(|id| archetype.column_index(*id).unwrap())
                .collect();

            let base_ptr = archetype.columns().as_ptr();
            let mut ptrs: Vec<(*const u8, usize)> = Vec::with_capacity(ids.len());

            for i in 0..len {
                ptrs.clear();
                for idx in &column_indices {
                    unsafe {
                        let col_ptr = base_ptr.add(*idx);
                        let layout_size = (*col_ptr).component_meta().layout.size();
                        let data_ptr = (*col_ptr).as_ptr().add(i * layout_size);
                        ptrs.push((data_ptr, layout_size));
                    }
                }

                f(archetype.entities()[i], ptrs.as_slice());
            }
        }
    }

    pub fn for_each<'w, Q: Query<'w>, F>(&'w self, f: F)
    where
        F: FnMut(EntityId, Q::Result),
    {
        Q::for_each_world(self, f)
    }
}

pub struct ComponentWrite {
    pub entity: EntityId,
    pub component_id: ComponentId,
    pub bytes: Vec<u8>,
    pub drop_fn: Option<unsafe fn(*mut u8)>,
}

impl ComponentWrite {
    pub fn new<T: Component>(entity: EntityId, component: T) -> ComponentWrite {
        match component.kind() {
            ComponentKind::Pod => {
                let bytes = unsafe { pod_to_bytes(&component) };
                ComponentWrite {
                    entity,
                    component_id: T::component_id(),
                    bytes,
                    drop_fn: None,
                }
            }
            ComponentKind::Extern => {
                let drop_fn = component.drop_fn();
                let boxed = Box::new(component);
                let ptr = Box::into_raw(boxed) as *mut u8;
                let bytes = (ptr as usize).to_ne_bytes().to_vec();

                ComponentWrite {
                    entity,
                    component_id: T::component_id(),
                    bytes,
                    drop_fn,
                }
            }
        }
    }
}

use std::{mem, ptr};

unsafe fn pod_to_bytes<T>(component: &T) -> Vec<u8> {
    let size = mem::size_of::<T>();
    let mut bytes = Vec::with_capacity(size);

    unsafe {
        // Set length so Vec can be written into
        bytes.set_len(size);

        // Copy the bytes of the component into the Vec
        ptr::copy_nonoverlapping(
            component as *const T as *const u8,
            bytes.as_mut_ptr(),
            size,
        );
    }

    bytes
}

pub trait Query<'w> {
    type Result: 'w;

    fn for_each_world<F>(world: &'w World, f: F)
    where
        F: FnMut(EntityId, Self::Result);
}