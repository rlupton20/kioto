use std::mem;
use std::raw;

pub trait EventHandler {
    fn handle_event(&mut self, event: u32) -> ();
}

pub struct FFIRef<'a> {
    handler: &'a EventHandler,
    trait_object: raw::TraitObject
}

impl<'a> FFIRef<'a> {
    pub fn new<T: EventHandler>(v: &T) -> FFIRef<'static> {
        let handler = v as &EventHandler;
        // It's all totally fine
        let static_handler: &EventHandler = unsafe {
            mem::transmute(handler)
        };
        let trait_object = unsafe {
            mem::transmute(static_handler)
        };

        FFIRef {
            handler: static_handler,
            trait_object: trait_object
        }
    }

    pub fn epoll_repr(&mut self) -> u64 {
        let to_ptr = &mut self.trait_object as *mut raw::TraitObject;
        to_ptr as u64
    }

    pub fn call_handler(epoll_repr: u64, events: u32) {
        let to_ptr = (epoll_repr as *mut libc::c_void) as *mut raw::TraitObject;
        let handler: &mut EventHandler = unsafe {
            mem::transmute(*to_ptr)
        };

        handler.handle_event(0);
    }

}


#[cfg(test)]
mod test {
    use super::*;

    struct Test {
        flag: bool
    }

    impl EventHandler for Test {
        fn handle_event(&mut self, event: u32) -> () {
            self.flag = true;
            ()
        }
    }

    #[test]
    fn test_marshall_from_pointer() {

        let mut v = Test{
            flag: false
        };

        // Scope the borrow of v
        {
            let mut ffi_ref = FFIRef::new(&mut v);

            let repr = ffi_ref.epoll_repr();
            FFIRef::call_handler(repr, 0);
        }

        let result = v.flag;

        assert_eq!(result, true);
    }
}
