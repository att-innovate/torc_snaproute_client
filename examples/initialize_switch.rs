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

// Simple example client to initalize switch with predefined configuration
//

extern crate torc_snaproute_client;

use torc_snaproute_client::api;
use std::env;

fn main() {
    let mut snaproute = "127.0.0.1:8080".to_string();
    let mut config_file = "./examples/config.yml".to_string();

    let args: Vec<_> = env::args().collect();
    if args.len() == 3 {
        snaproute = args[1].clone();
        config_file = args[2].clone();
    }

    println!("Connects to: {}, initalizes switch with config: {}", snaproute, config_file);

    api::reset_and_initalize(&snaproute, &config_file);
}
