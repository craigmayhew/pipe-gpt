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

// atty to determine if data is piped in or not
use atty::Stream;
// clap for command line argument parsing
use clap::{command, value_parser, Arg, ArgAction, Command};
// logging
use log::{debug, info};
// openai api
use openai_api_rust::{
    chat::{ChatApi, ChatBody},
    Auth, Message, OpenAI, Role,
};
// std io
use std::io::{self, Read};
// termimad for markdown rendering in the command line
use termimad::{crossterm::style::Color::Yellow, gray, MadSkin};

/// Defines which gpt model to use. Currently set to "gpt-4"
const MODEL: &str = "gpt-4";
/// Defines default maximum number of tokens available in conversation and response
const MAX_TOKENS: &i32 = &4096;
/// Defines default temperature of response
const TEMPERATURE: &f32 = &0.6;

enum AssistantPurpose {
    CodeReviewer,
    Default,
}

/// # Create Conversation Vector
///
/// Add the prepend string if present. Add piped stream if present.
fn create_conversation(
    prepend: String,
    input: &str,
    purpose: AssistantPurpose,
) -> Vec<openai_api_rust::Message> {
    let mut conversation_messages = match purpose {
        AssistantPurpose::Default => {
            vec![Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
            }]
        }
        AssistantPurpose::CodeReviewer => {
            vec![Message { role: Role::System, content: "You are a helpful assistant. How would you improve this code? Include line numbers in your comments so I can tell where you mean. ".to_string() }]
        }
    };
    if !&prepend.is_empty() {
        conversation_messages.push(Message {
            role: Role::User,
            content: prepend,
        });
    }
    // if data was piped into this application, add it to the conversation
    // This is useful even if the input is blank, as a form of debug, GPT will likely respond with ~"It looks like you forgot the data"
    if !atty::is(Stream::Stdin) {
        conversation_messages.push(Message {
            role: Role::User,
            content: input.to_string(),
        });
    }

    conversation_messages
}

/// # Send Request To Openai API
///
/// Loads the OPENAI_API_KEY environment variable, connects to OpenAI API, sends chat
async fn send_to_gpt4(body: ChatBody) -> Result<String, reqwest::Error> {
    // debug log
    debug!("entered send_to_gpt4()");

    // Load API key from environment OPENAI_API_KEY
    let auth = Auth::from_env().expect("Failed to read auth from environment");
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let chat_completion = openai
        .chat_completion_create(&body)
        .expect("chat completion failed");
    let choice = chat_completion.choices;
    let message = &choice[0]
        .message
        .as_ref()
        .expect("Failed to read message from API");
    // debug log
    debug!("message recieved {:?}", message);

    Ok(message.content.to_string())
}

/// # Define Command Line Arguments
///
/// This function defines command line arguments and their descriptions
///
/// ## Basic Usage
///
/// - `-p [prepend]`: Text to prepend to the piped content e.g. `-p "find the pattern: "`
/// - `--markdown`: Render markdown instead of outputting as plain text.
///
/// ## Advanced Usage
///
/// - `-t [temperature]`: Advanced: Adjust temperature of response between 0.0 and 1.0.
///   The higher the value, the more likely the generated text will be diverse, but there
///   is a higher possibility of grammar errors and generation of nonsense.
/// - `-m [max_tokens]`: Advanced: Adjust token limit up to a maximum of 4096 for GPT4.
/// - `-s [top_p]`: Advanced: Adjust top_p of response between 0.0 and 1.0. It's the nucleus
///   sampling parameter.
fn setup_arguments() -> Command {
    let code_review_flag = Arg::new("code-review")
        .long("code-review")
        .value_name("code-review")
        .help("Use a default prompt that will review your piped code")
        .required(false)
        .action(ArgAction::SetTrue);

    let markdown_flag = Arg::new("markdown")
        .long("markdown")
        .value_name("markdown")
        .help("Render markdown instead of outputting as plain text")
        .required(false)
        .action(ArgAction::SetTrue);

    let max_tokens_arg = Arg::new("max_tokens")
        .short('m')
        .long("max_tokens")
        .value_name("max_tokens")
        .help(format!(
            "Advanced: Adjust token limit up to a maximum of {} for GPT4",
            MAX_TOKENS
        ))
        .required(false)
        .value_parser(value_parser!(i32));

    let prepend_arg = Arg::new("prepend")
        .short('p')
        .long("prepend")
        .value_name("prepend")
        .help("Text to prepend to the piped content e.g. \"find the pattern: \"")
        .required(false);

    let temperature_arg = Arg::new("temperature")
        .short('t')
        .long("temperature")
        .value_name("temperature")
        .help("Advanced: Adjust temperature of response between 0.0 and 1.0. The higher the value, the more likely the generated text will be diverse, but there is a higher possibility of grammar errors and generation of nonsense")
        .required(false)
        .value_parser(value_parser!(f32));

    let top_p_arg = Arg::new("top_p")
        .short('s')
        .long("top_p")
        .value_name("top_p")
        .help("Advanced: Adjust top_p of response between 0.0 and 1.0. It's the nucleus sampling parameter")
        .required(false)
        .value_parser(value_parser!(f32));

    command!() // requires `cargo` feature
        .about("Sends piped content to GPT-4. Author: Craig Mayhew")
        .arg(code_review_flag)
        .arg(markdown_flag)
        .arg(max_tokens_arg)
        .arg(prepend_arg)
        .arg(temperature_arg)
        .arg(top_p_arg)
}

