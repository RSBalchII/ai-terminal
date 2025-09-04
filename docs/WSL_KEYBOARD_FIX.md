# Fixing Keyboard Input Issues in WSL GUI

## Problem
The text box in the AI Terminal GUI application was not registering keyboard inputs when running in WSL2 with X11.

## Root Causes
1. **Missing Input Method Configuration**: WSL doesn't set up input method environment variables by default
2. **Winit Backend Issues**: The Rust winit library needs specific configuration for WSL/X11
3. **Focus Management**: Text input fields may lose focus or not properly capture keyboard events

## Solutions Applied

### 1. Environment Variables
Set these environment variables before running the GUI:

```bash
# Essential for Winit (Rust's window handling library)
export WINIT_X11_SCALE_FACTOR=1.0
export WINIT_UNIX_BACKEND=x11

# Input method configuration
export XMODIFIERS=@im=local
export GTK_IM_MODULE=xim
export QT_IM_MODULE=xim

# Backend specification
export GDK_BACKEND=x11
```

### 2. Code Improvements
- Added persistent ID to text input field: `TextEdit::multiline(&mut self.input_text).id(egui::Id::new("main_input"))`
- Added hint text for better UX
- Improved focus management
- Added debug logging for input events
- Made the Send button more prominent as a fallback

### 3. Launcher Script
Created `run-gui.sh` that sets up the proper environment automatically:

```bash
./run-gui.sh        # Normal mode
./run-gui.sh --debug # Debug mode with extra logging
```

## Usage

### Quick Start
```bash
# Use the launcher script (recommended)
./run-gui.sh

# Or set environment manually
export WINIT_UNIX_BACKEND=x11 XMODIFIERS=@im=local GTK_IM_MODULE=xim
cargo run --bin ai-terminal-gui
```

### Troubleshooting

If keyboard input still doesn't work:

1. **Check X11 connection**:
   ```bash
   echo $DISPLAY  # Should show :0 or similar
   xset q         # Should connect to X server
   ```

2. **Install missing packages**:
   ```bash
   sudo apt-get install x11-xkb-utils xsel
   ```

3. **Try alternative input methods**:
   - Use the Send button instead of keyboard shortcuts
   - Copy/paste text using Ctrl+C/Ctrl+V or middle mouse button
   - Try running with `WINIT_UNIX_BACKEND=wayland` if using Wayland

4. **Debug mode**:
   ```bash
   RUST_LOG=debug ./run-gui.sh --debug
   ```

## Technical Details

### Why This Happens in WSL
- WSL2 runs Linux in a VM and uses X11 forwarding for GUI applications
- The X11 server (usually VcXsrv or WSLg) may not properly forward all keyboard events
- Input method frameworks (IME) need explicit configuration
- Winit (Rust's windowing library) defaults may not work correctly in WSL

### The Fix Explained
1. **WINIT_UNIX_BACKEND=x11**: Forces Winit to use X11 instead of trying Wayland
2. **XMODIFIERS=@im=local**: Tells X11 to use local input method
3. **GTK_IM_MODULE=xim**: Uses X Input Method for GTK compatibility
4. **Focus management**: Ensures the text field maintains keyboard focus

## Alternative Solutions

If the above doesn't work, consider:
1. Using a different X server (VcXsrv, Xming, or MobaXterm)
2. Running the terminal UI version instead: `cargo run --bin ai-terminal`
3. Using WSLg (Windows 11) which has better GUI support
4. Running in a full Linux VM or dual-boot setup

## References
- [Winit Platform-specific Issues](https://github.com/rust-windowing/winit/wiki/Platform-specific-notes)
- [egui Input Handling](https://github.com/emilk/egui/blob/master/crates/egui/src/input_state.rs)
- [WSL GUI Applications Guide](https://learn.microsoft.com/en-us/windows/wsl/tutorials/gui-apps)
