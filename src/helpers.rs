use std::{io, fs};
use std::io::{BufRead, BufReader};
// use std::net::{IpAddr, SocketAddr};


/// Reads the contents of a file into a vector of strings, where each string represents a line in the file.
///
/// # Arguments
///
/// * `filename` - The path to the file to read.
///
/// # Returns
///
/// A vector of strings, where each string is a line from the file. If an error occurs while reading the file, an `io::Error` is returned.
pub fn read_file_lines_to_vec(filename: &str) -> io::Result<Vec<String>> {
    // Open the file in read-only mode.
    let file_in = fs::File::open(filename)?;

    // Create a buffered reader to improve performance.
    let file_reader = BufReader::new(file_in);

    // Read the lines from the file, filtering out any errors that occur.
    Ok(file_reader.lines().filter_map(io::Result::ok).collect())
}

/// Checks if a given address is blocked by reading from a blacklist file.
///
/// This function reads the contents of a file named "blacklist.txt" in the current directory,
/// and checks if the given `address_to_check` is present in the file.
///
/// # Arguments
///
/// * `address_to_check` - The address to check against the blacklist.
///
/// # Returns
///
/// `true` if the address is blocked, `false` otherwise.
pub fn check_address_block(address_to_check: &str) -> bool {
    // Read the contents of the blacklist file into a vector of strings
    let addresses_blocked = read_file_lines_to_vec(&"./blacklist.txt".to_string());

    // Handle any errors that may occur while reading the file
    let addresses_blocked_iter: Vec<String> = addresses_blocked.unwrap_or_else(|_| vec!["Error".to_string()]);

    // Check if the address is present in the blacklist
    let address_in = addresses_blocked_iter.contains(&address_to_check.to_string());

    // Return the result of the check
    address_in
}
