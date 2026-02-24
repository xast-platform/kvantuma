use crate::ecs::world::World;

// struct TemporaryXastData {

// }

// pub trait TemporaryXastWorld {
//     fn spawn(&mut self, data: TemporaryXastData);

//     // fn query(&mut self, query: &[TemporaryXastComponentQuery]) -> ...;
// }

pub trait System {
    fn execute(&self, world: &mut World);
}