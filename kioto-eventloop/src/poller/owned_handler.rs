use libc;
use event_handler::EventHandler;

pub struct OwnedHandler {
    handler: Box<Box<dyn EventHandler>>,
    ptr: *mut libc::c_void
}

impl OwnedHandler {
    /// From a trait object, construct an owned handler
    pub fn new(v: Box<dyn EventHandler>) -> OwnedHandler {
        let wrapped_handler = Box::new(v);
        let ptr = Box::into_raw(wrapped_handler);
        let reconstructed = unsafe {
            Box::from_raw(ptr)
        };

        OwnedHandler{
            handler: reconstructed,
            ptr: ptr as *mut libc::c_void
        }
    }

    /// Provides a pointer to the underlying handler
    pub fn as_ptr(&self) -> *mut libc::c_void {
        self.ptr
    }

    /// From a pointer, dispatches event to an event handler
    pub fn handle_event(ptr: *mut libc::c_void, event: u32) {
        let mut p_handler = unsafe {
            Box::from_raw(ptr as *mut Box<dyn EventHandler>)
        };

        (*p_handler).handle_event(event);

        // Leak the pointer - it's owned elsewhere
        Box::into_raw(p_handler);
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct Test {
        flag: Arc<AtomicBool>
    }

    impl EventHandler for Test {
        fn handle_event(&mut self, event: u32) -> () {
            self.flag.store(true, Ordering::SeqCst);
            ()
        }
    }

    #[test]
    fn test_marshall_from_pointer() {

        let flag = Arc::new(AtomicBool::new(false));

        let v = Test{
            flag: Arc::clone(&flag)
        };

        let handler: Box<dyn EventHandler> = Box::new(v);
        let owned = OwnedHandler::new(handler);

        let ptr = owned.as_ptr();
        OwnedHandler::handle_event(ptr, 0);

        let result = flag.load(Ordering::SeqCst);
        assert_eq!(result, true);
    }
}
