use clap::{arg, command, Command, value_parser};
use openai_api_rust::{
    Auth,
    chat::{ChatApi,ChatBody},
    Message,
    OpenAI,
    Role,
};
use reqwest;
use std::io::{self, Read};
use termimad::{
    gray,
    crossterm::style::Color::Yellow,
    MadSkin,
};

/// Configuration
const MODEL: &str = "gpt-4";

/// initialise a ChatBody struct.
/// Todo: This can probably be part of a lrger refactor so we aren't passing so many tuples back and forther between functions. i.e. we have ChatBody, just use that
fn initialise_chat_body (max_tokens: i32, temperature: f32, top_p: f32, conversation_messages: Vec<Message>) -> ChatBody {
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
        messages: conversation_messages,
    };
    chatbody
}

/// send request to openai api
async fn send_to_gpt4(input: &str, arguments: (String, i32, f32, f32, bool)) -> Result<String, reqwest::Error> {
    let (prepend, max_tokens, temperature, top_p, _render_markdown) = arguments;
    let mut conversation_messages = vec![
        Message { role: Role::System, content: "You are a helpful assistant.".to_string() },
    ];
    if !&prepend.is_empty() {
        conversation_messages.push(Message { role: Role::User, content: prepend });
    }
    conversation_messages.push(Message { role: Role::User, content: "the following is piped input from the command line".to_owned() + input });
    // Load API key from environment OPENAI_API_KEY
    let auth = Auth::from_env().expect("Failed to read auth from environment");
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body = initialise_chat_body(max_tokens, temperature, top_p, conversation_messages);
    let chat_completion = openai.chat_completion_create(&body).expect("chat completion failed");
    let choice = chat_completion.choices;
    let message = &choice[0].message.as_ref().expect("Failed tgo read message from API");

    Ok(message.content.to_string())
}

/// define command line argument and their descriptions
fn args_setup() -> Command {
    command!() // requires `cargo` feature
        .about("Sends piped content to GPT-4. Author: Craig Mayhew")
        .arg(
            arg!(
                -p [prepend] "Text to prepend to the piped content e.g. \"find the pattern: \""
            )
            .required(false)
        )
        .arg(
            arg!(
                -t [temperature] "Advanced: Adjust temperature of response between 0.0 and 1.0. The higher the value, the more likely the generated text will be diverse, but there is a higher possibility of grammar errors and generation of nonsense."
            )
            .required(false)
        )
        .arg(
            arg!(
                -m [max_tokens] "Advanced: Adjust token limit up to a maximum of 4096 for GPT4."
            )
            .required(false)
        )
        .arg(
            arg!(
                -s [top_p] "Advanced: Adjust top_p of response between 0.0 and 1.0. It's the nucleus sampling parameter."
            )
            .required(false)
        )
        .arg(
            arg!(
                --markdown "Render markdown instead of outputting as plain text."
            )
            .required(false)
            .value_parser(value_parser!(bool))
        )
}

/// parse command line arguments, setting to defaults where ommitted
fn args_read (args_setup: Command) -> (std::string::String, i32, f32, f32, bool) {
    let matches = args_setup.get_matches();
    
    let empty_string = String::from("");

    let prepend = matches.get_one::<String>("prepend").unwrap_or(&empty_string);
    let max_tokens = matches.get_one::<i32>("max_tokens").unwrap_or(&4096);
    let temperature = matches.get_one::<f32>("temperature").unwrap_or(&0.6);
    let top_p = matches.get_one::<f32>("top_p").unwrap_or(&0.95);
    let render_markdown = matches.get_one::<bool>("markdown").unwrap_or(&false);
    
    (prepend.to_owned(),max_tokens.to_owned(),temperature.to_owned(),top_p.to_owned(),render_markdown.to_owned())
}

#[tokio::main]
async fn main() {
    let parsed_arguments = args_read(args_setup());
    let render_markdown = parsed_arguments.4;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("Failed to read from stdin");

    match send_to_gpt4(&input, parsed_arguments).await {
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
        },
        Err(e) => eprintln!("Error: {}", e),
    }
}
