#!/usr/bin/env python3
"""
Simple test script to verify keyboard input works in the terminal
"""

import sys
import select
import tty
import termios

def test_keyboard_input():
    print("Testing keyboard input... Press 'q' to quit or any other key to test:")
    
    # Save original terminal settings
    old_settings = termios.tcgetattr(sys.stdin)
    
    try:
        # Set terminal to raw mode
        tty.setraw(sys.stdin.fileno())
        
        while True:
            # Check if input is available
            if select.select([sys.stdin], [], [], 0) == ([sys.stdin], [], []):
                ch = sys.stdin.read(1)
                print(f"\rKey pressed: {repr(ch)}", end="", flush=True)
                if ch.lower() == 'q':
                    print("\nQuitting...")
                    break
            else:
                print(".", end="", flush=True)
                import time
                time.sleep(0.1)
    
    finally:
        # Restore terminal settings
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
        print("\nTerminal restored.")

if __name__ == "__main__":
    test_keyboard_input()
