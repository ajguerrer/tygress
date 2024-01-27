#![allow(unsafe_code, clippy::upper_case_acronyms)]

use std::io;
use std::mem;
use std::os::raw::{c_char, c_int, c_short, c_ushort};
use std::time::Duration;

use libc::{
    __c_anonymous_ifr_ifru, bind, ifreq, sockaddr, sockaddr_ll, socklen_t, IFF_NO_PI, IFF_TAP,
    IFF_TUN, IF_NAMESIZE,
};
use rustix::{
    event::{PollFd, PollFlags},
    fd::{AsFd, AsRawFd},
    ioctl::{ioctl, BadOpcode, Setter, Updater, WriteOpcode},
    net::{AddressFamily, Protocol},
};

use super::Event;
use super::HardwareType;

//https://github.com/torvalds/linux/blob/4fbbed7872677b0a28ba8237169968171a61efbd/include/uapi/linux/if_tun.h#L34
type TUNSETIFF = WriteOpcode<b'T', 202, c_int>;
// https://github.com/torvalds/linux/blob/4fbbed7872677b0a28ba8237169968171a61efbd/include/uapi/linux/sockios.h#L74
type SIOCGIFMTU = BadOpcode<0x8921>;
// https://github.com/torvalds/linux/blob/4fbbed7872677b0a28ba8237169968171a61efbd/include/uapi/linux/sockios.h#L85
type SIOCGIFINDEX = BadOpcode<0x8933>;

pub fn ifreq_name(name: &str) -> [c_char; IF_NAMESIZE] {
    let mut ifreq_name = [b'\0' as c_char; IF_NAMESIZE];
    for (index, byte) in name
        .as_bytes()
        .iter()
        // don't want the '\0'
        .take(IF_NAMESIZE - 1)
        .enumerate()
    {
        ifreq_name[index] = *byte as c_char;
    }
    ifreq_name
}

pub fn ioctl_tunsetiff<Fd: AsFd>(
    fd: Fd,
    hw_type: HardwareType,
    ifreq_name: [c_char; IF_NAMESIZE],
) -> io::Result<()> {
    let ifru_flags = match hw_type {
        HardwareType::Opaque => IFF_TUN as c_short,
        HardwareType::EthernetII => IFF_TAP as c_short,
        HardwareType::Ieee802154 => {
            return Err(io::Error::other("TUN/TAP does not support IEEE 802.15.4"))
        }
    } | IFF_NO_PI as c_short;

    let ifreq = ifreq {
        ifr_name: ifreq_name,
        ifr_ifru: __c_anonymous_ifr_ifru { ifru_flags },
    };

    unsafe { ioctl(fd, Setter::<TUNSETIFF, ifreq>::new(ifreq))? };

    Ok(())
}

pub fn ioctl_siocgifmtu<Fd: AsFd>(fd: Fd, ifreq_name: [c_char; IF_NAMESIZE]) -> io::Result<usize> {
    let mut ifreq = ifreq {
        ifr_name: ifreq_name,
        ifr_ifru: __c_anonymous_ifr_ifru { ifru_mtu: 0 },
    };

    unsafe {
        ioctl(fd, Updater::<SIOCGIFMTU, ifreq>::new(&mut ifreq))?;
        Ok(ifreq.ifr_ifru.ifru_mtu as usize)
    }
}

pub fn ioctl_siocgifindex<Fd: AsFd>(fd: Fd, ifreq_name: [c_char; IF_NAMESIZE]) -> io::Result<i32> {
    let mut ifreq = ifreq {
        ifr_name: ifreq_name,
        ifr_ifru: __c_anonymous_ifr_ifru { ifru_ifindex: 0 },
    };

    unsafe {
        ioctl(fd, Updater::<SIOCGIFINDEX, ifreq>::new(&mut ifreq))?;
        Ok(ifreq.ifr_ifru.ifru_ifindex)
    }
}

pub fn bind_interface<Fd: AsFd>(
    fd: Fd,
    protocol: Protocol,
    ifreq_name: [c_char; IF_NAMESIZE],
) -> io::Result<()> {
    let linkaddr = sockaddr_ll {
        sll_family: AddressFamily::PACKET.as_raw(),
        sll_protocol: protocol.as_raw().get() as c_ushort,
        sll_ifindex: ioctl_siocgifindex(&fd, ifreq_name)?,
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
