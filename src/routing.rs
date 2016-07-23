use std::net::Ipv4Addr;
use std::collections::HashMap;

use ipnetwork::Ipv4Network;

use Interface;

// TODO: Add metric
struct Entry {
    pub net: Ipv4Network,
    pub gw: Option<Ipv4Addr>,
    pub interface: Interface,
}

pub struct RoutingTable {
    prefixes: Vec<u8>,
    table: HashMap<u8, Vec<Entry>>,
}

impl RoutingTable {
    pub fn new() -> RoutingTable {
        RoutingTable {
            prefixes: vec![],
            table: HashMap::new(),
        }
    }

    // TODO: Check for collision
    pub fn add_route(&mut self, net: Ipv4Network, gw: Option<Ipv4Addr>, interface: Interface) {
        let prefix = net.prefix();
        let entry = Entry {
            net: net,
            gw: gw,
            interface: interface,
        };
        if !self.table.contains_key(&prefix) {
            self.prefixes.push(prefix);
            self.prefixes.sort_by(|a, b| b.cmp(a));
            self.table.insert(prefix, vec![entry]);
        } else {
            self.table.get_mut(&prefix).unwrap().push(entry);
        }
    }

    pub fn route(&self, ip: Ipv4Addr) -> Option<(Option<Ipv4Addr>, Interface)> {
        for prefix in self.prefixes.iter() {
            for entry in self.table.get(&prefix).unwrap() {
                if entry.net.contains(ip) {
                    return Some((entry.gw.clone(), entry.interface.clone()));
                }
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use pnet::util::MacAddr;
    use ipnetwork::Ipv4Network;

    use Interface;
    use super::*;

    #[test]
    fn empty() {
        let table = RoutingTable::new();
        assert!(table.route(Ipv4Addr::new(10, 0, 0, 1)).is_none());
        assert!(table.route(Ipv4Addr::new(0, 0, 0, 0)).is_none());
    }

    #[test]
    fn no_default() {
        let mut table = RoutingTable::new();
        table.add_route(Ipv4Network::from_cidr("10/8").unwrap(),
                        None,
                        iface("eth0"));
        let (gw, out_eth) = table.route(Ipv4Addr::new(10, 0, 0, 1)).unwrap();
        assert_eq!(gw, None);
        assert_eq!(out_eth, iface("eth0"));
        assert!(table.route(Ipv4Addr::new(192, 168, 0, 0)).is_none());
    }

    #[test]
    fn with_default() {
        let gw = Ipv4Addr::new(10, 0, 0, 1);

        let mut table = RoutingTable::new();
        table.add_route(Ipv4Network::from_cidr("10/16").unwrap(),
                        None,
                        iface("eth0"));
        table.add_route(Ipv4Network::from_cidr("0/0").unwrap(),
                        Some(gw),
                        iface("eth1"));

        let (out_gw, out_eth) = table.route(Ipv4Addr::new(10, 0, 200, 20)).unwrap();
        assert_eq!(out_gw, None);
        assert_eq!(out_eth, iface("eth0"));
        let (out_gw2, out_eth2) = table.route(Ipv4Addr::new(192, 168, 0, 0)).unwrap();
        assert_eq!(out_gw2, Some(gw));
        assert_eq!(out_eth2, iface("eth1"));
    }

    #[test]
    fn with_specific() {
        let gw = Ipv4Addr::new(10, 0, 0, 1);

        let mut table = RoutingTable::new();
        table.add_route(Ipv4Network::from_cidr("10.0.0.0/24").unwrap(),
                        None,
                        iface("eth0"));
        table.add_route(Ipv4Network::from_cidr("10.0.0.99/32").unwrap(),
                        Some(gw),
                        iface("eth1"));

        let (out_gw, out_eth) = table.route(Ipv4Addr::new(10, 0, 0, 20)).unwrap();
        assert_eq!(out_gw, None);
        assert_eq!(out_eth, iface("eth0"));
        let (out_gw2, out_eth2) = table.route(Ipv4Addr::new(10, 0, 0, 99)).unwrap();
        assert_eq!(out_gw2, Some(gw));
        assert_eq!(out_eth2, iface("eth1"));
    }

    fn iface(name: &str) -> Interface {
        Interface {
            name: name.to_string(),
            mac: MacAddr::new(0, 0, 0, 0, 0, 0),
        }
    }
}
