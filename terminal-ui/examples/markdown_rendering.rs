//! Example demonstrating the Markdown rendering functionality

use terminal_ui::markdown_renderer::render_markdown;

fn main() {
    // Example Markdown content
    let markdown_content = r#"
# Welcome to AI Terminal

This is a **bold** statement and this is *italic*.

Here's some code in Rust:

```rust
fn main() {
    println!("Hello, world!");
}
```

And here's some Python code:

```python
def hello_world():
    print("Hello, world!")
```

## Features

- Markdown rendering
- Syntax highlighting
- Beautiful terminal UI

> This is a blockquote that should be rendered nicely.
"#;

    // Render the Markdown content
    let rendered_lines = render_markdown(markdown_content);

    // Print the rendered lines
    println!("Rendered Markdown content:");
    for (i, line) in rendered_lines.iter().enumerate() {
        println!("Line {}: {:?}", i, line);
    }
}