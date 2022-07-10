mod collections_alloc;
mod dyn_array;
mod btree_map;
mod btree_set;
mod linked_list;
mod vec_deque;


pub use dyn_array::*;
pub use btree_map::*;
pub use btree_set::*;
pub use linked_list::*;
pub use vec_deque::*;


#[doc(hidden)]
trait SpecExtend<I: IntoIterator> {
    fn spec_extend(&mut self, iter: I);
}