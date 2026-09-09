use std::{collections::BTreeSet, mem::size_of, net::Ipv4Addr};

use windows::{
    Win32::{
        Foundation::ERROR_BUFFER_OVERFLOW,
        NetworkManagement::{
            IpHelper::{
                GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST,
                GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
            },
            Ndis::IfOperStatusUp,
        },
        Networking::WinSock::{AF_INET, SOCKADDR, SOCKADDR_IN},
    },
    core::{Error, HRESULT, Result},
};

pub fn private_ipv4_addresses() -> Result<Vec<String>> {
    let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;
    let mut byte_count = 0u32;
    let initial =
        unsafe { GetAdaptersAddresses(AF_INET.0 as u32, flags, None, None, &mut byte_count) };
    if initial != ERROR_BUFFER_OVERFLOW.0 {
        return Err(win32_error(initial));
    }
    let word_count = (byte_count as usize).div_ceil(size_of::<usize>());
    let mut buffer = vec![0usize; word_count];
    let first = buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
    let result = unsafe {
        GetAdaptersAddresses(AF_INET.0 as u32, flags, None, Some(first), &mut byte_count)
    };
    if result != 0 {
        return Err(win32_error(result));
    }
    let addresses = unsafe { collect_addresses(first) };
    Ok(addresses
        .into_iter()
        .map(|address| address.to_string())
        .collect())
}

unsafe fn collect_addresses(mut adapter: *mut IP_ADAPTER_ADDRESSES_LH) -> BTreeSet<Ipv4Addr> {
    let mut addresses = BTreeSet::new();
    while let Some(current) = unsafe { adapter.as_ref() } {
        if current.OperStatus == IfOperStatusUp {
            let mut unicast = current.FirstUnicastAddress;
            while let Some(address) = unsafe { unicast.as_ref() } {
                if let Some(ipv4) = unsafe { ipv4_address(address.Address.lpSockaddr) }
                    && ipv4.is_private()
                {
                    addresses.insert(ipv4);
                }
                unicast = address.Next;
            }
        }
        adapter = current.Next;
    }
    addresses
}

unsafe fn ipv4_address(socket_address: *mut SOCKADDR) -> Option<Ipv4Addr> {
    let address = unsafe { socket_address.cast::<SOCKADDR_IN>().as_ref()? };
    if address.sin_family != AF_INET {
        return None;
    }
    let bytes = unsafe { address.sin_addr.S_un.S_un_b };
    Some(Ipv4Addr::new(
        bytes.s_b1, bytes.s_b2, bytes.s_b3, bytes.s_b4,
    ))
}

fn win32_error(code: u32) -> Error {
    Error::from_hresult(HRESULT::from_win32(code))
}
