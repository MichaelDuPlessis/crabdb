#!/usr/bin/env python3

# This code was written by an LLM. I couldn't be bothered to write simple test
# program from scratch
import socket
import struct
import argparse
import sys
import shlex  # For splitting input lines in interactive mode
import signal  # For handling Ctrl+C

# Define constants for data types and request types as per the protocol.
TYPE_NULL = 0
TYPE_INT = 1
TYPE_TEXT = 2
TYPE_LIST = 3
TYPE_MAP = 4
TYPE_LINK = 5

REQUEST_GET = 0
REQUEST_SET = 1
REQUEST_DELETE = 2
REQUEST_CLOSE = 255

# Define a mapping from string names to type codes for user input.
DATA_TYPE_MAP = {
    "int": TYPE_INT,
    "text": TYPE_TEXT,
    "null": TYPE_NULL,
    "list": TYPE_LIST,
    "map": TYPE_MAP,
    "link": TYPE_LINK,
}


def _read_full_response(sock):
    """
    Reads the full response from the socket based on the 8-byte length prefix.
    Returns (response_data_content, response_total_len) or (None, error_message) on error.
    """
    try:
        # Read the 8-byte response length
        len_bytes = sock.recv(8)
        if not len_bytes:
            return None, "Error: Did not receive response length from server. Connection closed?"
        if len(len_bytes) < 8:
            return None, f"Error: Incomplete length header received. Expected 8 bytes, got {len(len_bytes)}."
        response_total_len = struct.unpack('>Q', len_bytes)[0]

        # Read the remaining response data as indicated by response_total_len
        response_data_content = b''
        bytes_received = 0
        while bytes_received < response_total_len:
            # Read in chunks, ensure we don't try to read more than available or remaining
            chunk = sock.recv(min(response_total_len - bytes_received, 4096))
            if not chunk:
                return None, "Error: Connection closed prematurely by server while reading response data content."
            response_data_content += chunk
            bytes_received += len(chunk)

        return response_data_content, response_total_len
    except socket.error as e:
        return None, f"Socket communication error: {e}"
    except Exception as e:
        return None, f"An unexpected error occurred during response reading: {e}"


def execute_command_on_socket(sock, payload):
    """
    Sends a payload on an *existing, open* socket and reads the response.
    Returns the parsed response string.
    """
    try:
        sock.sendall(payload)
        response_data_content, response_total_len = _read_full_response(sock)

        if response_data_content is None:
            # _read_full_response returned an error string in response_total_len
            return response_total_len

        return parse_server_response(response_data_content, response_total_len)
    except socket.error as e:
        return f"Socket communication error: {e}"
    except Exception as e:
        return f"An unexpected error occurred during command execution: {e}"


def execute_command_once(host, port, payload):
    """
    Establishes a new TCP connection, sends the payload, reads the response,
    and then closes the connection. Suitable for single commands.
    """
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect((host, port))
            # <--- THIS LINE WAS MISSING AND HAS BEEN ADDED BACK
            s.sendall(payload)
            response_data_content, response_total_len = _read_full_response(s)

            if response_data_content is None:
                return response_total_len  # Contains the error message from _read_full_response

            return parse_server_response(response_data_content, response_total_len)
    except socket.error as e:
        return f"Socket error: {e}"
    except Exception as e:
        return f"An unexpected error occurred: {e}"


