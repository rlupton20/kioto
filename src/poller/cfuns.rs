/// cfuns
/// Safe wrappers for C functions
use libc;
use std::io;


pub fn epoll_create() -> io::Result<i32> {
    let fd = unsafe { libc::epoll_create(1) };
    if fd < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "epoll_create(1) failed"
        ))
    } else {
        Ok(fd)
    }
}


pub fn epoll_ctl(epoll_fd: i32, op: i32, fd: i32, event: &mut libc::epoll_event) -> io::Result<()> {
    let rc = unsafe {
        libc::epoll_ctl(epoll_fd, op, fd, event as *mut libc::epoll_event)
    };
    if rc < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "epoll_ctl failed"
        ))
    } else {
        Ok(())
    }
}


pub fn epoll_event(events: u32, u64: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: events,
        u64: u64
    }
}


pub fn epoll_wait(epoll_fd: i32,
                  events: &mut [libc::epoll_event],
                  max_events: i32,
                  timeout: i32) -> io::Result<u32> {
    let num_events = unsafe {
        libc::epoll_wait(
            epoll_fd,
            events.as_mut_ptr(),
            max_events,
            timeout
        )
    };

    if num_events < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "epoll_wait failed"
        ))
    } else {
        Ok(num_events as u32)
    }
}


pub fn eventfd(init: u32, flags: i32) -> io::Result<i32> {
    let fd = unsafe { libc::eventfd(init, flags) };
    if fd < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "eventfd create failed"
        ))
    } else {
        Ok(fd)
    }
}
