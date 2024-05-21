// termimad for markdown rendering in the command line
use termimad::{crossterm::style::Color::Yellow, gray, MadSkin};
/// # Render in Markdown, Plaintext or Error
///
/// Takes a result from the Reqwest API call
pub fn markdown_plaintext_or_error(
    gpt_result: Result<String, reqwest::Error>,
    render_markdown: bool,
) {
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
        },
        Err(e) => eprintln!("Error: {}", e),
    }
}