def parse_server_response(response_data_content, declared_len):
    """
    Parses the binary response content received from the database.
    This content is everything *after* the initial 8-byte Response Length.
    """
    if not response_data_content:
        return "Empty response content received from server."

    # Check if the content indicates an error (single byte 255)
    # Per the latest clarification: Error is just 8-byte length (which would be 1) + 1-byte 255
    if len(response_data_content) == 1 and struct.unpack('>B', response_data_content[0:1])[0] == 255:
        return "SERVER ERROR: Operation failed (code 255)."

    # Otherwise, it must be a successful data object (Data Type + Data Payload)
    if len(response_data_content) < 1:
        return "ERROR: Malformed successful response: missing Data Type byte."

    data_type = struct.unpack('>B', response_data_content[0:1])[0]
    cursor = 1  # Points to the start of the Data Payload

    if data_type == TYPE_NULL:
        # Null has no payload. Declared length must match 1 (for the Data Type byte itself).
        if declared_len != 1:
            return f"ERROR: Malformed NULL response. Declared length {declared_len} but expected 1."
        return "Result: None (Null Value)"
    elif data_type == TYPE_INT:
        # Int payload is 8 bytes. Declared length must match 1 (Data Type) + 8 (Int payload).
        expected_len = 1 + 8
        if declared_len != expected_len or len(response_data_content) < cursor + 8:
            return f"ERROR: Incomplete INT data payload. Declared length {declared_len}, content length {len(response_data_content) - cursor} bytes, expected {8}."
        int_value = struct.unpack(
            '>q', response_data_content[cursor:cursor+8])[0]
        return f"Result (Int): {int_value}"
    elif data_type == TYPE_TEXT:
        # Text payload has a 2-byte length prefix + 'm' bytes of text data.
        if len(response_data_content) < cursor + 2:
            return f"ERROR: Incomplete TEXT length. Received {len(response_data_content) - cursor} bytes, expected 2."
        text_len = struct.unpack(
            '>H', response_data_content[cursor:cursor+2])[0]
        cursor += 2

        # 1 for Data Type, 2 for text length, text_len for text data
        expected_len = 1 + 2 + text_len
        if declared_len != expected_len or len(response_data_content) < cursor + text_len:
            return f"ERROR: Incomplete TEXT data payload. Declared length {declared_len}, content length {len(response_data_content) - cursor} bytes, expected {text_len}."
        text_value = response_data_content[cursor:cursor +
                                           text_len].decode('utf-8')
        return f"Result (Text): '{text_value}'"
    elif data_type == TYPE_LIST:
        # List payload has a 2-byte count prefix + serialized objects
        if len(response_data_content) < cursor + 2:
            return f"ERROR: Incomplete LIST count. Received {len(response_data_content) - cursor} bytes, expected 2."
        list_count = struct.unpack(
            '>H', response_data_content[cursor:cursor+2])[0]
        cursor += 2

        result, _ = _deserialize_list(
            response_data_content, cursor, list_count)
        return f"Result (List): {result}"
    elif data_type == TYPE_MAP:
        # Map payload has a 2-byte field count prefix + field entries
        if len(response_data_content) < cursor + 2:
            return f"ERROR: Incomplete MAP field count. Received {len(response_data_content) - cursor} bytes, expected 2."
        field_count = struct.unpack(
            '>H', response_data_content[cursor:cursor+2])[0]
        cursor += 2

        result, _ = _deserialize_map(
            response_data_content, cursor, field_count)
        return f"Result (Map): {result}"
    elif data_type == TYPE_LINK:
        # Link payload has a 2-byte length prefix + key data (same format as Key)
        if len(response_data_content) < cursor + 2:
            return f"ERROR: Incomplete LINK length. Received {len(response_data_content) - cursor} bytes, expected 2."
        link_len = struct.unpack(
            '>H', response_data_content[cursor:cursor+2])[0]
        cursor += 2
        
        # Check if we have enough data for the link key
        expected_len = 1 + 2 + link_len  # Data Type + Length + Key
        if declared_len != expected_len or len(response_data_content) < cursor + link_len:
            return f"ERROR: Incomplete LINK data payload. Declared length {declared_len}, content length {len(response_data_content) - cursor} bytes, expected {link_len}."
        link_key = response_data_content[cursor:cursor + link_len].decode('utf-8')
        return f"Result (Link): -> '{link_key}'"
    else:
        return f"ERROR: Unknown Data Type received in successful response: {data_type} (Expected 0, 1, 2, 3, 4, or 5)."


