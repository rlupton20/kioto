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

pub fn close(fd: i32) -> io::Result<()> {
    let ret = unsafe { libc::close(fd) };
    if ret < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "close file failed"
        ))
    } else {
        Ok(())
    }
}

pub fn write(fd: i32, buf: *const libc::c_void, count: usize) -> io::Result<usize> {
    let n = unsafe {
        libc::write(fd, buf, count)
    };

    if n < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "write failed"
            ))
    } else {
        Ok(n as usize)
    }
}

pub fn read(fd: i32, buf: *mut libc::c_void, size: usize) -> io::Result<usize> {
    let n = unsafe {
        libc::read(fd, buf, size)
    };

    if n < 0 {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "read failed"
        ))
    } else {
        Ok(n as usize)
    }


}
