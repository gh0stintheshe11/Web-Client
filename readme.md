# Web Client

## Overview

This project implements a simplified version of the UNIX curl utility as a Rust command-line application. The purpose is to create a tool that can perform HTTP requests (GET and POST) and handle responses, including error cases, in a user-friendly manner. This project demonstrates the ability to:

* Parse and validate user-provided URLs
* Perform GET and POST HTTP requests
* Handle and report errors gracefully
* Pretty-print JSON responses

## Implementation and Features

### Architecture

The project is divided into modular components to ensure clarity, maintainability, and testability:

1. Command-Line Argument Parsing:
   * Built using the structopt crate for clean and ergonomic CLI argument parsing
   * Defined a CurlArgs struct to encapsulate command-line options:
     * URL (required)
     * Method (-X, defaults to GET)
     * Form data (-d)
     * JSON data (--json, automatically sets method to POST)

2. URL Validation:
   * Used the url crate to parse and validate URLs
   * Pre-parse validation for:
     * Invalid IPv6 addresses (e.g., [...1])
     * Invalid IPv4 addresses (octets > 255)
     * Invalid port numbers (> 65535)
   * Post-parse validation for:
     * Protocol validity (only http/https)

3. HTTP Request Handling:
   * Used the reqwest crate for making HTTP requests
   * Supported methods:
     * GET requests with automatic error handling
     * POST requests with form data (-d)
     * POST requests with JSON payloads (--json)
   * Content-Type header handling:
     * application/x-www-form-urlencoded for form data
     * application/json for JSON payloads

4. Response Processing:
   * Parsed JSON responses with serde_json for:
     * Pretty-printing output
     * Automatic key sorting in alphabetical order
   * Direct output for non-JSON responses

5. Error Handling:
   * URL-related errors:
     * Invalid base protocol
     * Invalid IPv4/IPv6 addresses
     * Invalid port numbers
   * Network errors:
     * Connection failures
     * Unresolvable hosts
   * HTTP errors:
     * Non-success status codes (e.g., 404)
   * Data validation:
     * Invalid JSON format (panic with detailed message)

## Unit Tests

The test suite covers all required functionality and error cases:

### URL Validation Tests
* [x] Invalid base protocol cases:
  * Missing protocol (www.eecg.toronto.edu)
  * Unsupported protocol (data://)
  * Malformed protocol (http//)
* [x] Invalid IP address cases:
  * Malformed IPv6 ([...1])
  * Invalid IPv4 octets (255.255.255.256)
* [x] Invalid port number case (65536)

### Request Error Tests
* [x] Network error handling:
  * Unresolvable host (example.rs)
  * Connection failures
* [x] HTTP status code handling:
  * 404 Not Found responses

### POST Request Tests
* [x] Form data (-d option):
  * Key-value pairs
  * JSON response formatting
* [x] JSON payload (--json option):
  * Valid JSON handling
  * Invalid JSON validation
  * Automatic method setting to POST

## Usage Instructions

### Build the Project

```bash
cargo build
```

### Test the Project

```bash
cargo test
```

### Run the Utility

**GET Request:**
```bash
target/debug/curl "http://example.com"
```

**POST Request with Form Data:**
```bash
target/debug/curl "http://example.com" -X POST -d "key=value"
```

**POST Request with JSON:**
```bash
target/debug/curl "http://example.com" -X POST --json '{"key":"value"}'
```

## Conclusion

This project demonstrates a clean and robust implementation of a simplified curl utility. It adheres to the requirements, ensures reliability through comprehensive testing, and leverages Rust's ecosystem for ergonomic and safe programming.