def construct_set_payload(key, value_str, value_type):
    """
    Constructs the binary payload for a SET request.
    """
    key_bytes = key.encode('utf-8')
    key_len_bytes = struct.pack('>H', len(key_bytes))  # 2 bytes for key length

    # --- Construct the data payload based on type ---
    data_type_byte = struct.pack('>B', value_type)
    data_payload = b''
    if value_type == TYPE_INT:
        try:
            # Pack as a signed 8-byte integer (long long)
            data_payload = struct.pack('>q', int(value_str))
        except ValueError:
            raise ValueError(f"'{value_str}' is not a valid integer.")
    elif value_type == TYPE_TEXT:
        value_bytes = value_str.encode('utf-8')
        # Pack the text length (2 bytes) followed by the text itself
        data_payload = struct.pack('>H', len(value_bytes)) + value_bytes
    elif value_type == TYPE_NULL:
        # Null has no data payload
        pass
    elif value_type == TYPE_LIST:
        # Parse JSON-like list format: [item1, item2, ...]
        data_payload = _serialize_list(value_str)
    elif value_type == TYPE_MAP:
        # Parse JSON-like map format: {"key1": value1, "key2": value2}
        data_payload = _serialize_map(value_str)
    elif value_type == TYPE_LINK:
        # Link type uses the same format as a key: 2-byte length + UTF-8 key data
        link_key_bytes = value_str.encode('utf-8')
        data_payload = struct.pack('>H', len(link_key_bytes)) + link_key_bytes
    else:
        raise ValueError(f"Unsupported data type for SET: {value_type}")

    # --- Assemble the request-specific part ---
    request_specific_part = key_len_bytes + \
        key_bytes + data_type_byte + data_payload

    # --- Assemble the full payload ---
    request_type_byte = struct.pack('>B', REQUEST_SET)
    # The total length is the length of everything *after* the initial 8-byte length field.
    total_len = len(request_type_byte) + len(request_specific_part)
    total_len_bytes = struct.pack('>Q', total_len)

    return total_len_bytes + request_type_byte + request_specific_part


def construct_get_payload(key):
    """
    Constructs the binary payload for a GET request without parameters.
    """
    key_bytes = key.encode('utf-8')

    # --- Assemble the request-specific part ---
    request_specific_part = struct.pack('>H', len(key_bytes)) + key_bytes

    # --- Assemble the full payload ---
    request_type_byte = struct.pack('>B', REQUEST_GET)
    # The total length is the length of everything *after* the initial 8-byte length field.
    total_len = len(request_type_byte) + len(request_specific_part)
    total_len_bytes = struct.pack('>Q', total_len)

    return total_len_bytes + request_type_byte + request_specific_part


def construct_get_payload_with_params(key, link_resolution_depth=None):
    """
    Constructs the binary payload for a GET request with optional parameters.
    
    Args:
        key: The key to retrieve
        link_resolution_depth: Optional maximum link resolution depth (0-255)
    """
    key_bytes = key.encode('utf-8')
    
    # Start with key
    request_specific_part = struct.pack('>H', len(key_bytes)) + key_bytes
    
    # Add parameters if any
    if link_resolution_depth is not None:
        # Add number of parameters (1 byte)
        request_specific_part += struct.pack('>B', 1)  # 1 parameter
        
        # Add link resolution parameter (type 1, 1-byte value)
        request_specific_part += struct.pack('>B', 1)  # Parameter type: Link Resolution
        request_specific_part += struct.pack('>B', link_resolution_depth)  # Max resolution depth
    
    # --- Assemble the full payload ---
    request_type_byte = struct.pack('>B', REQUEST_GET)
    total_len = len(request_type_byte) + len(request_specific_part)
    total_len_bytes = struct.pack('>Q', total_len)

    return total_len_bytes + request_type_byte + request_specific_part


