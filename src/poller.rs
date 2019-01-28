use libc;
use std::io;

pub struct Epoller {
    epoll_fd: i32,
    mgmt_event_fd: i32
}

impl Epoller {
    pub fn new() -> io::Result<Epoller> {
        Ok(Epoller{
            epoll_fd: Epoller::epoll_create()?,
            mgmt_event_fd: Epoller::eventfd(0, 0)?
        })
    }

    fn epoll_create() -> io::Result<i32> {
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

    fn eventfd(init: u32, flags: i32) -> io::Result<i32> {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_can_create_epoller() {
        let epoller = Epoller::new().unwrap();
    }
}
