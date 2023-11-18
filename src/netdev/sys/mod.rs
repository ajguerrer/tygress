#![allow(unsafe_code)]

use core::{mem, time::Duration};
use std::io;

use rustix::{
    event::{PollFd, PollFlags},
    fd::{AsFd, AsRawFd},
    ioctl::{BadOpcode, Setter, WriteOpcode},
};

use crate::netdev::sys::ifreq_ioctl::IfreqGetter;

use super::Event;

mod ifreq_ioctl;

pub fn ifreq_name(name: &str) -> [libc::c_char; libc::IFNAMSIZ] {
    let mut ifreq_name = [b'\0' as i8; libc::IFNAMSIZ];
    for (index, byte) in name
        .as_bytes()
        .iter()
        // last byte must be '\0'
        .take(libc::IFNAMSIZ - 1)
        .enumerate()
    {
        ifreq_name[index] = *byte as libc::c_char;
    }
    ifreq_name
}

pub fn htons(hostshort: libc::c_ushort) -> libc::c_ushort {
    hostshort.to_be()
}

pub fn ioctl_tunsetiff<Fd: AsFd>(
    fd: Fd,
    ifru_flags: libc::c_int,
    ifreq_name: [libc::c_char; libc::IFNAMSIZ],
) -> io::Result<()> {
    let ifreq = libc::ifreq {
        ifr_name: ifreq_name,
        ifr_ifru: libc::__c_anonymous_ifr_ifru {
            ifru_flags: ifru_flags as libc::c_short,
        },
    };

    type Tunsetiff = Setter<WriteOpcode<b'T', 202, libc::c_int>, libc::ifreq>;
    unsafe { Ok(rustix::ioctl::ioctl(fd, Tunsetiff::new(ifreq))?) }
}

pub fn ioctl_siocgifmtu<Fd: AsFd>(
    fd: Fd,
    ifreq_name: [libc::c_char; libc::IFNAMSIZ],
) -> io::Result<usize> {
    let mut ifreq = libc::ifreq {
        ifr_name: ifreq_name,
        ifr_ifru: libc::__c_anonymous_ifr_ifru { ifru_mtu: 0 },
    };

    type Siocgifmtu<'a> = IfreqGetter<'a, BadOpcode<{ libc::SIOCGIFMTU as libc::c_uint }>>;
    unsafe {
        rustix::ioctl::ioctl(fd, Siocgifmtu::new(&mut ifreq))?;
        Ok(ifreq.ifr_ifru.ifru_mtu as usize)
    }
}

pub fn ioctl_siocgifindex<Fd: AsFd>(
    fd: Fd,
    ifreq_name: [libc::c_char; libc::IFNAMSIZ],
) -> io::Result<i32> {
    let mut ifreq = libc::ifreq {
        ifr_name: ifreq_name,
        ifr_ifru: libc::__c_anonymous_ifr_ifru { ifru_ifindex: 0 },
    };

    type Siocgifindex<'a> = IfreqGetter<'a, BadOpcode<{ libc::SIOCGIFINDEX as libc::c_uint }>>;
    unsafe {
        rustix::ioctl::ioctl(fd, Siocgifindex::new(&mut ifreq))?;
        Ok(ifreq.ifr_ifru.ifru_ifindex)
    }
}

pub fn bind_interface<Fd: AsFd>(
    fd: Fd,
    ifreq_name: [libc::c_char; libc::IFNAMSIZ],
    protocol: libc::c_int,
) -> io::Result<()> {
    let index = ioctl_siocgifindex(&fd, ifreq_name)?;

    let linkaddr = libc::sockaddr_ll {
        sll_family: libc::AF_PACKET as libc::c_ushort,
        sll_protocol: htons(protocol as libc::c_ushort),
        sll_ifindex: index,
        // rest is just default values
        sll_hatype: 0,
        sll_pkttype: 0,
        sll_halen: 0,
        sll_addr: [0; 8],
    };

    let address = &linkaddr as *const libc::sockaddr_ll as *const libc::sockaddr;
    let address_len = mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t;
    if unsafe { libc::bind(fd.as_fd().as_raw_fd(), address, address_len) } == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
pub fn poll<Fd: AsFd>(fd: Fd, interest: Event, timeout: Option<Duration>) -> io::Result<Event> {
    let mut flags = PollFlags::empty();
    if interest.is_readable() {
        flags |= PollFlags::IN;
    }
    if interest.is_writable() {
        flags |= PollFlags::OUT
    }

    let fds = &mut [PollFd::new(&fd, flags)];
    rustix::event::poll(
        fds,
        match timeout {
            Some(timeout) => timeout.as_millis() as i32,
            None => -1, // negative value means wait indefinitely
        },
    )?;

    let mut event = Event::new();
    if fds[0].revents().contains(PollFlags::IN) {
        event |= Event::READABLE;
    }
    if fds[0].revents().contains(PollFlags::OUT) {
        event |= Event::WRITABLE;
    }

    Ok(event)
}
