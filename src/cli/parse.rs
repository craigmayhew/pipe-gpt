use crate::api::openai::AssistantPurpose;
use crate::api::openai::{count_tokens, create_conversation};
use crate::config::models::{MAX_TOKENS, MODEL, TEMPERATURE};
use clap::{command, value_parser, Arg, ArgAction, Command}; // clap for command line argument parsing
use log::*; // logging
use openai_api_rust::chat::ChatBody;
use std::process; // needed to exit early
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
/// - `-t [temperature]`: Set response temperature between 0.0 and 1.0. Higher values are more
///   likely to generate diverse text, but with a risk of grammar errors and generation of nonsense
/// - `-m [max_tokens]`: Advanced: Adjust token limit up to a maximum of 4096 for GPT4.
/// - `-s [top_p]`: Advanced: Adjust top_p of response between 0.0 and 1.0. It's the nucleus
///   sampling parameter.
pub fn setup_arguments() -> Command {
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
        .help("Set response temperature between 0.0 and 1.0. Higher values are more likely to generate diverse text, but with a risk of grammar errors and generation of nonsense")
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
pub fn parse_arguments(input: &str, args_setup: Command) -> (ChatBody, bool) {
    let matches = args_setup.get_matches();

    let empty_string = String::from("");

    let prepend = matches
        .get_one::<String>("prepend")
        .unwrap_or(&empty_string);
    let max_tokens = *matches.get_one::<i32>("max_tokens").unwrap_or(MAX_TOKENS);
    let temperature = *matches.get_one::<f32>("temperature").unwrap_or(TEMPERATURE);
    let top_p = *matches.get_one::<f32>("top_p").unwrap_or(&0.95);
    let render_markdown = *matches.get_one::<bool>("markdown").unwrap_or(&false);

    let assistant_purpose = if *matches.get_one::<bool>("code-review").unwrap_or(&false) {
        AssistantPurpose::CodeReviewer
    } else {
        AssistantPurpose::Default
    };

    let conversation = create_conversation(prepend, input, &assistant_purpose);

    let token_count = count_tokens(&format!(
        "{}{}{}",
        prepend,
        input,
        &assistant_purpose.to_string()
    ));

    if token_count as i32 > max_tokens {
        println!("Maximum tokens set to: {}", max_tokens);
        println!("Estimated tokens in request: {}", token_count);
        println!("Exiting early due to exceeding max input tokens. Reduce input length or increase max tokens.");
        process::exit(1);
    }

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
        messages: conversation,
    };

    info!("ChatBody struct generated");
    debug!("ChatBody struct: {:?} ", chatbody);

    (chatbody, render_markdown)
}

#[cfg(any(test, doc))]
mod tests {
    use super::*;

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
}
