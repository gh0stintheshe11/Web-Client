use structopt::StructOpt;

// Command-line argument structure using structopt
#[derive(StructOpt)]
#[structopt(name = "curl", about = "A simple HTTP client.")]
struct CurlArgs {
    /// URL to request
    url: String,

    /// HTTP method (GET or POST)
    #[structopt(short = "X", default_value = "GET")]
    method: String,

    /// Data to send in a POST request (form data format: key1=value1&key2=value2)
    #[structopt(short = "d")]
    data: Option<String>,

    /// JSON data for POST request (automatically sets method to POST)
    #[structopt(long = "json")]
    json_data: Option<String>,
}

// Helper methods for CurlArgs
impl CurlArgs {
    /// Determines the HTTP method to use based on arguments
    /// Returns POST if json_data is present, otherwise returns the specified method
    fn get_method(&self) -> String {
        if self.json_data.is_some() {
            "POST".to_string()
        } else {
            self.method.clone()
        }
    }
}

use url::Url;
// Helper methods for URL parsing and validation
fn validate_url(url: &str) -> Result<Url, String> {
    // Pre-validation checks before URL parsing
    if let Some(host) = url.split("://").nth(1) {
        let host = host.split('/').next().unwrap_or(host);
        
        // Check for IPv6 address validity
        if host.starts_with('[') && host.contains(']') {
            // Extract the IPv6 address part between brackets
            if let Some(ipv6_str) = host.split('[')
                .nth(1)
                .and_then(|s| s.split(']').next()) 
            {
                // Check for compressed zeros format
                if ipv6_str.contains("::") {
                    let double_colon_count = ipv6_str.matches("::").count();
                    if double_colon_count > 1 {
                        return Err("The URL contains an invalid IPv6 address.".to_string());
                    }
                }

                // Split into segments and validate each
                let segments: Vec<&str> = ipv6_str.split(':').collect();
                
                // IPv6 should have 8 segments (or fewer with ::)
                if segments.len() > 8 {
                    return Err("The URL contains an invalid IPv6 address.".to_string());
                }

                // Validate each segment
                for segment in segments {
                    if segment.is_empty() && !ipv6_str.contains("::") {
                        return Err("The URL contains an invalid IPv6 address.".to_string());
                    }
                    if !segment.is_empty() {
                        // Each segment should be valid hexadecimal and not longer than 4 chars
                        if segment.len() > 4 || !segment.chars().all(|c| c.is_ascii_hexdigit()) {
                            return Err("The URL contains an invalid IPv6 address.".to_string());
                        }
                    }
                }
            }
        }

        // Validate IPv4 address format and values
        let ip_part = host.split(':').next().unwrap_or(host);
        if ip_part.split('.').count() == 4 {
            let octets: Vec<&str> = ip_part.split('.').collect();
            if octets.iter().any(|&octet| {
                if let Ok(num) = octet.parse::<u32>() {
                    num > 255
                } else {
                    false
                }
            }) {
                return Err("The URL contains an invalid IPv4 address.".to_string());
            }
        }

        // Validate port number range
        if let Some(port_str) = host.split(':').nth(1) {
            if let Ok(port) = port_str.split('/').next().unwrap_or(port_str).parse::<u32>() {
                if port > 65535 {
                    return Err("The URL contains an invalid port number.".to_string());
                }
            }
        }
    }

    // Parse and validate URL structure
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(_) => return Err("The URL does not have a valid base protocol.".to_string()),
    };

    // Ensure protocol is http or https
    match parsed_url.scheme() {
        "http" | "https" => (),
        _ => return Err("The URL does not have a valid base protocol.".to_string()),
    }

    Ok(parsed_url)
}

use reqwest::blocking::Client;

