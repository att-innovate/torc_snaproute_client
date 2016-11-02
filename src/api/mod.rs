// The MIT License (MIT)
//
// Copyright (c) 2015 AT&T
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use hyper::Client;
use hyper::header::ContentType;
use hyper::status::StatusCode;
use rustc_serialize::json::{self, Json};
use std::fs::File;
use std::io::Read;
use yaml_rust::{Yaml, YamlLoader};

macro_rules! log_request_error {
    ($expr:expr) => (match $expr {
        Result::Ok(response) => {
            if response.status.to_u16() >= 400 {
                println!("error code {}", response.status);
            };
            Some(response)
        },
        Result::Err(err) => {
            println!("error {}", err);
            None
        }
    })
}

pub struct PortStat {
    pub id: i32,
    pub connected: bool,
}

pub struct Route {
    pub from: String,
    pub to: String,
}

pub fn get_ports_stats(connect_string: &str) -> Vec<PortStat> {
    let mut result = vec![];

    let client = Client::new();
    let address = format!("http://{}/public/v1/state/Ports", connect_string);

    let mut response = match log_request_error!(client.get(&address).send()) {
        Some(response) => response,
        None => return result,
    };

    match response.status {
        StatusCode::Ok => {
            let mut body = String::new();
            response.read_to_string(&mut body).unwrap();

            let jsondata = Json::from_str(&body.clone()).unwrap();
            let snap_objects_raw = jsondata.as_object().unwrap();
            let snap_objects = snap_objects_raw.get("Objects").unwrap().as_array().unwrap();
            for snap_object in snap_objects.iter() {
                let mut connected = false;
                let id = snap_object.search("IfIndex").unwrap().as_u64().unwrap();

                let port_status = snap_object.search("OperState").unwrap();
                if port_status.as_string().unwrap() == "UP" {
                    connected = true
                }

                result.push(PortStat {
                    id: id as i32,
                    connected: connected,
                })
            }
        }
        _ => println!("error code {}", response.status),
    }

    result
}

pub fn get_routes(connect_string: &str) -> Vec<Route> {
    let mut result = vec![];

    let client = Client::new();
    let address = format!("http://{}/public/v1/state/IPv4Routes", connect_string);

    let mut response = match log_request_error!(client.get(&address).send()) {
        Some(response) => response,
        None => return result,
    };

    match response.status {
        StatusCode::Ok => {
            let mut body = String::new();
            response.read_to_string(&mut body).unwrap();

            let jsondata = Json::from_str(&body.clone()).unwrap();
            let snap_objects_raw = jsondata.as_object().unwrap();
            let snap_objects = snap_objects_raw.get("Objects").unwrap().as_array().unwrap();
            for snap_object in snap_objects.iter() {
                let from = snap_object.search("DestinationNw").unwrap().to_string();
                let mut to = "".to_string();

                let nexthoplist = snap_object.search("NextHopList").unwrap().as_array().unwrap();
                if nexthoplist.len() > 0 {
                    to = nexthoplist[0].search("NextHopIp").unwrap().to_string();
                }

                result.push(Route {
                    from: from,
                    to: to,
                })
            }
        }
        _ => println!("error code {}", response.status),
    }

    result
}

pub fn reset_routes(_connect_string: &str) {
    println!("reset routes not implemented for snaproute")
}


#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct Port {
    pub IntfRef: String,
    pub BreakOutMode: String,
}

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct SubPort {
    pub IntfRef: String,
    pub Speed: i32,
    pub AdminState: String,
}

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct Vlan {
    pub VlanId: i32,
    pub UntagIntfList: Vec<String>,
}

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct IPv4Intf {
    pub IntfRef: String,
    pub IpAddr: String,
}

pub fn reset_and_initalize(connect_string: &str, config_file: &str) {
    let client = Client::new();

    let reset_config = format!("http://{}/public/v1/action/ResetConfig", connect_string);
    log_request_error!(client.post(&reset_config).send());

    if config_file.is_empty() {
        return;
    }

    let config_port = format!("http://{}/public/v1/config/Port", connect_string);
    let config_vlan = format!("http://{}/public/v1/config/Vlan", connect_string);
    let config_interface = format!("http://{}/public/v1/config/IPv4Intf", connect_string);

    let config = read_config_file(config_file);

    let ports = read_ports(&config);
    let sub_ports = read_sub_ports(&config);
    let vlans = read_vlans(&config);
    let interfaces = read_ipv4intf(&config);

    for port in ports {
        let data = json::encode(&port).unwrap();
        log_request_error!(client.patch(&config_port).body(&data).header(ContentType::json()).send());
    }

    for sub_port in sub_ports {
        let data = json::encode(&sub_port).unwrap();
        log_request_error!(client.patch(&config_port).body(&data).header(ContentType::json()).send());
    }

    for vlan in vlans {
        let data = json::encode(&vlan).unwrap();
        log_request_error!(client.post(&config_vlan).body(&data).header(ContentType::json()).send());
    }

    for interface in interfaces {
        let data = json::encode(&interface).unwrap();
        log_request_error!(client.post(&config_interface).body(&data).header(ContentType::json()).send());
    }

}

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct NextHopInfo {
    pub NextHopIp: String,
}

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct IPv4Route {
    pub DestinationNw: String,
    pub NetworkMask: String,
    pub Protocol: String,
    pub NextHop: Vec<NextHopInfo>,
}

