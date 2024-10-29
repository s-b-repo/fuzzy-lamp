
# Secure Proxy Server with IP Allowlist and Session-Based Authentication

This Rust project is a secure proxy server built using Actix Web, Reqwest, and SQLx. It forwards incoming requests to a target server and includes the following security features:
- **IP Allowlist** stored in a SQL database.
- **Session-Based Authentication** via headers.
- **HTTPS Encryption** using TLS certificates.

## Prerequisites

- **Rust** (https://www.rust-lang.org/tools/install)
- **SQLite** (or any SQL database supported by SQLx)

## Installation

1. **Clone this repository:**
   ```
   git clone https://github.com/s-b-repo/fuzzy-lamp.git
   cd fuzzy-lamp

    Install Dependencies:

    

cargo build

Set Up Database: Initialize SQLite and create the allowlist table:

sql

CREATE TABLE allowlist (
    ip TEXT PRIMARY KEY
);

Insert IP addresses to allow access:

sql

INSERT INTO allowlist (ip) VALUES ('192.168.1.10');
INSERT INTO allowlist (ip) VALUES ('10.0.0.1');

Generate Self-Signed Certificates (for development):



    openssl genrsa -out key.pem 2048
    openssl req -new -x509 -key key.pem -out cert.pem -days 365 \
      -subj "/C=US/ST=State/L=Locality/O=Organization/CN=localhost"

Running the Server

Start the server with:



cargo run

The server will listen on https://0.0.0.0:8443.
Configuration

    Session Authentication: To disable session-based authentication, set session_required: false in AppState.
    Certificates: Place the key.pem and cert.pem files in the root directory or update load_rustls_config to use your certificate paths.

Usage
Test the IP Allowlist

If you attempt to connect from an IP not in the allowlist, the server responds with 403 Forbidden.
Session Authentication

To authenticate, include the Session-Token header with a valid token in the request.

http

GET / HTTP/1.1
Host: localhost:8443
Session-Token: your_expected_token