// Makes HTTP requests based on command-line arguments
// Handles both GET and POST methods
// For POST requests, supports both form data and JSON data
// Returns response body as string or error message
fn make_request(args: &CurlArgs) -> Result<String, String> {
    let client = Client::new();

    // Validate JSON data if present
    if let Some(json_data) = &args.json_data {
        println!("JSON: {}", json_data);
        if let Err(e) = serde_json::from_str::<serde_json::Value>(json_data) {
            panic!("Invalid JSON: {}", e);
        }
    }

    match args.get_method().as_str() {
        // Handle GET requests
        "GET" => {
            let response = client.get(&args.url).send();
            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        return Err(format!(
                            "Request failed with status code: {}",
                            resp.status().as_u16()
                        ));
                    }
                    resp.text().map_err(|e| e.to_string())
                }
                Err(e) => {
                    if e.is_connect() {
                        Err("Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.".to_string())
                    } else {
                        Err(e.to_string())
                    }
                }
            }
        }
        // Handle POST requests
        "POST" => {
            // Handle JSON data POST requests
            if let Some(json_data) = &args.json_data {
                let response = client
                    .post(&args.url)
                    .header("Content-Type", "application/json")
                    .body(json_data.clone())
                    .send();

                match response {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            return Err(format!(
                                "Request failed with status code: {}",
                                resp.status().as_u16()
                            ));
                        }
                        resp.text().map_err(|e| e.to_string())
                    }
                    Err(e) => {
                        if e.is_connect() {
                            Err("Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.".to_string())
                        } else {
                            Err(e.to_string())
                        }
                    }
                }
            } else if let Some(data) = &args.data {
                println!("Data: {}", data);

                // Attempt to parse as JSON first
                if data.starts_with('{') {
                    // Parse JSON data
                    let json_value: serde_json::Value = serde_json::from_str(data)
                        .map_err(|e| format!("Invalid JSON data: {}", e))?;

                    let response = client
                        .post(&args.url)
                        .header("Content-Type", "application/json")
                        .json(&json_value)
                        .send();

                    match response {
                        Ok(resp) => {
                            if !resp.status().is_success() {
                                return Err(format!(
                                    "Request failed with status code: {}",
                                    resp.status().as_u16()
                                ));
                            }
                            resp.text().map_err(|e| e.to_string())
                        }
                        Err(e) => {
                            if e.is_connect() {
                                Err("Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.".to_string())
                            } else {
                                Err(e.to_string())
                            }
                        }
                    }
                } else {
                    // Handle form data POST requests
                    let response = client
                        .post(&args.url)
                        .header("Content-Type", "application/x-www-form-urlencoded")
                        .body(data.clone())
                        .send();

                    match response {
                        Ok(resp) => {
                            if !resp.status().is_success() {
                                return Err(format!(
                                    "Request failed with status code: {}",
                                    resp.status().as_u16()
                                ));
                            }
                            resp.text().map_err(|e| e.to_string())
                        }
                        Err(e) => {
                            if e.is_connect() {
                                Err("Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.".to_string())
                            } else {
                                Err(e.to_string())
                            }
                        }
                    }
                }
            } else {
                Err("No data provided for POST request.".to_string())
            }
        }
        _ => Err("Unsupported HTTP method.".to_string()),
    }
}

// Formats JSON responses with pretty printing and sorted keys
// Returns original string if input is not valid JSON
fn format_json(response_body: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(response_body) {
        Ok(json) => {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| response_body.to_string())
        }
        Err(_) => response_body.to_string(),
    }
}

// Main function that ties everything together
// 1. Parses command-line arguments
// 2. Validates URL
// 3. Makes HTTP request
// 4. Formats and prints response
fn main() {
    let args = CurlArgs::from_args();
    let method = args.get_method();

    println!("Requesting URL: {}", args.url);
    println!("Method: {}", method);

    match validate_url(&args.url) {
        Ok(_) => match make_request(&args) {
            Ok(body) => {
                // Check if response is JSON
                if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
                    println!("Response body (JSON with sorted keys):");
                    println!("{}", format_json(&body));
                } else {
                    println!("Response body:");
                    println!("{}", body);
                }
            }
            Err(e) => println!("Error: {}", e),
        },
        Err(e) => println!("Error: {}", e),
    }
}

