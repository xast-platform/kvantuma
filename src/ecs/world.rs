use super::archetype::*;
use super::component::*;

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    next_entity: EntityId,
}

impl World {
    pub fn new() -> World {
        World::default()
    }
}

impl World {
    pub fn spawn(&mut self, components: impl ComponentsBundle) -> EntityId {
        let mut ids = vec![];

        components.for_each(&mut |comp| {
            let id = comp.id();
            ids.push(id);
        });

        ids.sort();

        if let Some(archetype) = self
            .archetypes
            .iter_mut()
            .find(|a| a.has_components(&ids)) 
        {
            components.for_each(&mut |comp| {
                let id = comp.id();
                let col = archetype
                    .get_column_with_component_mut(id)
                    .expect("Should have found column after bitset check");

                col.push(comp);
            });

            let id = self.next_entity;
            archetype.add_entity(id);
            self.next_entity += 1;
            
            id
        } else {
            let mask = ArchetypeMask::from_ids(&ids);
            let mut columns = vec![];
            components.for_each(&mut |comp| {
                let mut col = Column::new(64, comp.id(), comp.layout(), comp.kind(), comp.drop_fn());
                col.push(comp);
                columns.push(col);
            });

            let mut archetype = Archetype::new(mask, columns);

            let id = self.next_entity;
            archetype.add_entity(id);
            self.archetypes.push(archetype);
            self.next_entity += 1;
            
            id
        }
    }

    pub fn spawn_erased(&mut self, components: &[ErasedComponent]) -> EntityId {
        let mut ids = components
            .iter()
            .map(|comp| comp.id)
            .collect::<Vec<_>>();

        ids.sort();

        if let Some(archetype) = self
            .archetypes
            .iter_mut()
            .find(|a| a.has_components(&ids)) 
        {
            components.iter().for_each(|comp| {
                let id = comp.id;
                let col = archetype
                    .get_column_with_component_mut(id)
                    .expect("Should have found column after bitset check");

                col.push_erased(comp);
            });

            let id = self.next_entity;
            archetype.add_entity(id);
            self.next_entity += 1;
            
            id
        } else {
            let mask = ArchetypeMask::from_ids(&ids);
            let mut columns = vec![];
            components.iter().for_each(|comp| {
                let mut col = Column::new(64, comp.id, comp.layout, comp.kind, comp.drop_fn);
                col.push_erased(comp);
                columns.push(col);
            });

            let mut archetype = Archetype::new(mask, columns);

            let id = self.next_entity;
            archetype.add_entity(id);
            self.archetypes.push(archetype);
            self.next_entity += 1;
            
            id
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
    pub fn query_erased_for_each<F>(&mut self, components: &[(ComponentId, Access)], mut f: F)
    where
        F: FnMut(EntityId, &[(*mut u8, usize, Access)]),
    {
        let ids = components
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        for archetype in &mut self.archetypes {
            if !archetype.has_components(&ids) { continue }

            let len = archetype.entities().len();
            let column_indices: Vec<(usize, Access)> = components
                .iter()
                .map(|(id, access)| (
                    archetype.column_index(*id).unwrap(),
                    *access
                ))
                .collect();

            let base_ptr = archetype.columns_mut().as_mut_ptr();
            let mut ptrs: Vec<(*mut u8, usize, Access)> = Vec::with_capacity(components.len());

            for i in 0..len {
                ptrs.clear();
                for (idx, access) in &column_indices {
                    unsafe {
                        let col_ptr = base_ptr.add(*idx);
                        let layout_size = (*col_ptr).component_meta().layout.size();
                        let data_ptr = (*col_ptr).as_ptr().add(i * layout_size);
                        ptrs.push((data_ptr, layout_size, *access));
                    }
                }

                f(archetype.entities()[i], ptrs.as_slice());
            }
        }
    }

    pub fn for_each<'w, Q: Query<'w>, F>(&'w mut self, f: F)
    where
        F: FnMut(Q::Result),
    {
        Q::for_each_world(self, f)
    }

    pub fn query<'w, Q: Query<'w>>(&'w mut self) -> Vec<Q::Result> {
        let mut out = Vec::new();
        self.for_each::<Q, _>(|r| out.push(r));
        out
    }
}

pub trait Query<'w> {
    type Result: 'w;

    fn for_each_world<F>(world: &'w mut World, f: F)
    where
        F: FnMut(Self::Result);
}