pub fn add_route(connect_string: &str, route_from: &str, route_to: &str) {
    let (ip, mask) = split_address_into_ip_and_mask(&route_from);
    let nexthop = NextHopInfo { NextHopIp: route_to.to_string() };
    let ipv4route = IPv4Route {
        DestinationNw: ip,
        NetworkMask: mask,
        Protocol: "STATIC".to_string(),
        NextHop: vec![nexthop],
    };

    let data = json::encode(&ipv4route).unwrap();
    let client = Client::new();
    let address = format!("http://{}/public/v1/config/IPv4Route", connect_string);
    log_request_error!(client.post(&address).body(&data).header(ContentType::json()).send());
}

pub fn delete_route(connect_string: &str, route_from: &str) {
    let (ip, mask) = split_address_into_ip_and_mask(&route_from);
    let ipv4route = IPv4Route {
        DestinationNw: ip,
        NetworkMask: mask,
        Protocol: "STATIC".to_string(),
        NextHop: vec![],
    };

    let data = json::encode(&ipv4route).unwrap();
    let client = Client::new();
    let address = format!("http://{}/public/v1/config/IPv4Route", connect_string);
    log_request_error!(client.delete(&address).body(&data).header(ContentType::json()).send());
}


fn read_config_file(config_file: &str) -> Yaml {
    let mut file = match File::open(config_file) {
        Ok(file) => file,
        Err(err) => panic!(err.to_string()),
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config = YamlLoader::load_from_str(&content).unwrap();
    // Multi document support, doc is a yaml::Yaml
    config[0].clone()
}

fn read_ports(config: &Yaml) -> Vec<Port> {
    let mut result = Vec::new();

    match config["ports"].is_badvalue() {
        true => {}
        false => {
            let ports = config["ports"].as_vec().unwrap();
            for port in ports {
                match port["mode"].is_badvalue() {
                    true => {}
                    false => {
                        let definition = Port {
                            IntfRef: port["name"].as_str().unwrap().to_string(),
                            BreakOutMode: port["mode"].as_str().unwrap().to_string(),
                        };
                        result.push(definition);
                    }
                }
            }
        }
    }

    result.clone()
}

fn read_sub_ports(config: &Yaml) -> Vec<SubPort> {
    let mut result = Vec::new();

    match config["ports"].is_badvalue() {
        true => {}
        false => {
            let ports = config["ports"].as_vec().unwrap();
            for port in ports {
                match port["speed"].is_badvalue() {
                    true => {}
                    false => {
                        let definition = SubPort {
                            IntfRef: port["name"].as_str().unwrap().to_string(),
                            Speed: port["speed"].as_i64().unwrap() as i32,
                            AdminState: "UP".to_string(),
                        };
                        result.push(definition);
                    }
                }
            }
        }
    }

    result.clone()
}

fn read_vlans(config: &Yaml) -> Vec<Vlan> {
    let mut result = Vec::new();

    match config["vlans"].is_badvalue() {
        true => {}
        false => {
            let vlans = config["vlans"].as_vec().unwrap();
            for vlan in vlans {
                let mut inf_list = Vec::new();
                inf_list.push(vlan["ports"].as_str().unwrap().to_string());
                let definition = Vlan {
                    VlanId: vlan["id"].as_i64().unwrap() as i32,
                    UntagIntfList: inf_list,
                };
                result.push(definition);
            }
        }
    }

    result.clone()
}

fn read_ipv4intf(config: &Yaml) -> Vec<IPv4Intf> {
    let mut result = Vec::new();

    match config["interfaces"].is_badvalue() {
        true => {}
        false => {
            let interfaces = config["interfaces"].as_vec().unwrap();
            for interface in interfaces {
                let definition = IPv4Intf {
                    IntfRef: format!("vlan{}", interface["vlan_id"].as_i64().unwrap()),
                    IpAddr: interface["addr"].as_str().unwrap().to_string(),
                };
                result.push(definition);
            }
        }
    }

    result.clone()
}

fn split_address_into_ip_and_mask(address: &str) -> (String, String) {
    let len = address.len();
    let mut ip = address.to_string();
    let mut mask = "255.255.255.255".to_string();
    if address.ends_with("/32") {
        ip.truncate(len - 3);
    } else if address.ends_with("/24") {
        ip.truncate(len - 3);
        mask = "255.255.255.0".to_string()
    }
    (ip.clone(), mask.clone())
}
