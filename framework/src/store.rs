#[derive(Copy, Clone, Debug)]
pub struct Handle<T> {
    index: u32,
    _marker: std::marker::PhantomData<T>,
}

pub struct Slab<T> {
    slots: Vec<Option<T>>,
    free_slots: Vec<u32>,
}

impl<T: Clone> Slab<T> {
    pub fn new() -> Slab<T> {
        return Slab {
            slots: Vec::new(),
            free_slots: Vec::new(),
        };
    }

    pub fn insert(&mut self, item: T) -> Handle<T> {
        if let Some(index) = self.free_slots.pop() {
            self.slots[index as usize] = Some(item);
            return Handle {
                index,
                _marker: Default::default(),
            };
        };
        let index = self.slots.len() as u32;
        self.slots.push(Some(item));
        return Handle {
            index,
            _marker: Default::default(),
        };
    }

    pub fn free(&mut self, handle: Handle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index as usize)?;
        if slot.is_some() {
            self.free_slots.push(handle.index);
            return slot.take();
        }
        return None;
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        let item = self.slots.get(handle.index as usize)?;
        if item.is_some() {
            return item.as_ref();
        }
        return None;
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let item = self.slots.get_mut(handle.index as usize)?;
        if item.is_some() {
            return item.as_mut();
        }
        return None;
    }
}
