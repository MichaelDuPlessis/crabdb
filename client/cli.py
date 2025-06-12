#!/usr/bin/env python3

# This code was written by an LLM. I couldn't be bothered to write simple test
# program from scratch

import socket
import struct
import argparse
import sys

# Define constants for data types and request types as per the protocol.
TYPE_NULL = 0
TYPE_INT = 1
TYPE_TEXT = 2

REQUEST_GET = 0
REQUEST_SET = 1

# Define a mapping from string names to type codes for user input.
DATA_TYPE_MAP = {
    "int": TYPE_INT,
    "text": TYPE_TEXT,
    "null": TYPE_NULL,
}


def send_and_receive(host, port, payload):
    """
    Establishes a TCP connection, sends the payload, and reads the full response
    based on the updated, conditional response protocol.
    """
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect((host, port))
            s.sendall(payload)

            # Read the 8-byte response length (always the first part of any response)
            len_bytes = s.recv(8)
            if not len_bytes:
                print(
                    "Error: Did not receive response length from server. Connection closed?", file=sys.stderr)
                sys.exit(1)
            if len(len_bytes) < 8:
                print(f"Error: Incomplete length header received. Expected 8 bytes, got {
                      len(len_bytes)}.", file=sys.stderr)
                sys.exit(1)
            response_total_len = struct.unpack('>Q', len_bytes)[0]

            # Read the remaining response data as indicated by response_total_len
            response_data_content = b''
            bytes_received = 0
            while bytes_received < response_total_len:
                chunk = s.recv(min(response_total_len - bytes_received, 4096))
                if not chunk:
                    print(
                        "Error: Connection closed prematurely by server while reading response data content.", file=sys.stderr)
                    sys.exit(1)
                response_data_content += chunk
                bytes_received += len(chunk)

            return parse_server_response(response_data_content, response_total_len)
    except socket.error as e:
        print(f"Socket error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"An unexpected error occurred: {e}", file=sys.stderr)
        sys.exit(1)


def parse_server_response(response_data_content, declared_len):
    """
    Parses the binary response content received from the database.
    This content is everything *after* the initial 8-byte Response Length.
    """
    if not response_data_content:
        return "Empty response content received from server."

    # Check if the content indicates an error (single byte 255)
    if len(response_data_content) == 1 and struct.unpack('>B', response_data_content[0:1])[0] == 255:
        # This matches the specific error response format: 8-byte length (which would be 1) + 1-byte 255
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
    else:
        return f"ERROR: Unknown Data Type received in successful response: {data_type} (Expected 0, 1, or 2)."


def construct_set_payload(key, value_str, value_type):
    """
    Constructs the binary payload for a SET request.

    Payload Structure:
    | 8 bytes: Total length of the request-specific part |
    | 1 byte:  Request type (1 for SET)                  |
    | 2 bytes: Key length (n)                            |
    | n bytes: Key (UTF-8 encoded)                       |
    | 1 byte:  Data type code (0=Null, 1=Int, 2=Text)    |
    | ...:     Data payload                              |
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
            print(
                f"Error: '{value_str}' is not a valid integer.", file=sys.stderr)
            sys.exit(1)
    elif value_type == TYPE_TEXT:
        value_bytes = value_str.encode('utf-8')
        # Pack the text length (2 bytes) followed by the text itself
        data_payload = struct.pack('>H', len(value_bytes)) + value_bytes
    elif value_type == TYPE_NULL:
        # Null has no data payload
        pass
    else:
        print(f"Error: Unsupported data type for SET: {
              value_type}", file=sys.stderr)
        sys.exit(1)

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
    Constructs the binary payload for a GET request.

    Payload Structure:
    | 8 bytes: Total length of the request-specific part |
    | 1 byte:  Request type (0 for GET)                  |
    | 2 bytes: Key length (n)                            |
    | n bytes: Key (UTF-8 encoded)                       |
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


def main():
    """
    Main function to parse command-line arguments and execute commands.
    """
    parser = argparse.ArgumentParser(
        description="CLI client for a custom TCP database.")
    parser.add_argument('--host', default='127.0.0.1',
                        help='The server host address.')
    parser.add_argument('--port', type=int, default=7227,
                        help='The server port.')

    subparsers = parser.add_subparsers(
        dest='command', required=True, help='Available commands')

    # --- Parser for the "set" command ---
    parser_set = subparsers.add_parser(
        'set', help='Set a key-value pair in the database.')
    parser_set.add_argument('key', help='The key to set.')
    parser_set.add_argument(
        'value', help='The value to associate with the key. For "null" type, use "None" or leave empty (will be ignored).')
    parser_set.add_argument(
        '--type', choices=['int', 'text', 'null'], required=True, help='The data type of the value.')

    # --- Parser for the "get" command ---
    parser_get = subparsers.add_parser(
        'get', help='Get a value by its key from the database.')
    parser_get.add_argument('key', help='The key to retrieve.')

    args = parser.parse_args()

    payload = b''
    if args.command == 'set':
        print(f"Executing SET: key='{args.key}', value='{
              args.value}', type='{args.type}'")

        value_type_code = DATA_TYPE_MAP[args.type]

        # For Null type, the 'value' argument from command line is irrelevant
        # We just pass an empty string, as it won't be used in the payload
        if value_type_code == TYPE_NULL:
            value_to_send = ''
        else:
            value_to_send = args.value

        payload = construct_set_payload(
            args.key, value_to_send, value_type_code)
    elif args.command == 'get':
        print(f"Executing GET: key='{args.key}'")
        payload = construct_get_payload(args.key)

    if payload:
        print(f"Sending {len(payload)} bytes to {args.host}:{args.port}")
        parsed_response = send_and_receive(args.host, args.port, payload)
        print("\n--- Server Response ---")
        print(parsed_response)


if __name__ == '__main__':
    main()
