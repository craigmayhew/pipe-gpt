use clap::{arg, command};
use openai_api_rust::{*,chat::*};
use reqwest;
use std::io::{self, Read};

fn initialise_chat_body (conversation_messages: Vec<Message>) -> ChatBody {
    ChatBody {
        model: "gpt-4".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.6_f32),
        top_p: Some(0.95_f32),
        n: Some(1),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: conversation_messages,
    }
}

async fn send_to_gpt4(input: &str, prepend: &str) -> Result<String, reqwest::Error> {
    let mut conversation_messages = vec![
        Message { role: Role::System, content: "You are a helpful assistant.".to_string() },
    ];
    if !prepend.is_empty() {
        conversation_messages.push(Message { role: Role::User, content: prepend.to_string() });
    }
    conversation_messages.push(Message { role: Role::User, content: "the following is piped input from the command line".to_owned() + input });
    // Load API key from environment OPENAI_API_KEY.
    // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body = initialise_chat_body(conversation_messages);
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();

    Ok(message.content.to_string())
}

#[tokio::main]
async fn main() {
    let matches = command!() // requires `cargo` feature
        .version("0.1")
        .author("Craig Mayhew")
        .about("Sends piped content to GPT-4")
        .arg(
            arg!(
                -p [prepend] "Text to prepend to the piped content e.g. \"find the pattern\""
            )
            .required(false)
        )
        .get_matches();

    let empty_string = String::from("");
    let prepend = matches.get_one::<String>("prepend").unwrap_or(&empty_string);

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("Failed to read from stdin");

    match send_to_gpt4(&input, prepend).await {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }
}
