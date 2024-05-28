Pull Requests welcome. Please run fmt against your code before commits/prs.

### Design Patterns
**Enum and Match Pattern** - The AssistantPurpose enum and the match statement used in its to_string implementation is an example of the Enum and Match pattern. This pattern is often used in Rust to handle different variants of a type.

**Factory Pattern** - The create_conversation function can be seen as an implementation of the Factory pattern. It creates and returns a new conversation (a Vec<Message>) based on the given parameters.

**Singleton Pattern** - The Auth::from_env function can be seen as an implementation of the Singleton pattern. It ensures that only one instance of the authentication token is loaded from the environment.

**Adapter Pattern** - The send_to_gpt4 function can be seen as an implementation of the Adapter pattern. It adapts the interface of the OpenAI API to a simpler interface (a single function call) for sending chat messages.

**Facade Pattern** - The send_to_gpt4 function can also be seen as an implementation of the Facade pattern. It provides a simplified interface to a complex subsystem (the OpenAI API).

**Builder Pattern** - The ChatBody struct and its usage in the test_send_to_gpt4 function can be seen as an implementation of the Builder pattern. The ChatBody struct has optional fields that can be set in a chained manner.

