# IP Geolocation and Port Scanner

This Rust-based web application allows users to perform IP geolocation lookups and port scanning on multiple IP addresses simultaneously. It also detects if the IP belongs to a major cloud provider.

## Features

- IP geolocation lookup
- Port scanning
- Cloud provider detection
- Concurrent processing of multiple IP addresses
- Web-based user interface

## Setup

1. Clone the repository
2. Install Rust and Cargo
3. Run `cargo build` to compile the project
4. Start the server with `cargo run`
5. Open `index.html` in a web browser

## Usage

Enter IP addresses (one per line) in the text area, choose whether to use the default output directory, and click "Process IPs". The application will return geolocation information, active status, open ports, and cloud provider information for each IP address.

## Note

Ensure you have the necessary permissions before scanning IP addresses or ports that you do not own or have explicit permission to test.