def construct_delete_payload(key):
    """
    Constructs the binary payload for a DELETE request.
    """
    # --- Encode the key ---
    key_bytes = key.encode('utf-8')
    key_len = len(key_bytes)
    if key_len > 65535:  # 2^16 - 1
        raise ValueError(f"Key too long: {key_len} bytes (max 65535)")
    key_len_bytes = struct.pack('>H', key_len)

    # --- Assemble the request-specific part ---
    request_specific_part = key_len_bytes + key_bytes

    # --- Assemble the full payload ---
    request_type_byte = struct.pack('>B', REQUEST_DELETE)
    # The total length is the length of everything *after* the initial 8-byte length field.
    total_len = len(request_type_byte) + len(request_specific_part)
    total_len_bytes = struct.pack('>Q', total_len)

    # Combine all parts
    payload = total_len_bytes + request_type_byte + request_specific_part
    return payload


def construct_close_payload():
    """
    Constructs the binary payload for a CLOSE request.
    The CLOSE command has no additional data, just the command type.
    """
    # --- Assemble the full payload ---
    request_type_byte = struct.pack('>B', REQUEST_CLOSE)
    # The total length is just the length of the command type byte
    total_len = len(request_type_byte)
    total_len_bytes = struct.pack('>Q', total_len)

    return total_len_bytes + request_type_byte


def send_close_request(sock):
    """
    Send a close request to the server and handle the response.
    Returns True if successful, False otherwise.
    """
    try:
        close_payload = construct_close_payload()
        sock.send(close_payload)
        print("Close request sent to server.")

        # The server should close the connection after receiving CLOSE
        # We don't expect a response, but we'll try to read to see if connection closes
        try:
            response = sock.recv(1024)
            if not response:
                print("Server closed the connection gracefully.")
                return True
            else:
                print("Server sent unexpected response to close request.")
                return True
        except socket.error:
            # Connection closed by server, which is expected
            print("Server closed the connection.")
            return True

    except socket.error as e:
        print(f"Error sending close request: {e}")
        return False


