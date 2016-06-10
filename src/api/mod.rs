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
use hyper::status::StatusCode;
use hyper::header::ContentType;
use std::io::Read;
use rustc_serialize::json::{self, Json};

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
    let mut response = client.get(&address).send().unwrap();

    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();

    match response.status {
        StatusCode::Ok => {
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

                result.push(PortStat{id: id as i32 + 1, connected: connected})
            }
        }
        _ => println!("error getting portstats"),
    }

    result
}

pub fn get_routes(connect_string: &str) -> Vec<Route> {
    let mut result = vec![];

    let client = Client::new();
    let address = format!("http://{}/public/v1/state/IPv4Routes", connect_string);
    let mut response = client.get(&address).send().unwrap();

    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();

    match response.status {
        StatusCode::Ok => {
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
                
                result.push(Route{from: from, to: to})
            }
        }
        _ => println!("error getting routes"),
    }

    result
}

pub fn reset_routes(_connect_string: &str) {
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
    let nexthop = NextHopInfo{NextHopIp: route_to.to_string()};
    let ipv4route = IPv4Route{
        DestinationNw: ip,
        NetworkMask: mask,
        Protocol: "STATIC".to_string(),
        NextHop: vec![nexthop]
    };

    let data = json::encode(&ipv4route).unwrap();
    let client = Client::new();
    let address = format!("http://{}/public/v1/config/IPv4Route", connect_string);
    let _ = client.post(&address).body(&data).header(ContentType::json()).send();
}

pub fn delete_route(connect_string: &str, route_from: &str) {
    let (ip, mask) = split_address_into_ip_and_mask(&route_from);
    let ipv4route = IPv4Route{
        DestinationNw: ip,
        NetworkMask: mask,
        Protocol: "STATIC".to_string(),
        NextHop: vec![]
    };

    let data = json::encode(&ipv4route).unwrap();
    let client = Client::new();
    let address = format!("http://{}/public/v1/config/IPv4Route", connect_string);
    let _ = client.delete(&address).body(&data).header(ContentType::json()).send();    
}

fn split_address_into_ip_and_mask(address: &str) -> (String, String) {
    let len = address.len();
    let mut ip = address.to_string();
    let mut mask = "255.255.255.255".to_string();
    if address.ends_with("/32") {
        ip.truncate(len-3);
    } else if address.ends_with("/24") {
        ip.truncate(len-3);
        mask = "255.255.255.0".to_string()
    }
    (ip.clone(), mask.clone())
}


