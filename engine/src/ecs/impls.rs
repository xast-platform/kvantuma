#![allow(non_snake_case)]
#![allow(unused_parens)]

use super::world::{World, Query};
use super::component::Component;
use super::archetype::EntityId;

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

macro_rules! count_idents {
    ($($idents:ident),*) => {
        <[()]>::len(&[$(count_idents!(@sub $idents)),*])
    };
    (@sub $ident:ident) => { () };
}

macro_rules! impl_query {
    ($($ID:ident),+ : $LIFETIME:tt => $($RET:ty),+; $($TY:ty),+) => {
        impl<$LIFETIME, $($ID: Component + $LIFETIME),+> Query<$LIFETIME> for ($($RET),+) {
            type Result = ($($TY),+);

            fn for_each_world<Closure>(world: &$LIFETIME World, mut f: Closure)
            where
                Closure: FnMut(EntityId, Self::Result),
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
                ], |e, ptrs| {
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

                    f(e, ($($ID),+));
                });
            }
        }
    };
}

impl_query!(A: 'w                   => &A;                         &'w A);
impl_query!(A, B: 'w                => &A, &B;                     &'w A, &'w B);
impl_query!(A, B, C: 'w             => &A, &B, &C;                 &'w A, &'w B, &'w C);
impl_query!(A, B, C, D: 'w          => &A, &B, &C, &D;             &'w A, &'w B, &'w C, &'w D);
impl_query!(A, B, C, D, E: 'w       => &A, &B, &C, &D, &E;         &'w A, &'w B, &'w C, &'w D, &'w E);
impl_query!(A, B, C, D, E, F: 'w    => &A, &B, &C, &D, &E, &F;     &'w A, &'w B, &'w C, &'w D, &'w E, &'w F);