// UNIT TESTS

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::str;

    // Helper function to run command and get output
    fn run_command(args: &[&str]) -> String {
        let mut command_args = vec!["run", "--"];
        command_args.extend(args);

        let output = Command::new("cargo")
            .args(&command_args)
            .output()
            .expect("Failed to execute command");

        str::from_utf8(&output.stdout).unwrap().trim().to_string()
    }

    // Helper function to run command and get both stdout and stderr
    fn run_command_with_stderr(args: &[&str]) -> (String, String) {
        let mut command_args = vec!["run", "--"];
        command_args.extend(args);

        let output = Command::new("cargo")
            .args(&command_args)
            .output()
            .expect("Failed to execute command");

        (
            str::from_utf8(&output.stdout).unwrap().trim().to_string(),
            str::from_utf8(&output.stderr).unwrap().trim().to_string(),
        )
    }

    #[test]
    fn test_basic_get_request() {
        let output = run_command(&[
            "https://www.eecg.toronto.edu/~bli/ece1724/assignments/files/lab3.html"
        ]);
        
        assert_eq!(output, 
            "Requesting URL: https://www.eecg.toronto.edu/~bli/ece1724/assignments/files/lab3.html
Method: GET
Response body:
<html>
<body>
<h1>
Hello, World!
</h1>
</body>
</html>");
    }

    #[test]
    fn test_url_errors() {
        // Invalid protocol cases
        let protocol_test_cases = vec![
            "www.eecg.toronto.edu",
            "data://www.eecg.toronto.edu",
            "http//www.eecg.toronto.edu",
        ];

        for url in protocol_test_cases {
            let output = run_command(&[url]);
            assert_eq!(output, format!(
                "Requesting URL: {}\nMethod: GET\nError: The URL does not have a valid base protocol.",
                url
            ));
        }

        // Invalid IP addresses
        let ip_test_cases = vec![
            ("https://[...1]", "Error: The URL contains an invalid IPv6 address."),
            ("https://255.255.255.256", "Error: The URL contains an invalid IPv4 address."),
        ];

        for (url, error_msg) in ip_test_cases {
            let output = run_command(&[url]);
            assert_eq!(output, format!(
                "Requesting URL: {}\nMethod: GET\n{}",
                url, error_msg
            ));
        }

        // Invalid port
        let output = run_command(&["http://127.0.0.1:65536"]);
        assert_eq!(output, 
            "Requesting URL: http://127.0.0.1:65536\nMethod: GET\nError: The URL contains an invalid port number.");
    }

    #[test]
    fn test_request_errors() {
        // Unreachable host
        let output = run_command(&["https://example.rs"]);
        assert_eq!(output, 
            "Requesting URL: https://example.rs\nMethod: GET\nError: Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.");

        // HTTP status error
        let output = run_command(&[
            "https://www.eecg.toronto.edu/~bli/ece1724/assignments/files/lab4.html"
        ]);
        assert_eq!(output, 
            "Requesting URL: https://www.eecg.toronto.edu/~bli/ece1724/assignments/files/lab4.html\nMethod: GET\nError: Request failed with status code: 404");
    }

    #[test]
    fn test_post_requests() {
        // Form data POST
        let output = run_command(&[
            "https://jsonplaceholder.typicode.com/posts",
            "-d", "userId=1&title=Hello World",
            "-X", "POST",
        ]);
        assert_eq!(output, 
            "Requesting URL: https://jsonplaceholder.typicode.com/posts\nMethod: POST\nData: userId=1&title=Hello World\nResponse body (JSON with sorted keys):\n{\n  \"id\": 101,\n  \"title\": \"Hello World\",\n  \"userId\": \"1\"\n}");

        // JSON POST
        let output = run_command(&[
            "--json", "{\"title\": \"World\", \"userId\": 5}",
            "https://dummyjson.com/posts/add",
        ]);
        assert_eq!(output, 
            "Requesting URL: https://dummyjson.com/posts/add\nMethod: POST\nJSON: {\"title\": \"World\", \"userId\": 5}\nResponse body (JSON with sorted keys):\n{\n  \"id\": 252,\n  \"title\": \"World\",\n  \"userId\": 5\n}");

        // Invalid JSON
        let (stdout, stderr) = run_command_with_stderr(&[
            "--json", "{\"title\": \"World\"; \"userId\": 5}",
            "https://dummyjson.com/posts/add",
        ]);
        
        // Only check the first part of the output since the error might vary
        let expected_start = "Requesting URL: https://dummyjson.com/posts/add\nMethod: POST\nJSON: {\"title\": \"World\"; \"userId\": 5}";
        assert!(stdout.starts_with(expected_start));
        assert!(stderr.contains("thread 'main' panicked"));
        assert!(stderr.contains("Invalid JSON:"));
    }
}
