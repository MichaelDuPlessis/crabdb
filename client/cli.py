#!/usr/bin/env python3
"""
CrabDB CLI Interface

Provides both single-line and interactive modes for CrabDB client.
"""

import argparse
import json
import signal
import sys
from typing import Any, Optional

from crabdb_client import CrabDBClient, CrabDBError


class CrabDBCLI:
    """Command-line interface for CrabDB"""
    
    def __init__(self, host: str = "localhost", port: int = 7227):
        self.client = CrabDBClient(host, port)
        self.running = True
        
        # Set up signal handler for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
    
    def _signal_handler(self, signum, frame):
        """Handle Ctrl+C gracefully"""
        print("\nShutting down gracefully...")
        self.running = False
        self.client.disconnect()
        sys.exit(0)
    
    def _parse_value(self, value_str: str) -> Any:
        """Parse a string value into appropriate Python type"""
        # Try to parse as JSON first
        try:
            return json.loads(value_str)
        except json.JSONDecodeError:
            # If JSON parsing fails, treat as string
            return value_str
    
    def _format_output(self, value: Any) -> str:
        """Format a value for display"""
        if value is None:
            return "null"
        elif isinstance(value, dict) and "_link" in value:
            return f"Link -> {value['_link']}"
        else:
            return json.dumps(value, indent=2)
    
    def _execute_command(self, command: str) -> bool:
        """Execute a single command. Returns False if should exit."""
        parts = command.strip().split()
        if not parts:
            return True
        
        cmd = parts[0].lower()
        
        try:
            if cmd == "help":
                self._show_help()
            elif cmd == "quit" or cmd == "exit":
                return False
            elif cmd == "get":
                if len(parts) < 2:
                    print("Usage: get <key> [link_depth]")
                    return True
                
                key = parts[1]
                link_depth = None
                if len(parts) > 2:
                    try:
                        link_depth = int(parts[2])
                    except ValueError:
                        print("Link depth must be an integer")
                        return True
                
                result = self.client.get(key, link_depth)
                print(self._format_output(result))
            
            elif cmd == "set":
                if len(parts) < 3:
                    print("Usage: set <key> <value>")
                    return True
                
                key = parts[1]
                value_str = " ".join(parts[2:])
                value = self._parse_value(value_str)
                
                self.client.set(key, value)
                print(f"Set {key} = {self._format_output(value)}")
            
            elif cmd == "delete" or cmd == "del":
                if len(parts) < 2:
                    print("Usage: delete <key>")
                    return True
                
                key = parts[1]
                self.client.delete(key)
                print(f"Deleted {key}")
            
            elif cmd == "link":
                if len(parts) < 3:
                    print("Usage: link <key> <target_key>")
                    return True
                
                key = parts[1]
                target_key = parts[2]
                link_value = {"_link": target_key}
                
                self.client.set(key, link_value)
                print(f"Created link {key} -> {target_key}")
            
            else:
                print(f"Unknown command: {cmd}. Type 'help' for available commands.")
        
        except CrabDBError as e:
            print(f"Error: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")
        
        return True
    
    def _show_help(self):
        """Show help information"""
        print("""
CrabDB Client Commands:

  get <key> [link_depth]    - Get value by key, optionally resolve links
  set <key> <value>         - Set key to value (JSON format supported)
  delete <key>              - Delete key
  link <key> <target_key>   - Create a link from key to target_key
  help                      - Show this help
  quit/exit                 - Exit the client

Examples:
  set user1 "John Doe"
  set user2 {"name": "Jane", "age": 30}
  set numbers [1, 2, 3, 4, 5]
  link current_user user1
  get current_user 1        - Get with link resolution depth 1
  delete user1
        """)
    
    def run_interactive(self):
        """Run in interactive mode"""
        print("CrabDB Interactive Client")
        print("Type 'help' for commands or 'quit' to exit")
        print("Press Ctrl+C to exit gracefully")
        
        try:
            self.client.connect()
            print(f"Connected to {self.client.host}:{self.client.port}")
        except CrabDBError as e:
            print(f"Failed to connect: {e}")
            return 1
        
        try:
            while self.running:
                try:
                    command = input("crabdb> ").strip()
                    if not command:
                        continue
                    
                    if not self._execute_command(command):
                        break
                
                except EOFError:
                    # Handle Ctrl+D
                    print("\nGoodbye!")
                    break
                except KeyboardInterrupt:
                    # This will be handled by the signal handler
                    continue
        
        finally:
            self.client.disconnect()
        
        return 0
    
    def run_single_command(self, command: str) -> int:
        """Run a single command and exit"""
        try:
            self.client.connect()
        except CrabDBError as e:
            print(f"Failed to connect: {e}")
            return 1
        
        try:
            self._execute_command(command)
            return 0
        except Exception as e:
            print(f"Error: {e}")
            return 1
        finally:
            self.client.disconnect()


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="CrabDB Python Client")
    parser.add_argument("--host", default="localhost", help="CrabDB server host")
    parser.add_argument("--port", type=int, default=7227, help="CrabDB server port")
    parser.add_argument("--command", "-c", help="Execute single command and exit")
    
    args = parser.parse_args()
    
    cli = CrabDBCLI(args.host, args.port)
    
    if args.command:
        return cli.run_single_command(args.command)
    else:
        return cli.run_interactive()


if __name__ == "__main__":
    sys.exit(main())
