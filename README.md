# pipe-gpt
Pipe your content to gpt directly from the command line. A concept that allows for a lot of possibilities.

Installation via cargo `cargo install pipe-gpt`, build it locally via `cargo build --release`, or download from the releases page.

Please note you will need an api key e.g. [OpenAI API Key](https://platform.openai.com/account/api-keys).

### Set the API key env var
 - The app reads AI_API_KEY first, then falls back to OPENAI_API_KEY
 - in linux/macOS: `export AI_API_KEY=sk-12345abc` or `export OPENAI_API_KEY=sk-12345abc`
 - in windows powershell `$env:AI_API_KEY = 'sk-12345abc'` or `$env:OPENAI_API_KEY = 'sk-12345abc'`

## Configuration
- Config file path: ~/.config/pipe-gpt/config.yaml (Linux/macOS) or %APPDATA%/pipe-gpt/config.yaml (Windows)
- Fields and defaults:
  - api_url: https://api.openai.com/v1/
  - model: gpt-4o
  - max_tokens: 8192
  - temperature: 0.6
- Example:
```
api_url: "https://api.openai.com/v1/"
model: "gpt-4o"
max_tokens: 2048
temperature: 0.6
```

## Use cases

### Local command-line usage
- `tail -30 /var/httpd.log | pipe-gpt --p "Is there anything in the http log file I should fix?"`
- `cat main.rs | pipe-gpt -p "How would you improve this code? Include line numbers in your comments so I can tell where you mean."`
- `cat main.rs | pipe-gpt -p "Is this code production ready? If yes reply 'Yes'. If no, then explain why not. Be concise."`
- `cat file.json | pipe-gpt -p "Convert this JSON to YAML" > file.yaml`
- `cat french.txt | pipe-gpt -p "Translate this to English please."` 
- `git diff --staged | pipe-gpt -p "Code review this code change"`
 - `cat src/main.rs | pipe-gpt -p "improve the code and only output the replacement code as I will pipe the output directly back into a file, no explanations, just pure code please" > src/main.new.rs`

### pipe-gpt for local dev

```
user@computer:/GitHub/pipe-gpt$ cat src/main.rs | pipe-gpt -p "how would you improve this code? include line numbers in your comments so I can tell where you mean"

Here are some suggestions to improve your code:

1. Line 2, 3: You can combine these two lines into one as follows:
`use openai_api_rust::{*, chat::*};`

2. Line 6-16: You can simplify the creation of `conversation_messages` by using a `Vec::new()` and then pushing messages as needed. This way, you avoid creating an unnecessary `Message` when `prepend` is empty.

3. Line 18-19: Instead of using `unwrap()`, it's better to handle the error properly. You can use `?` to propagate the error up.

4. Line 20-33: You can create a function to initialize `ChatBody` to make the code cleaner.

5. Line 35-38: Instead of using `unwrap()`, it's better to handle the error properly. You can use `match` to handle the possible `None` case.
```

### pipe-gpt for CI

```yaml
jobs:
  build:

    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
    - name: GPT Code Review
      env:
        OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
      run: pwd && find . -path './target' -prune -o -name '*.rs' -exec echo {} \; -exec cat {} \; | ./target/debug/pipe-gpt -p "how would you improve this code? include line numbers in your comments so I can tell where you mean"
```

Which gives this output in Github Actions CI:
![Pipe GPT used in CI to give code review recomendations](./imgs/github-workflow-gpt-code-review-markdown.png)

## Roadmap
 - [ ] gpt-5 update
 - [ ] graceful API throttling
 - [ ] secrets and config files
 - [ ] loading custom prompts as short arguments
 - [ ] namespaced roles/prompts
 - [ ] reduce dependencies

## Contributing
If you wish to contribute to the codebase, please see [CONTRIBUTE.md](./CONTRIBUTE.md)

## Useful links
 - https://platform.openai.com/account/usage
 - https://platform.openai.com/docs/api-reference/chat/create
 - https://platform.openai.com/docs/models/overview
