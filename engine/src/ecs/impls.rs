#![allow(non_snake_case)]
#![allow(unused_parens)]

use super::world::{READ, WRITE, World, Query, QueryMut};
use super::component::Component;

macro_rules! impl_components_bundle_tuple {
    () => {};
    ($($name:ident),+) => {
        impl<$($name: $crate::ecs::component::Component),+> $crate::ecs::component::ComponentsBundle for ($($name,)+) {
            fn for_each(&self, f: &mut dyn FnMut(&dyn $crate::ecs::component::Component)) {
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

macro_rules! query_helper {
    ($ID:ident : READ, $ptrs:expr, $index:expr) => {
        paste!(
            let ([<ptr_ $ID>], [<size_ $ID>], _) = $ptrs[$index];
            let [<slice_ $ID>] = unsafe { std::slice::from_raw_parts([<ptr_ $ID>] as *const u8, [<size_ $ID>]) };
            let $ID = unsafe { &*([<slice_ $ID>].as_ptr() as *const $ID) };
        );
    };

    ($ID:ident : WRITE, $ptrs:expr, $index:expr) => {
        paste!(
            let ([<ptr_ $ID>], [<size_ $ID>], _) = $ptrs[$index];
            let [<slice_ $ID>] = unsafe { std::slice::from_raw_parts_mut([<ptr_ $ID>] as *mut u8, [<size_ $ID>]) };
            let $ID = unsafe { &mut *([<slice_ $ID>].as_mut_ptr() as *mut $ID) };
        );
    };
}

macro_rules! count_idents {
    ($($idents:ident),*) => {
        <[()]>::len(&[$(count_idents!(@sub $idents)),*])
    };
    (@sub $ident:ident) => { () };
}

macro_rules! impl_query_mut {
    ($($ID:ident),+ : $LIFETIME:tt => $($RET:ty),+; $($TY:ty),+; $($RW:ident),+) => {
        impl<$LIFETIME, $($ID: Component + $LIFETIME),+> QueryMut<$LIFETIME> for ($($RET),+) {
            type Result = ($($TY),+);

            fn for_each_world_mut<F>(world: &$LIFETIME mut World, mut f: F)
            where
                F: FnMut(Self::Result),
            {
                use ::std::collections::HashSet;
                use ::pretty_type_name::pretty_type_name;
                let mut seen = HashSet::with_capacity(count_idents!($($ID),+));
                $(
                    if !seen.insert($ID::component_id()) {
                        panic!("Duplicate component in query: {}", pretty_type_name::<$ID>());
                    }
                )+

                world.query_erased_mut(&[
                    $(
                        ($ID::component_id(), $RW)
                    ),+
                ], |_, ptrs| {
                    let mut __index = 0;

                    use ::paste::paste;
                    $(
                        query_helper!($ID: $RW, ptrs, __index);
                        __index += 1;
                    )+

                    f(($($ID),+));
                });
            }
        }
    };
}

macro_rules! impl_query {
    ($($ID:ident),+ : $LIFETIME:tt => $($RET:ty),+; $($TY:ty),+) => {
        impl<$LIFETIME, $($ID: Component + $LIFETIME),+> Query<$LIFETIME> for ($($RET),+) {
            type Result = ($($TY),+);

            fn for_each_world<F>(world: &$LIFETIME World, mut f: F)
            where
                F: FnMut(Self::Result),
            {
                use ::std::collections::HashSet;
                use ::pretty_type_name::pretty_type_name;
                let mut seen = HashSet::with_capacity(count_idents!($($ID),+));
                $(
                    if !seen.insert($ID::component_id()) {
                        panic!("Duplicate component in query: {}", pretty_type_name::<$ID>());
                    }
                )+

                world.query_erased(&[
                    $(
                        ($ID::component_id())
                    ),+
                ], |_, ptrs| {
                    let mut __index = 0;

                    use ::paste::paste;
                    $(
                        paste!(
                            let ([<ptr_ $ID>], [<size_ $ID>]) = ptrs[__index];
                            let [<slice_ $ID>] = unsafe { std::slice::from_raw_parts([<ptr_ $ID>] as *const u8, [<size_ $ID>]) };
                            let $ID = unsafe { &*([<slice_ $ID>].as_ptr() as *const $ID) };
                        );
                        __index += 1;
                    )+

                    f(($($ID),+));
                });
            }
        }
    };
}

impl_query!(A: 'w => &A;                        &'w A);
impl_query!(A, B: 'w => &A, &B;                 &'w A, &'w B);
impl_query!(A, B, C: 'w => &A, &B, &C;          &'w A, &'w B, &'w C);
impl_query!(A, B, C, D: 'w => &A, &B, &C, &D;   &'w A, &'w B, &'w C, &'w D);

impl_query_mut!(A: 'w => &A;        &'w A;      READ);
impl_query_mut!(A: 'w => &mut A;    &'w mut A;  WRITE);

impl_query_mut!(A, B: 'w => &A, &B;          &'w A, &'w B;          READ, READ);
impl_query_mut!(A, B: 'w => &mut A, &B;      &'w mut A, &'w B;      WRITE, READ);
impl_query_mut!(A, B: 'w => &A, &mut B;      &'w A, &'w mut B;      READ, WRITE);
impl_query_mut!(A, B: 'w => &mut A, &mut B;  &'w mut A, &'w mut B;  WRITE, WRITE);

impl_query_mut!(A, B, C: 'w => &A, &B, &C;               &'w A, &'w B, &'w C;               READ, READ, READ);
impl_query_mut!(A, B, C: 'w => &mut A, &B, &C;           &'w mut A, &'w B, &'w C;           WRITE, READ, READ);
impl_query_mut!(A, B, C: 'w => &A, &mut B, &C;           &'w A, &'w mut B, &'w C;           READ, WRITE, READ);
impl_query_mut!(A, B, C: 'w => &A, &B, &mut C;           &'w A, &'w B, &'w mut C;           READ, READ, WRITE);
impl_query_mut!(A, B, C: 'w => &mut A, &mut B, &C;       &'w mut A, &'w mut B, &'w C;       WRITE, WRITE, READ);
impl_query_mut!(A, B, C: 'w => &mut A, &B, &mut C;       &'w mut A, &'w B, &'w mut C;       WRITE, READ, WRITE);
impl_query_mut!(A, B, C: 'w => &A, &mut B, &mut C;       &'w A, &'w mut B, &'w mut C;       READ, WRITE, WRITE);
impl_query_mut!(A, B, C: 'w => &mut A, &mut B, &mut C;   &'w mut A, &'w mut B, &'w mut C;   WRITE, WRITE, WRITE);

impl_query_mut!(A, B, C, D: 'w => &A, &B, &C, &D;                   &'w A, &'w B, &'w C, &'w D;                 READ, READ, READ, READ);
impl_query_mut!(A, B, C, D: 'w => &mut A, &B, &C, &D;               &'w mut A, &'w B, &'w C, &'w D;             WRITE, READ, READ, READ);
impl_query_mut!(A, B, C, D: 'w => &A, &mut B, &C, &D;               &'w A, &'w mut B, &'w C, &'w D;             READ, WRITE, READ, READ);
impl_query_mut!(A, B, C, D: 'w => &A, &B, &mut C, &D;               &'w A, &'w B, &'w mut C, &'w D;             READ, READ, WRITE, READ);
impl_query_mut!(A, B, C, D: 'w => &A, &B, &C, &mut D;               &'w A, &'w B, &'w C, &'w mut D;             READ, READ, READ, WRITE);
impl_query_mut!(A, B, C, D: 'w => &mut A, &mut B, &C, &D;           &'w mut A, &'w mut B, &'w C, &'w D;         WRITE, WRITE, READ, READ);
impl_query_mut!(A, B, C, D: 'w => &mut A, &B, &mut C, &D;           &'w mut A, &'w B, &'w mut C, &'w D;         WRITE, READ, WRITE, READ);
impl_query_mut!(A, B, C, D: 'w => &mut A, &B, &C, &mut D;           &'w mut A, &'w B, &'w C, &'w mut D;         WRITE, READ, READ, WRITE);
impl_query_mut!(A, B, C, D: 'w => &A, &mut B, &mut C, &D;           &'w A, &'w mut B, &'w mut C, &'w D;         READ, WRITE, WRITE, READ);
impl_query_mut!(A, B, C, D: 'w => &A, &mut B, &C, &mut D;           &'w A, &'w mut B, &'w C, &'w mut D;         READ, WRITE, READ, WRITE);
impl_query_mut!(A, B, C, D: 'w => &A, &B, &mut C, &mut D;           &'w A, &'w B, &'w mut C, &'w mut D;         READ, READ, WRITE, WRITE);
impl_query_mut!(A, B, C, D: 'w => &mut A, &mut B, &mut C, &D;       &'w mut A, &'w mut B, &'w mut C, &'w D;     WRITE, WRITE, WRITE, READ);
impl_query_mut!(A, B, C, D: 'w => &mut A, &mut B, &C, &mut D;       &'w mut A, &'w mut B, &'w C, &'w mut D;     WRITE, WRITE, READ, WRITE);
impl_query_mut!(A, B, C, D: 'w => &mut A, &B, &mut C, &mut D;       &'w mut A, &'w B, &'w mut C, &'w mut D;     WRITE, READ, WRITE, WRITE);
impl_query_mut!(A, B, C, D: 'w => &A, &mut B, &mut C, &mut D;       &'w A, &'w mut B, &'w mut C, &'w mut D;     READ, WRITE, WRITE, WRITE);
impl_query_mut!(A, B, C, D: 'w => &mut A, &mut B, &mut C, &mut D;   &'w mut A, &'w mut B, &'w mut C, &'w mut D; WRITE, WRITE, WRITE, WRITE);