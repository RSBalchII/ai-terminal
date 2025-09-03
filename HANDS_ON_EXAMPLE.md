# Hands-On Example - Adding a New Feature

Let's walk through adding a simple new feature to understand how to modify Rust code.

## 🎯 Goal: Add a "Clear Screen" command

We'll add a new keyboard shortcut (Ctrl+L) that clears all messages from the chat.

## Step 1: Understanding the Current Code

First, let's look at where keyboard input is handled:

```rust
// In terminal-ui/src/lib.rs, around line 218
async fn handle_chat_key(&mut self, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) => {
            self.input.push(c);
        }
        KeyCode::Backspace => {
            self.input.pop();
        }
        KeyCode::Enter => {
            // Send message to AI
        }
        KeyCode::F(1) => {
            self.mode = AppMode::Help;
        }
        // ... more keys
    }
}
```

## Step 2: Add the New Key Handler

Let's add our new Ctrl+L handler. In Rust, we need to check for both the key and modifier:

```rust
// Add this inside the match statement in handle_chat_key
KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    // Clear all messages
    self.messages.clear();
    
    // Add a confirmation message
    self.add_message(Message {
        role: "system".to_string(),
        content: "🧹 Chat cleared!".to_string(),
        timestamp: Instant::now(),
    });
}
```

## Step 3: Import the Required Module

At the top of the file, we need to import `KeyModifiers`:

```rust
// Change this line (around line 3):
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    // ... other imports
};

// To this:
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    // ... other imports
};
```

## Step 4: Update the Help Text

Let's also update the help screen to mention our new shortcut:

```rust
// In the render_help_ui function (around line 675), add a new line:
Line::from("  Ctrl+L  - Clear chat history"),
```

## Step 5: The Complete Change

Here's exactly what you'd modify:

### File: `terminal-ui/src/lib.rs`

**Change 1** - Add KeyModifiers import:
```rust
// Line ~3, change:
event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},

// To:
event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
```

**Change 2** - Add the key handler (around line 272):
```rust
KeyCode::F(10) => {
    self.should_quit = true;
}
// ADD THIS NEW CASE:
KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    // Clear all messages
    self.messages.clear();
    
    // Add a confirmation message
    self.add_message(Message {
        role: "system".to_string(),
        content: "🧹 Chat cleared!".to_string(),
        timestamp: Instant::now(),
    });
}
_ => {}
```

**Change 3** - Update help text (around line 678):
```rust
Line::from("  F10/Esc - Exit application"),
// ADD THIS LINE:
Line::from("  Ctrl+L  - Clear chat history"),
Line::from(""),
```

## Step 6: Test Your Change

1. **Build the project**:
   ```bash
   cd /home/rsbiiw/projects/ai-terminal
   cargo build
   ```

2. **Run and test**:
   ```bash
   ./target/debug/ai-terminal
   ```
   - Type some messages
   - Press `Ctrl+L` to clear
   - Press `F1` to see help with your new shortcut

## 🤔 Understanding What We Did

### Rust Concepts Used:

1. **Pattern Matching with Guards**:
   ```rust
   KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL)
   ```
   This matches the 'l' key BUT ONLY if Ctrl is held down.

2. **Method Chaining**:
   ```rust
   self.messages.clear();  // Method call on self
   ```
   
3. **Struct Instantiation**:
   ```rust
   Message {
       role: "system".to_string(),
       content: "🧹 Chat cleared!".to_string(), 
       timestamp: Instant::now(),
   }
   ```

4. **Importing Modules**:
   ```rust
   use crossterm::event::KeyModifiers;
   ```

### Why This Works:

- **Event-driven architecture**: The app waits for keyboard events
- **Match expressions**: Route different keys to different actions
- **Mutable self**: `&mut self` allows us to modify the app state
- **Message system**: Everything is a message in the chat

## 🚀 Try These Next:

1. **Add Ctrl+D to delete last message**
2. **Add F4 to save chat to file**
3. **Add arrow keys to scroll through message history**
4. **Add Ctrl+R to retry last AI response**

## 💡 Rust Learning Tips:

1. **The compiler is your friend** - it tells you exactly what's wrong
2. **Use `cargo check`** - faster than full build, just checks for errors
3. **Read error messages carefully** - Rust errors are very helpful
4. **Use the `?` operator** - it makes error handling much cleaner

## 🔧 Common Issues:

**If you get "borrowed value" errors:**
- You might need to use `.clone()` to make a copy
- Or restructure to avoid borrowing conflicts

**If imports don't work:**
- Check spelling and make sure the module is public (`pub`)
- Look at existing imports for examples

**If pattern matching fails:**
- Make sure you handle all cases or add a catch-all `_ => {}`

---

This example shows how Rust's type system and pattern matching make code both safe and expressive. The compiler ensures you don't forget to handle cases, and the ownership system prevents many common bugs!
