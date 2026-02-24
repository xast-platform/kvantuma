use std::{alloc::Layout, any::TypeId, collections::HashMap, sync::{OnceLock, atomic::{AtomicU32, Ordering}}};

use parking_lot::Mutex;

pub trait ComponentsBundle {
    fn for_each(&self, f: &mut dyn FnMut(&dyn Component));
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

struct Registry {
    map: Mutex<HashMap<TypeId, u32>>,
    next: AtomicU32,
}

pub fn component_id<T: 'static>() -> ComponentId {
    let reg = REGISTRY.get_or_init(|| Registry {
        map: Default::default(),
        next: AtomicU32::new(1),
    });

    let mut map = reg.map.lock();

    ComponentId(*map.entry(TypeId::of::<T>())
        .or_insert_with(|| reg.next.fetch_add(1, Ordering::Relaxed))
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(pub(super) u32);

pub struct ErasedComponent {
    pub id: ComponentId,
    pub data: *const u8,
    pub layout: Layout,
    pub kind: ComponentKind,
    pub drop_fn: Option<unsafe fn(*mut u8)>,
}

#[macro_export]
macro_rules! component {
    {POD: $name:ident} => {
        impl $crate::ecs::component::Component for $name {
            fn component_id() -> $crate::ecs::component::ComponentId {
                $crate::ecs::component::component_id::<$name>()
            }
            fn id(&self) -> $crate::ecs::component::ComponentId {
                $crate::ecs::component::component_id::<$name>()
            }
            fn layout(&self) -> std::alloc::Layout {
                std::alloc::Layout::new::<Self>()
            }
            fn kind(&self) -> $crate::ecs::component::ComponentKind {
                $crate::ecs::component::ComponentKind::Pod
            }
            fn drop_fn(&self) -> Option<unsafe fn(*mut u8)> {
                None
            }
        }
    };

    {EXTERN: $name:ident} => {
        impl $crate::ecs::component::Component for $name {
            fn component_id() -> $crate::ecs::component::ComponentId {
                $crate::ecs::component::component_id::<$name>()
            }
            fn id(&self) -> $crate::ecs::component::ComponentId {
                $crate::ecs::component::component_id::<$name>()
            }
            fn layout(&self) -> std::alloc::Layout {
                std::alloc::Layout::new::<Self>()
            }
            fn kind(&self) -> $crate::ecs::component::ComponentKind {
                $crate::ecs::component::ComponentKind::Extern
            }
            fn drop_fn(&self) -> Option<unsafe fn(*mut u8)> {
                Some(|ptr| unsafe { std::ptr::drop_in_place(ptr as *mut $name) })
            }
        }
    };
}

pub trait Component {
    fn component_id() -> ComponentId where Self: Sized;
    fn id(&self) -> ComponentId;
    fn layout(&self) -> Layout;
    fn kind(&self) -> ComponentKind;
    fn drop_fn(&self) -> Option<unsafe fn(*mut u8)>;
}

pub enum Access {
    Write,
    Read,
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Pod,
    Extern,
}

pub struct ComponentMeta {
    pub id: ComponentId,
    pub kind: ComponentKind,
    pub layout: Layout,
    pub drop_fn: Option<unsafe fn(*mut u8)>,
}

pub struct ComponentQuery {
    _id: ComponentId,
    _access: Access,
}