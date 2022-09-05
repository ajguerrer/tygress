#![allow(unsafe_code)]
#![allow(non_camel_case_types, non_snake_case, deref_nullptr)]

use std::io;
use std::mem;
use std::os::raw::{c_char, c_short, c_ulong, c_ushort};
use std::time::Duration;

use rustix::fd::{AsFd, AsRawFd};
use rustix::io::{PollFd, PollFlags};
use rustix::net::AddressFamily;

use super::Event;
use super::Topology;

#[cfg(all(feature = "bindgen", not(feature = "overwrite")))]
include!(concat!(env!("OUT_DIR"), "/sys.rs"));

#[cfg(any(
    not(feature = "bindgen"),
    all(feature = "bindgen", feature = "overwrite")
))]
include!("sys.rs");

pub fn ifreq_name(name: &str) -> [c_char; IF_NAMESIZE as usize] {
    let mut ifreq_name = [b'\0' as i8; IF_NAMESIZE as usize];
    for (index, byte) in name
        .as_bytes()
        .iter()
        // last byte must be '\0'
        .take(IF_NAMESIZE as usize - 1)
        .enumerate()
    {
        ifreq_name[index] = *byte as c_char;
    }
    ifreq_name
}

pub fn ioctl_tunsetiff<Fd: AsFd>(
    fd: Fd,
    topology: Topology,
    ifreq_name: [c_char; IF_NAMESIZE as usize],
) -> io::Result<()> {
    let ifru_flags = match topology {
        Topology::Ip => IFF_TUN as c_short,
        Topology::EthernetII => IFF_TAP as c_short,
    } | IFF_NO_PI as c_short;

    let ifreq = ifreq {
        ifr_ifrn: ifreq__bindgen_ty_1 {
            ifrn_name: ifreq_name,
        },
        ifr_ifru: ifreq__bindgen_ty_2 { ifru_flags },
    };

    let result = unsafe {
        ioctl(
            fd.as_fd().as_raw_fd(),
            linux_raw_sys::ioctl::TUNSETIFF as c_ulong,
            &ifreq,
        )
    };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub fn ioctl_siocgifmtu<Fd: AsFd>(
    fd: Fd,
    ifreq_name: [c_char; IF_NAMESIZE as usize],
) -> io::Result<usize> {
    let mut ifreq = ifreq {
        ifr_ifrn: ifreq__bindgen_ty_1 {
            ifrn_name: ifreq_name,
        },
        ifr_ifru: ifreq__bindgen_ty_2 { ifru_mtu: 0 },
    };

    let result = unsafe { ioctl(fd.as_fd().as_raw_fd(), SIOCGIFMTU as c_ulong, &mut ifreq) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(unsafe { ifreq.ifr_ifru.ifru_mtu } as usize)
}

pub fn ioctl_siocgifindex<Fd: AsFd>(
    fd: Fd,
    ifreq_name: [c_char; IF_NAMESIZE as usize],
) -> io::Result<i32> {
    let mut ifreq = ifreq {
        ifr_ifrn: ifreq__bindgen_ty_1 {
            ifrn_name: ifreq_name,
        },
        ifr_ifru: ifreq__bindgen_ty_2 { ifru_ivalue: 0 },
    };

    let result = unsafe { ioctl(fd.as_fd().as_raw_fd(), SIOCGIFINDEX as c_ulong, &mut ifreq) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(unsafe { ifreq.ifr_ifru.ifru_ivalue })
}

pub fn bind_interface<Fd: AsFd>(
    fd: Fd,
    ifreq_name: [c_char; IF_NAMESIZE as usize],
) -> io::Result<()> {
    let index = ioctl_siocgifindex(&fd, ifreq_name)?;

    let linkaddr = sockaddr_ll {
        sll_family: AddressFamily::PACKET.as_raw(),
        // Equivalent to htons(libc::ETH_P_ALL).
        // To keep your sanity, make sure the integer width is 16 bits first!
        sll_protocol: (ETH_P_ARP as c_ushort).to_be(),
        sll_ifindex: index,
        sll_hatype: 1,
        sll_pkttype: 0,
        sll_halen: 6,
        sll_addr: [0; 8],
    };

    let result = unsafe {
        bind(
            fd.as_fd().as_raw_fd(),
            &linkaddr as *const sockaddr_ll as *const sockaddr,
            mem::size_of::<sockaddr_ll>() as socklen_t,
        )
    };

    if result == -1 {
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
    rustix::io::poll(
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
