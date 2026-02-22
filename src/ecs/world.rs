use pretty_type_name::pretty_type_name;

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

impl<'w, A: Component + 'w> Query<'w> for &A {
    type Result = &'w A;

    fn for_each_world<F>(world: &'w mut World, mut f: F)
    where
        F: FnMut(Self::Result),
    {
        world.query_erased_for_each(&[(A::component_id(), READ)], |_, ptrs| {
            let (ptr, size, _) = ptrs[0];
            let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) };
            let a = unsafe { &*(slice.as_ptr() as *const A) };
            f(a);
        });
    }
}

impl<'w, A: Component + 'w, B: Component + 'w> Query<'w> for (&A, &B) {
    type Result = (&'w A, &'w B);

    fn for_each_world<F>(world: &'w mut World, mut f: F)
    where
        F: FnMut(Self::Result),
    {
        if A::component_id() == B::component_id() {
            panic!(
                "Invalid query (&{}, &{}): cannot borrow the same component type as immutable twice",
                pretty_type_name::<A>(),
                pretty_type_name::<B>(),
            );
        }

        world.query_erased_for_each(&[
            (A::component_id(), READ),
            (B::component_id(), READ),
        ], |_, ptrs| {
            let (ptr_a, size_a, _) = ptrs[0];
            let (ptr_b, size_b, _) = ptrs[1];
            let slice_a = unsafe { std::slice::from_raw_parts(ptr_a as *const u8, size_a) };
            let slice_b = unsafe { std::slice::from_raw_parts(ptr_b as *const u8, size_b) };
            let a = unsafe { &*(slice_a.as_ptr() as *const A) };
            let b = unsafe { &*(slice_b.as_ptr() as *const B) };
            f((a, b));
        });
    }
}

impl<'w, A: Component + 'w, B: Component + 'w> Query<'w> for (&A, &mut B) {
    type Result = (&'w A, &'w mut B);

    fn for_each_world<F>(world: &'w mut World, mut f: F)
    where
        F: FnMut(Self::Result),
    {
        if A::component_id() == B::component_id() {
            panic!(
                "Invalid query (&{}, &mut {}): cannot borrow the same component type as both immutable and mutable",
                pretty_type_name::<A>(),
                pretty_type_name::<B>(),
            );
        }

        world.query_erased_for_each(&[
            (A::component_id(), READ),
            (B::component_id(), WRITE),
        ], |_, ptrs| {
            let (ptr_a, size_a, _) = ptrs[0];
            let (ptr_b, size_b, access_b) = ptrs[1];
            let slice_a = unsafe { std::slice::from_raw_parts(ptr_a as *const u8, size_a) };
            let a = unsafe { &*(slice_a.as_ptr() as *const A) };
            let b = match access_b {
                WRITE => unsafe { &mut *(std::slice::from_raw_parts_mut(ptr_b, size_b).as_mut_ptr() as *mut B) },
                _ => unreachable!(),
            };
            f((a, b));
        });
    }
}

impl<'w, A: Component + 'w, B: Component + 'w> Query<'w> for (&mut A, &mut B) {
    type Result = (&'w mut A, &'w mut B);

    fn for_each_world<F>(world: &'w mut World, mut f: F)
    where
        F: FnMut(Self::Result),
    {
        if A::component_id() == B::component_id() {
            panic!(
                "Invalid query (&mut {}, &mut {}): cannot borrow the same component type as mutable twice",
                pretty_type_name::<A>(),
                pretty_type_name::<B>(),
            );
        }

        world.query_erased_for_each(&[
            (A::component_id(), WRITE),
            (B::component_id(), WRITE),
        ], |_, ptrs| {
            let (ptr_a, size_a, access_a) = ptrs[0];
            let (ptr_b, size_b, access_b) = ptrs[1];
            let a = match access_a {
                WRITE => unsafe { &mut *(std::slice::from_raw_parts_mut(ptr_a, size_a).as_mut_ptr() as *mut A) },
                _ => unreachable!(),
            };
            let b = match access_b {
                WRITE => unsafe { &mut *(std::slice::from_raw_parts_mut(ptr_b, size_b).as_mut_ptr() as *mut B) },
                _ => unreachable!(),
            };
            f((a, b));
        });
    }
}

impl<'w, A: Component + 'w, B: Component + 'w> Query<'w> for (&mut A, &B) {
    type Result = (&'w mut A, &'w B);

    fn for_each_world<F>(world: &'w mut World, mut f: F)
    where
        F: FnMut(Self::Result),
    {
        if A::component_id() == B::component_id() {
            panic!(
                "Invalid query (&mut {}, &{}): cannot borrow the same component type as both immutable and mutable",
                pretty_type_name::<A>(),
                pretty_type_name::<B>(),
            );
        }

        world.query_erased_for_each(&[
            (A::component_id(), WRITE),
            (B::component_id(), READ),
        ], |_, ptrs| {
            let (ptr_a, size_a, access_a) = ptrs[0];
            let (ptr_b, size_b, _) = ptrs[1];
            let a = match access_a {
                WRITE => unsafe { &mut *(std::slice::from_raw_parts_mut(ptr_a, size_a).as_mut_ptr() as *mut A) },
                _ => unreachable!(),
            };
            let slice_b = unsafe { std::slice::from_raw_parts(ptr_b as *const u8, size_b) };
            let b = unsafe { &*(slice_b.as_ptr() as *const B) };
            f((a, b));
        });
    }
}

pub trait ComponentsBundle {
    fn for_each(&self, f: &mut dyn FnMut(&dyn Component));
}

macro_rules! impl_components_bundle_tuple {
    () => {};
    ($($name:ident),+) => {
        impl<$($name: Component),+> ComponentsBundle for ($($name,)+) {
            fn for_each(&self, f: &mut dyn FnMut(&dyn Component)) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(f($name);)+
            }
        }
    };
}

impl_components_bundle_tuple! { A }
impl_components_bundle_tuple! { A, B }
impl_components_bundle_tuple! { A, B, C }
impl_components_bundle_tuple! { A, B, C, D }
impl_components_bundle_tuple! { A, B, C, D, E }
impl_components_bundle_tuple! { A, B, C, D, E, F }
impl_components_bundle_tuple! { A, B, C, D, E, F, G }
impl_components_bundle_tuple! { A, B, C, D, E, F, G, H }