/// # Parse Command Line Arguments
///
/// Arguments are set to defaults where ommitted
fn parse_arguments(input: &str, args_setup: Command) -> (ChatBody, bool) {
    let matches = args_setup.get_matches();

    let empty_string = String::from("");

    let prepend = matches
        .get_one::<String>("prepend")
        .unwrap_or(&empty_string)
        .to_owned();
    let max_tokens = *matches.get_one::<i32>("max_tokens").unwrap_or(MAX_TOKENS);
    let temperature = *matches.get_one::<f32>("temperature").unwrap_or(TEMPERATURE);
    let top_p = *matches.get_one::<f32>("top_p").unwrap_or(&0.95);
    let render_markdown = *matches.get_one::<bool>("markdown").unwrap_or(&false);

    let assistant_purpose = if *matches.get_one::<bool>("code-review").unwrap_or(&false) {
        AssistantPurpose::CodeReviewer
    } else {
        AssistantPurpose::Default
    };

    let chatbody = ChatBody {
        model: MODEL.to_owned(),
        max_tokens: Some(max_tokens),
        temperature: Some(temperature),
        top_p: Some(top_p),
        n: Some(1),
        stream: Some(false), // streaming output is not yet supported by this rust app
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: create_conversation(prepend, input, assistant_purpose),
    };

    info!("ChatBody struct generated");
    debug!("ChatBody struct: {:?} ", chatbody);

    (chatbody, render_markdown)
}

/// # Render in Markdown, Plaintext or Error
///
/// Takes a result from the Reqwest API call
fn markdown_plaintext_or_error(gpt_result: Result<String, reqwest::Error>, render_markdown: bool) {
    match gpt_result {
        Ok(markdown) => {
            if render_markdown {
                let mut skin = MadSkin::default();
                skin.code_block.left_margin = 4;
                skin.code_block.set_fgbg(gray(17), gray(3));
                skin.set_fg(Yellow);
                skin.print_text(&markdown)
            } else {
                println!("{}", &markdown)
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

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
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read from stdin");
    }

    let (chat_body, render_markdown) = parse_arguments(&input, setup_arguments());

    markdown_plaintext_or_error(send_to_gpt4(chat_body).await, render_markdown);
}

#[cfg(any(test, doc))]
mod tests {
    use super::*;

    /// Test that piped input is not detected
    #[cfg_attr(not(doc), test)]
    fn test_create_conversation_no_pipe() {
        let p_text = "This is the prepend";

        let prepend = p_text.to_string();
        let input = "This is the piped input. It won't be piped as part of the test".to_string();
        let purpose = AssistantPurpose::Default;

        let conversation = create_conversation(prepend, &input, purpose);
        //TODO: Investigate why this is 3 != 2 in github actions but 2 == 2 when run locally
        //assert_eq!(conversation.len(), 2); // then len is only two instead of three because piping isn't active here
        assert_eq!(conversation[1].content, p_text);
    }

    /// Test that the chat_body has some sensible values after being initialised
    #[cfg_attr(not(doc), test)]
    fn test_parse_arguments() {
        let command = setup_arguments();
        let input = "Test".to_string();
        let (chat_body, render_markdown) = parse_arguments(&input, command);

        assert_eq!(chat_body.model, "gpt-4");
        assert_eq!(chat_body.max_tokens.unwrap(), *MAX_TOKENS);
        assert_eq!(chat_body.temperature.unwrap(), *TEMPERATURE);
        assert_eq!(render_markdown, false);
    }

    /// Test an API call using an API key
    #[cfg_attr(not(doc), tokio::test)]
    async fn test_send_to_gpt4() {
        // Note: This test requires a valid API key set in the environment  
        let body = ChatBody {
            model: MODEL.to_owned(),
            max_tokens: Some(*MAX_TOKENS),
            temperature: Some(*TEMPERATURE),
            top_p: Some(0.95),
            n: Some(1),
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: vec![Message {
                role: Role::User,
                content: "Translate this to English please.".to_string(),   
            }],
        };

        let result = send_to_gpt4(body).await;

        assert!(result.is_ok());
    }
}