def interactive_mode(initial_host, initial_port):
    """
    Runs the CLI client in interactive mode, keeping the database connection open.
    """
    current_host = initial_host
    current_port = initial_port
    db_socket = None

    def establish_connection():
        nonlocal db_socket
        if db_socket:
            try:
                db_socket.close()
                print("Closing existing connection.")
            except socket.error as e:
                print(f"Error closing existing socket: {e}", file=sys.stderr)
            db_socket = None

        print(f"Attempting to connect to {current_host}:{current_port}...")
        try:
            new_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            new_socket.connect((current_host, current_port))
            db_socket = new_socket
            print("Connection established.")
            return True
        except socket.error as e:
            print(f"Failed to connect to {current_host}:{
                  current_port}: {e}", file=sys.stderr)
            db_socket = None
            return False
        except Exception as e:
            print(f"An unexpected error occurred during connection: {
                  e}", file=sys.stderr)
            db_socket = None
            return False

    # Establish initial connection
    if not establish_connection():
        print("Starting interactive mode without an active connection. Use 'connect' to establish one.")

    # Set up signal handler for graceful shutdown on Ctrl+C
    def signal_handler(signum, frame):
        print("\n\nReceived interrupt signal (Ctrl+C)")
        if db_socket:
            print("Sending close request to server...")
            send_close_request(db_socket)
            try:
                db_socket.close()
            except:
                pass
        print("Exiting interactive mode.")
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)

    print("\nEntering interactive mode.")
    print("Commands: set <key> <value> --type <int|text|null|list|map|link> | get <key> [--resolve-links <depth>] | delete <key> | close | connect <host>:<port> | exit | quit")
    print("Quote values with spaces, e.g., set \"my key\" \"my value\" --type text")
    print("Use 'link' type to reference other keys, e.g., set mylink mykey --type link")
    print("Use --resolve-links with GET to automatically resolve link references")
    print("Press Ctrl+C to send close request and exit gracefully.")

    try:
        while True:
            try:
                # Update prompt to show connection status
                status_indicator = ' disconnected' if db_socket is None else ''
                user_input = input(f"CrabDB ({current_host}:{current_port}{
                                   status_indicator})> ").strip()
                if not user_input:
                    continue

                parts = shlex.split(user_input)
                command = parts[0].lower()

                if command in ['exit', 'quit']:
                    if db_socket:
                        print("Sending close request to server...")
                        send_close_request(db_socket)
                    print("Exiting interactive mode.")
                    break
                elif command == 'connect':
                    if len(parts) == 2:
                        try:
                            host_port_str = parts[1]
                            if ':' in host_port_str:
                                new_host, new_port_str = host_port_str.split(
                                    ':')
                                new_port = int(new_port_str)
                                current_host = new_host
                                current_port = new_port
                                establish_connection()  # Attempt to connect to new host/port
                            else:
                                print(
                                    "Invalid connect format. Use: connect <host>:<port>")
                        except ValueError:
                            print("Invalid port number.")
                        except Exception as e:
                            print(f"Error changing connection: {e}")
                    else:
                        print("Usage: connect <host>:<port>")
                elif command == 'set':
                    if db_socket is None:
                        print(
                            "Error: Not connected to the database. Use 'connect' first.")
                        continue

                    if '--type' not in parts:
                        print("Error: Missing --type argument for set command.")
                        continue

                    try:
                        type_arg_index = parts.index('--type')
                        if type_arg_index + 1 >= len(parts):
                            print(
                                "Error: --type argument requires a value (int, text, null).")
                            continue

                        cmd_type = parts[type_arg_index + 1].lower()
                        if cmd_type not in DATA_TYPE_MAP:
                            print(f"Error: Invalid type '{cmd_type}'. Must be int, text, null, list, map, or link.")
                            continue

                        # Ensure enough parts for key and value before --type
                        # e.g., ['set', 'key', 'value', '--type', 'text'] => parts[1] is key, parts[2] is value
                        if type_arg_index < 2:
                            print(
                                "Error: Invalid format for set. Usage: set <key> <value> --type <type>")
                            continue

                        key = parts[1]
                        # Join all parts between the key and '--type' as the value
                        value = ' '.join(parts[2:type_arg_index])

                        value_type_code = DATA_TYPE_MAP[cmd_type]

                        if value_type_code == TYPE_NULL:
                            value_to_send = ''  # Value string is ignored for Null type
                        else:
                            value_to_send = value

                        payload = construct_set_payload(
                            key, value_to_send, value_type_code)
                        print(f"Sending SET request for key='{key}'...")
                        response = execute_command_on_socket(
                            db_socket, payload)
                        print("--- Server Response ---")
                        print(response)

                    except ValueError as ve:
                        print(f"Error parsing set command: {ve}")
                    except IndexError:
                        print(
                            "Error: Invalid format for set. Usage: set <key> <value> --type <type>")
                    except Exception as e:
                        print(f"An unexpected error occurred during set: {e}")

                elif command == 'get':
                    if db_socket is None:
                        print(
                            "Error: Not connected to the database. Use 'connect' first.")
                        continue

                    if len(parts) < 2:
                        print("Usage: get <key> [--resolve-links <depth>]")
                        continue

                    key = parts[1]
                    
                    # Check for --resolve-links parameter
                    link_resolution_depth = None
                    if len(parts) >= 4 and parts[2] == '--resolve-links':
                        try:
                            link_resolution_depth = int(parts[3])
                            if link_resolution_depth < 0 or link_resolution_depth > 255:
                                print("Error: Link resolution depth must be between 0 and 255")
                                continue
                        except ValueError:
                            print("Error: Link resolution depth must be a valid integer")
                            continue
                    elif len(parts) > 2:
                        print("Usage: get <key> [--resolve-links <depth>]")
                        continue
                    
                    if link_resolution_depth is not None:
                        payload = construct_get_payload_with_params(key, link_resolution_depth)
                        print(f"Sending GET request for key='{key}' with link resolution depth={link_resolution_depth}...")
                    else:
                        payload = construct_get_payload(key)
                        print(f"Sending GET request for key='{key}'...")
                    
                    response = execute_command_on_socket(db_socket, payload)
                    print("--- Server Response ---")
                    print(response)

                elif command == 'delete':
                    if db_socket is None:
                        print(
                            "Error: Not connected to the database. Use 'connect' first.")
                        continue

                    if len(parts) != 2:
                        print("Usage: delete <key>")
                        continue

                    key = parts[1]
                    payload = construct_delete_payload(key)
                    print(f"Sending DELETE request for key='{key}'...")
                    response = execute_command_on_socket(db_socket, payload)
                    print("--- Server Response ---")
                    print(response)

                elif command == 'close':
                    if db_socket is None:
                        print("Error: Not connected to the database.")
                        continue

                    print("Sending close request to server...")
                    if send_close_request(db_socket):
                        try:
                            db_socket.close()
                        except:
                            pass
                        db_socket = None
                        print("Connection closed.")
                    else:
                        print("Failed to send close request properly.")

                else:
                    print(f"Unknown command: '{command}'.")

            except EOFError:  # User pressed Ctrl+D
                print("\nExiting interactive mode.")
                break
            except Exception as e:
                print(f"An unhandled error occurred in interactive mode: {e}")
    finally:
        # Ensure the socket is closed when interactive mode exits for any reason
        if db_socket:
            try:
                print("Sending close request to server before exit...")
                send_close_request(db_socket)
                db_socket.close()
                print("Database connection closed.")
            except socket.error as e:
                print(f"Error closing socket on exit: {e}", file=sys.stderr)


