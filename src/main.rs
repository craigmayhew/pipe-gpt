//! Pipe your content to gpt directly from the command line.
//! A concept that allows for a lot of possibilities in local environments and CI pipelines
//!
//! ```sh
//! tail -30 /var/httpd.log | pipe-gpt --p "Is there anything in the http log file I should fix?"
//! ```
//!
//! ```sh
//! cat main.rs | pipe-gpt -p "How would you improve this code? Include line numbers in your comments so I can tell where you mean."
//! ```
//!
//!```sh
//!cat main.rs | pipe-gpt -p "Is this code production ready? If yes reply 'Yes'. If no, then explain why not. Be concise."
//!```
//!
//! ```sh
//! cat file.json | pipe-gpt -p "Convert this JSON to YAML" > file.yaml
//! ```
//!
//! ```sh
//! cat french.txt | pipe-gpt -p "Translate this to English please."
//! ```
//!
//! ```sh
//! git diff --staged | pipe-gpt -p "Code review this code change"
//! ```

use atty::Stream; // atty to determine if data is piped in or not
use log::*; // logging
use std::io::{self, Read}; // std io

mod api;
mod cli;
mod config;

use crate::api::openai::send_to_gpt4;
use crate::cli::{
    output::markdown_plaintext_or_error,
    parse::{parse_arguments, setup_arguments},
};

/// # Entry Point for Application
///
/// - Initializes the application
/// - Initializes logging
/// - Makes calls to parse command-line arguments
/// - Checks for piped input
/// - Calls fn to send request to API
/// - Outputs result
///
/// ## Example Usage
///
/// Please see [crate] level docs for usage examples
#[tokio::main]
async fn main() {
    // enable logging
    env_logger::init();

    let mut input = String::new();
    // if data is being piped in
    // this check is necessary or we hang the whole program waiting for stdin when none arrives
    if !atty::is(Stream::Stdin) {
        debug!("Attempt: read from stdin");
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read from stdin");
        debug!("Success: read from stdin");
    }

    let (chat_body, render_markdown) = parse_arguments(&input, setup_arguments());

    markdown_plaintext_or_error(send_to_gpt4(chat_body).await, render_markdown);
    debug!("end of program");
}
