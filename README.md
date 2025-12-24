# dns_resolver
[![Ask DeepWiki](https://devin.ai/assets/askdeepwiki.png)](https://deepwiki.com/venoosoo/dns_resolver)

A simple, command-line DNS resolver implemented in Rust. This tool allows you to perform DNS queries for various record types against a specified DNS server. It is built from scratch, handling the creation of DNS query packets and the parsing of response packets.

## Features

*   Query for `A`, `AAAA`, `MX`, and `TXT` records.
*   Specify one or more target domains in a single command.
*   Option to use a custom DNS server.
*   Asynchronous network operations using Tokio.
*   Parses DNS responses, including support for compressed domain names.
*   Provides clear, human-readable output for query results.

## Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install) and Cargo must be installed.

## Installation & Building

1.  Clone the repository:
    ```sh
    git clone https://github.com/venoosoo/dns_resolver.git
    cd dns_resolver
    ```

2.  Build the project. For a development build:
    ```sh
    cargo build
    ```
    For an optimized release build:
    ```sh
    cargo build --release
    ```
    The executable will be located at `target/debug/dns` or `target/release/dns`.

## Usage

You can run the resolver using `cargo run` or by executing the compiled binary directly.

### Command-Line Arguments

*   `--target <DOMAIN...>`: (Required) One or more domain names to query.
*   `--type <RECORD_TYPE>`: The type of DNS record to query for. Defaults to `A`.
    *   Possible values: `a`, `mx`, `txt`, `aaaa`
*   `--server <IP_ADDRESS>`: The IP address of the DNS server to use. Defaults to `1.1.1.1`.

### Examples

**Query for an A record (default):**
```sh
cargo run -- --target google.com
```

**Query for AAAA (IPv6) records for multiple domains:**
```sh
cargo run -- --target google.com github.com --type aaaa
```

**Query for MX (Mail Exchange) records using Google's DNS server:**
```sh
cargo run -- --target proton.me --type mx --server 8.8.8.8
```
**Example Output:**
```
$ cargo run -- --target google.com --type a
   Compiling dns v0.1.0 (/path/to/dns_resolver)
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/dns --target google.com --type a`
The results for google.com
Got answer records
A     142.250.180.142   TTL 143
```

## Project Structure

The project is organized into several modules to handle different aspects of the DNS resolution process:

*   `src/main.rs`: The main entry point. Handles command-line argument parsing with `clap` and orchestrates the query process.
*   `src/dns_header.rs`: Defines the `DnsHeader` struct and logic for creating and writing the header part of a DNS query packet.
*   `src/dns_question.rs`: Defines the `DnsQuestion` struct for building the "question" section of a DNS query, including domain name encoding.
*   `src/dns_packet.rs`: A simple container that combines a `DnsHeader` and `DnsQuestion` to form a complete query packet.
*   `src/parse_answer.rs`: Contains the logic for parsing the binary response from a DNS server. It handles different record types (`A`, `AAAA`, `MX`), decodes compressed names, and prints the results in a user-friendly format.
