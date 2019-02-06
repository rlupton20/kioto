mod cfuns;
mod owned_handler;

use event_handler::EventHandler;
use self::owned_handler::OwnedHandler;

use std::io;
use std::sync;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use libc::{EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD};
use libc;


const MAX_EVENTS: usize = 256;

// Re-export event flags
pub const EPOLLIN: u32 = libc::EPOLLIN as u32;
pub const EPOLLOUT: u32 = libc::EPOLLOUT as u32;
pub const EPOLLET: u32 = libc::EPOLLET as u32;


struct MgmtFd {
    mgmt_fd: i32
}

impl EventHandler for MgmtFd {
    fn handle_event(&mut self, _: u32) {
        let mut buf: u64 = 0;
        let buf_ref = &mut buf as *mut u64;
        cfuns::read(self.mgmt_fd, buf_ref as *mut libc::c_void, 8);
        ()
    }
}

impl MgmtFd {
    fn new(mgmt_fd: i32) -> MgmtFd {
        MgmtFd {
            mgmt_fd: mgmt_fd
        }
    }
}

impl Drop for MgmtFd {
    fn drop(&mut self) -> () {
        cfuns::close(self.mgmt_fd);
    }
}


struct EventLoop {
    epoll_fd: i32,
    mgmt_fd: i32,
    mgmt_handler: OwnedHandler,
    stop_flag: sync::Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>
}

impl Drop for EventLoop {
    fn drop(&mut self) -> () {
        cfuns::close(self.epoll_fd);
        self.join_handler_thread();
    }
}

impl EventLoop {
    fn new() -> io::Result<EventLoop> {
        let epoll_fd = cfuns::epoll_create()?;
        let mgmt_fd = cfuns::eventfd(0,0)?;
        let mgmt_handler: Box<dyn EventHandler> = Box::new(MgmtFd::new(mgmt_fd));
        let owned_handler = OwnedHandler::new(mgmt_handler);
        let stop_flag_owned = sync::Arc::new(AtomicBool::new(false));
        let stop_flag_given = sync::Arc::clone(&stop_flag_owned);

        // Register the management event file descriptor
        let mut event = cfuns::epoll_event(EPOLLIN as u32 | EPOLLET as u32, owned_handler.as_ptr() as u64);
        cfuns::epoll_ctl(epoll_fd, EPOLL_CTL_ADD, mgmt_fd, &mut event)?;

        let worker = thread::spawn(move || {

            let empty_event: libc::epoll_event = libc::epoll_event {
                events: 0,
                u64: 0
            };

            let mut events: [libc::epoll_event; MAX_EVENTS] = [empty_event; MAX_EVENTS];

            while stop_flag_given.load(Ordering::SeqCst) != true {
                EventLoop::process_events(epoll_fd, &mut events);
            }

        });

        let event_loop = EventLoop {
            epoll_fd: epoll_fd,
            mgmt_fd: mgmt_fd,
            mgmt_handler: owned_handler,
            stop_flag: stop_flag_owned,
            thread: Some(worker)
        };

        Ok(event_loop)
    }

    pub fn stop(mut self) -> io::Result<()> {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.ping();
        self.join_handler_thread()
    }

    fn join_handler_thread(&mut self) -> io::Result<()> {
        match self.thread.take() {
            Some(handle) => handle.join()
                .map_err(|_| io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to wait for event_loop to join"
                )),
            None => Ok(())  // Thread already stopped
        }
    }

    fn ping(&self) {
        let buf: u64 = 1;
        let buf_ptr = &buf as *const u64;
        cfuns::write(self.mgmt_fd, buf_ptr as *const libc::c_void, 8);
    }

    fn process_events(epoll_fd: i32, events: &mut [libc::epoll_event; MAX_EVENTS]) -> io::Result<()>{
        let num_events = cfuns::epoll_wait(epoll_fd, events, MAX_EVENTS as i32, 100)?;
        for ix in 0..num_events {
            let event = events[ix as usize];
            OwnedHandler::handle_event(event.u64 as *mut libc::c_void, event.events);
        }
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_event_loop() -> io::Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.stop()?;
        Ok(())
    }
}