def main():
    """
    Main function to parse command-line arguments and execute commands.
    Supports single command execution or interactive mode.
    """
    parser = argparse.ArgumentParser(
        description="CLI client for a custom TCP database.",
        formatter_class=argparse.RawTextHelpFormatter
    )
    parser.add_argument('--host', default='127.0.0.1',
                        help='The server host address (default: 127.0.0.1).\n'
                             'Used for single commands or as initial host for interactive mode.')
    parser.add_argument('--port', type=int, default=7227,
                        help='The server port (default: 7227).\n'
                             'Used for single commands or as initial port for interactive mode.')

    parser.add_argument('--interactive', '-i', action='store_true',
                        help='Run the client in interactive mode.\n'
                             'Ignores other command-line arguments if present.')

    subparsers = parser.add_subparsers(
        dest='command', help='Available single commands.\n'
        'Use --interactive for interactive mode.')

    # --- Parser for the "set" command ---
    parser_set = subparsers.add_parser(
        'set', help='Set a key-value pair in the database.')
    parser_set.add_argument('key', help='The key to set.')
    parser_set.add_argument(
        'value', help='The value to associate with the key. For "null" type, the value will be ignored.')
    parser_set.add_argument(
        '--type', choices=['int', 'text', 'null', 'list', 'map', 'link'], required=True, 
        help='The data type of the value. Use "link" to reference another key.')

    # --- Parser for the "get" command ---
    parser_get = subparsers.add_parser(
        'get', help='Get a value by its key from the database.')
    parser_get.add_argument('key', help='The key to retrieve.')
    parser_get.add_argument('--resolve-links', type=int, metavar='DEPTH',
                           help='Enable link resolution with maximum depth (0-255). '
                                'Links will be automatically resolved to their target objects.')

    # --- Parser for the "delete" command ---
    parser_delete = subparsers.add_parser(
        'delete', help='Delete a key-value pair from the database.')
    parser_delete.add_argument('key', help='The key to delete.')

    # --- Parser for the "updated_time" command ---
    parser_updated_time = subparsers.add_parser(
        'updated_time', help='Get the updated timestamp for a key from the database.')
    parser_updated_time.add_argument(
        'key', help='The key to get the timestamp for.')

    # --- Parser for the "close" command ---
    parser_close = subparsers.add_parser(
        'close', help='Send a close/shutdown request to the server.')

    args = parser.parse_args()

    if args.interactive:
        # If interactive flag is present, ignore any other command and go interactive
        interactive_mode(args.host, args.port)
    elif args.command:  # A specific command was provided (not interactive)
        payload = b''
        try:
            if args.command == 'set':
                print(f"Executing SET: key='{args.key}', value='{
                      args.value}', type='{args.type}'")

                value_type_code = DATA_TYPE_MAP[args.type]

                if value_type_code == TYPE_NULL:
                    value_to_send = ''
                else:
                    value_to_send = args.value

                payload = construct_set_payload(
                    args.key, value_to_send, value_type_code)
            elif args.command == 'get':
                if hasattr(args, 'resolve_links') and args.resolve_links is not None:
                    print(f"Executing GET: key='{args.key}' with link resolution depth={args.resolve_links}")
                    payload = construct_get_payload_with_params(args.key, args.resolve_links)
                else:
                    print(f"Executing GET: key='{args.key}'")
                    payload = construct_get_payload(args.key)
            elif args.command == 'delete':
                print(f"Executing DELETE: key='{args.key}'")
                payload = construct_delete_payload(args.key)
            elif args.command == 'close':
                print("Executing CLOSE: sending shutdown request to server")
                payload = construct_close_payload()

            if payload:
                print(f"Sending {len(payload)} bytes to {
                      args.host}:{args.port}")
                # Use execute_command_once for single command execution
                parsed_response = execute_command_once(
                    args.host, args.port, payload)
                print("\n--- Server Response ---")
                print(parsed_response)
        except (ValueError, socket.error) as e:
            print(f"Error during command execution: {e}", file=sys.stderr)
            sys.exit(1)
        except Exception as e:
            print(f"An unexpected error occurred: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        # If no interactive flag and no command specified (e.g., just `./cli.py`),
        # print usage and suggest interactive mode.
        parser.print_help()
        print("\nTip: Use '--interactive' or '-i' to enter interactive mode.")


def _serialize_object(value):
    """
    Serialize a single JSON value to CrabDB object format.
    Handles null, int, string, list, and map (dict) values recursively.
    """
    if value is None:
        # Serialize null
        return struct.pack('>B', TYPE_NULL)
    elif isinstance(value, int):
        # Serialize int
        return struct.pack('>B', TYPE_INT) + struct.pack('>q', value)
    elif isinstance(value, str):
        # Serialize text
        text_bytes = value.encode('utf-8')
        return (
            struct.pack('>B', TYPE_TEXT) +
            struct.pack('>H', len(text_bytes)) +
            text_bytes
        )
    elif isinstance(value, list):
        # Serialize list recursively
        serialized_objects = []
        for item in value:
            serialized_objects.append(_serialize_object(item))

        serialized_data = b''.join(serialized_objects)
        return (
            struct.pack('>B', TYPE_LIST) +
            struct.pack('>H', len(value)) +
            serialized_data
        )
    elif isinstance(value, dict):
        # Serialize map recursively
        serialized_fields = []
        for key, val in value.items():
            if not isinstance(key, str):
                raise ValueError("Map keys must be strings")

            # Serialize field name
            key_bytes = key.encode('utf-8')
            field_data = struct.pack('>H', len(key_bytes)) + key_bytes

            # Serialize field value recursively
            field_data += _serialize_object(val)

            serialized_fields.append(field_data)

        serialized_data = b''.join(serialized_fields)
        return (
            struct.pack('>B', TYPE_MAP) +
            struct.pack('>H', len(value)) +
            serialized_data
        )
    else:
        raise ValueError(f"Unsupported JSON value type: {type(value)}")


def _serialize_list(value_str):
    """
    Serialize a list from JSON-like string format.
    Supports nested structures: [1, "hello", null, [1, 2], {"key": "value"}]
    """
    import json
    try:
        # Parse as JSON array
        items = json.loads(value_str)
        if not isinstance(items, list):
            raise ValueError("List value must be a JSON array")

        serialized_objects = []
        for item in items:
            serialized_objects.append(_serialize_object(item))

        # Combine all serialized objects
        serialized_data = b''.join(serialized_objects)

        # Return count + data
        return struct.pack('>H', len(items)) + serialized_data

    except json.JSONDecodeError:
        raise ValueError("List value must be valid JSON array format")


def _serialize_map(value_str):
    """
    Serialize a map from JSON-like string format.
    Supports nested structures: {"key1": value1, "nested": {"inner": "value"}, "list": [1, 2, 3]}
    """
    import json
    try:
        # Parse as JSON object
        obj = json.loads(value_str)
        if not isinstance(obj, dict):
            raise ValueError("Map value must be a JSON object")

        serialized_fields = []
        for key, value in obj.items():
            if not isinstance(key, str):
                raise ValueError("Map keys must be strings")

            # Serialize field name
            key_bytes = key.encode('utf-8')
            field_data = struct.pack('>H', len(key_bytes)) + key_bytes

            # Serialize field value using the generic object serializer
            field_data += _serialize_object(value)

            serialized_fields.append(field_data)

        # Combine all serialized fields
        serialized_data = b''.join(serialized_fields)

        # Return field count + data
        return struct.pack('>H', len(obj)) + serialized_data

    except json.JSONDecodeError:
        raise ValueError("Map value must be valid JSON object format")


def _deserialize_object(data, cursor):
    """
    Deserialize a single object from binary data.
    Handles all CrabDB types recursively: null, int, text, list, map.
    Returns (value, new_cursor_position).
    """
    if cursor >= len(data):
        raise ValueError("Incomplete object data")

    # Read type ID
    type_id = data[cursor]
    cursor += 1

    if type_id == TYPE_NULL:
        return None, cursor
    elif type_id == TYPE_INT:
        if cursor + 8 > len(data):
            raise ValueError("Incomplete int data")
        value = struct.unpack('>q', data[cursor:cursor+8])[0]
        return value, cursor + 8
    elif type_id == TYPE_TEXT:
        if cursor + 2 > len(data):
            raise ValueError("Incomplete text length")
        text_len = struct.unpack('>H', data[cursor:cursor+2])[0]
        cursor += 2
        if cursor + text_len > len(data):
            raise ValueError("Incomplete text data")
        text_value = data[cursor:cursor+text_len].decode('utf-8')
        return text_value, cursor + text_len
    elif type_id == TYPE_LIST:
        if cursor + 2 > len(data):
            raise ValueError("Incomplete list count")
        list_count = struct.unpack('>H', data[cursor:cursor+2])[0]
        cursor += 2
        return _deserialize_list(data, cursor, list_count)
    elif type_id == TYPE_MAP:
        if cursor + 2 > len(data):
            raise ValueError("Incomplete map field count")
        field_count = struct.unpack('>H', data[cursor:cursor+2])[0]
        cursor += 2
        return _deserialize_map(data, cursor, field_count)
    elif type_id == TYPE_LINK:
        if cursor + 2 > len(data):
            raise ValueError("Incomplete link length")
        link_len = struct.unpack('>H', data[cursor:cursor+2])[0]
        cursor += 2
        if cursor + link_len > len(data):
            raise ValueError("Incomplete link data")
        link_key = data[cursor:cursor+link_len].decode('utf-8')
        return f"Link -> '{link_key}'", cursor + link_len
    else:
        raise ValueError(f"Unsupported type ID: {type_id}")


def _deserialize_list(data, cursor, count):
    """Deserialize a list from binary data, supporting nested structures."""
    items = []
    for _ in range(count):
        if cursor >= len(data):
            raise ValueError("Incomplete list data")

        # Deserialize object recursively
        obj, cursor = _deserialize_object(data, cursor)
        items.append(obj)

    return items, cursor


def _deserialize_map(data, cursor, field_count):
    """Deserialize a map from binary data, supporting nested structures."""
    fields = {}
    for _ in range(field_count):
        if cursor + 2 > len(data):
            raise ValueError("Incomplete field name length in map")

        # Read field name length
        name_len = struct.unpack('>H', data[cursor:cursor+2])[0]
        cursor += 2

        # Read field name
        if cursor + name_len > len(data):
            raise ValueError("Incomplete field name in map")
        field_name = data[cursor:cursor+name_len].decode('utf-8')
        cursor += name_len

        # Deserialize field value recursively
        value, cursor = _deserialize_object(data, cursor)
        fields[field_name] = value

    return fields, cursor


if __name__ == '__main__':
    main()
