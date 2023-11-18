#[cfg(not(feature = "bindgen"))]
fn main() {}

#[cfg(feature = "bindgen")]
fn main() {
    use std::env;
    use std::path::PathBuf;

    const INCLUDE: &str = r#"
#include <net/if.h>
#include <netpacket/packet.h>
#include <sys/ioctl.h>
#include <asm-generic/ioctl.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
    "#;

    #[cfg(not(feature = "overwrite"))]
    let outdir = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "overwrite")]
    let outdir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/netdev/sys");

    bindgen::Builder::default()
        .header_contents("include-file.h", INCLUDE)
        .allowlist_type("(ifreq|sockaddr|sockaddr_ll|socklen_t)")
        .allowlist_function("(bind|ioctl)")
        .allowlist_var("(SIOCGIFINDEX|SIOCGIFMTU|IF_NAMESIZE|IFF_TUN|IFF_TAP|IFF_NO_PI|ETH_P_ARP)")
        .layout_tests(false)
        .generate()
        .unwrap()
        .write_to_file(outdir.join("sys.rs"))
        .unwrap();
}
