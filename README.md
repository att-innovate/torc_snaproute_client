## torc_snaproute_client 

Simple SnapRoute client written in Rust.

We use it in our ToRC project, further information can be found at: [https://att-innovate.github.io/torc/](https://att-innovate.github.io/torc/) 

Build library:

	cargo build

The example folder contains some sample code.
To run follow steps below and replace `127.0.0.1:8080` with the connection arguments for your SnapRoute API services.

Build and run example list_port_stats:
	
	cargo build --example list_port_stats
	cargo run --example list_port_stats 127.0.0.1:8080

Build and run example list_routes:
	
	cargo build --example list_routes
	cargo run --example list_routes 127.0.0.1:8080

Build and run example modify_routes. Adjust IP addresses of the routes according to your network setup:
	
	cargo build --example modify_routes
	cargo run --example modify_routes 127.0.0.1:8080

Build and run switch initialization.
Will reset the switch to a default configuration defined in ./examples/config.yml. Adjust config.yml according to your own network setup:
	
	cargo build --example initialize_switch
	cargo run --example initialize_switch 127.0.0.1:8080 ./examples/config.yml
