mod cfuns;

use event_handler::{EventHandler, FFIRef};

use std::io;
use std::sync;
use std::thread;

use libc::{EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD};
use libc::{EPOLLIN, EPOLLOUT, EPOLLET};
use libc;

use std::raw;
use std::mem;


const MAX_EVENTS: usize = 256;


pub struct Epoller {
    epoll_fd: i32,
    mgmt_event_fd: MgmtFd,
    mgmt_ffi_ref: FFIRef<'static>
}


struct MgmtFd {
    mgmt_event_fd: i32
}

impl EventHandler for MgmtFd {
    fn handle_event(&mut self, _events: u32) -> () {
        println!("FOO");
        ()
    }
}


impl Epoller {
    /// Create a new epoller
    pub fn new() -> io::Result<Epoller> {
        let mgmt_event_fd = MgmtFd { mgmt_event_fd: cfuns::eventfd(0, 0)? };
        let mgmt_ffi_ref = FFIRef::new(&mgmt_event_fd);
        let mut epoller = Epoller{
            epoll_fd: cfuns::epoll_create()?,
            mgmt_event_fd: mgmt_event_fd,
            mgmt_ffi_ref: mgmt_ffi_ref
        };

        // Add event handler to epoller
        let mut events = cfuns::epoll_event(EPOLLIN as u32 | EPOLLET as u32, epoller.mgmt_ffi_ref.epoll_repr());

        cfuns::epoll_ctl(epoller.epoll_fd,
                         EPOLL_CTL_ADD,
                         epoller.mgmt_event_fd.mgmt_event_fd,
                         &mut events)?;
        Ok(epoller)
    }

    /// From the current epoller, run an event loop
    pub fn event_loop(self, poll_wait: u16) -> EventLoop {
        let stop = sync::Arc::new(
            sync::atomic::AtomicBool::new(false)
        );

        // Clone things we need inside the event loop
        let stop_flag = sync::Arc::clone(&stop);
        let epoll_fd = self.epoll_fd;

        // Spawn a thread to handle events
        let thread = thread::spawn(move || {

            let empty_event: libc::epoll_event = libc::epoll_event {
                events: 0,
                u64: 0
            };
            let mut events: [libc::epoll_event; MAX_EVENTS] = [empty_event; MAX_EVENTS];

            while !stop_flag.load(sync::atomic::Ordering::SeqCst) {
                Epoller::process_events(epoll_fd, &mut events, poll_wait);
            }
        });

        // Return a handle to the event loop
        EventLoop{
            stop: stop,
            thread: thread,
            epoller: self,
        }
    }

    /// Helper to process a batch of events
    fn process_events(epoll_fd: i32, events: &mut [libc::epoll_event], poll_wait: u16) {
        let n = events.len();
        // epoll_wait and act on the result
        match cfuns::epoll_wait(epoll_fd, events, n as i32, poll_wait as i32) {
            Ok(num_events) => {
                for ix in 0..num_events {
                    Epoller::handle(events[ix as usize]);
                }
                println!("Handled {} events", num_events);
            },
            Err(_) => ()
        }
    }

    /// Handle an individual event
    fn handle(event: libc::epoll_event) {
        let epoll_repr = event.u64;
        FFIRef::call_handler(epoll_repr, event.events);
    }

}

pub struct EventLoop {
    stop: sync::Arc<sync::atomic::AtomicBool>,
    thread: thread::JoinHandle<()>,
    epoller: Epoller,
}

impl EventLoop {
    /// Stop the event loop
    pub fn stop(self) -> () {
        self.stop.store(true, sync::atomic::Ordering::SeqCst);
        self.thread.join();
        ()
    }

    /// Write to the management eventfd to notify it of an event
    fn ping(&self) -> () {
        let buf: u64 = 1;
        let buf_ptr = &buf as *const u64;
        let n = unsafe {
            libc::write(self.epoller.mgmt_event_fd.mgmt_event_fd,
                        buf_ptr as *const libc::c_void,
                        8)
        };
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_can_create_epoller() {
        let epoller = Epoller::new();
        assert!(epoller.is_ok());
    }

    #[test]
    fn test_can_run_event_loop() {
        let epoller = Epoller::new().unwrap();
        let event_loop = epoller.event_loop(100);
        event_loop.stop();
    }

    #[test]
    fn test_ping_of_event_loop() {
        // Literally just testing we don't segfault etc.
        let mut epoller = Epoller::new().unwrap();
        let event_loop = epoller.event_loop(100);
        event_loop.ping();
        event_loop.stop();
    }
}
