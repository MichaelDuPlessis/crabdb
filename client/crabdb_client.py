#!/usr/bin/env python3
"""
CrabDB Python Client

A Python client for interacting with CrabDB using the binary TCP protocol.
"""

import socket
import struct
import sys
from typing import Optional, Union, List, Dict, Any
from enum import IntEnum


class DataType(IntEnum):
    """CrabDB data types"""
    NULL = 0
    INT = 1
    TEXT = 2
    LIST = 3
    MAP = 4
    LINK = 5


class CommandType(IntEnum):
    """CrabDB command types"""
    GET = 0
    SET = 1
    DELETE = 2
    CLOSE = 255


class ParameterType(IntEnum):
    """CrabDB parameter types"""
    LINK_RESOLUTION = 1


class CrabDBError(Exception):
    """Base exception for CrabDB client errors"""
    pass


class CrabDBClient:
    """CrabDB client for TCP communication"""

    def __init__(self, host: str = "localhost", port: int = 7227):
        self.host = host
        self.port = port
        self.socket: Optional[socket.socket] = None
        self.connected = False

    def connect(self) -> None:
        """Connect to the CrabDB server"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            self.connected = True
        except Exception as e:
            raise CrabDBError(f"Failed to connect to {
                              self.host}:{self.port}: {e}")

    def disconnect(self) -> None:
        """Disconnect from the CrabDB server"""
        if self.socket and self.connected:
            try:
                # Send CLOSE command
                self._send_command(CommandType.CLOSE, b"")
            except:
                pass  # Ignore errors during close
            finally:
                self.socket.close()
                self.connected = False

    def _send_command(self, command_type: CommandType, payload: bytes) -> bytes:
        """Send a command and return the response"""
        if not self.connected or not self.socket:
            raise CrabDBError("Not connected to server")

        # Build request: length (8 bytes) + command type (1 byte) + payload
        request_data = struct.pack(">B", command_type) + payload
        request_length = len(request_data)
        request = struct.pack(">Q", request_length) + request_data

        # Send request
        self.socket.sendall(request)

        # Read response length
        response_length_data = self._recv_exact(8)
        response_length = struct.unpack(">Q", response_length_data)[0]

        # Read response data
        response_data = self._recv_exact(response_length)

        # Check for error response
        if response_length == 1 and response_data[0] == 255:
            raise CrabDBError("Server returned error")

        return response_data

    def _recv_exact(self, length: int) -> bytes:
        """Receive exactly the specified number of bytes"""
        data = b""
        while len(data) < length:
            chunk = self.socket.recv(length - len(data))
            if not chunk:
                raise CrabDBError("Connection closed unexpectedly")
            data += chunk
        return data

    def _encode_key(self, key: str) -> bytes:
        """Encode a key according to CrabDB format"""
        key_bytes = key.encode('utf-8')
        return struct.pack(">H", len(key_bytes)) + key_bytes

    def _encode_text(self, text: str) -> bytes:
        """Encode text according to CrabDB format"""
        text_bytes = text.encode('utf-8')
        return struct.pack(">H", len(text_bytes)) + text_bytes

    def _encode_value(self, value: Any) -> bytes:
        """Encode a value according to CrabDB format"""
        if value is None:
            return struct.pack(">B", DataType.NULL)
        elif isinstance(value, int):
            return struct.pack(">B", DataType.INT) + struct.pack(">q", value)
        elif isinstance(value, str):
            return struct.pack(">B", DataType.TEXT) + self._encode_text(value)
        elif isinstance(value, list):
            data = struct.pack(">BH", DataType.LIST, len(value))
            for item in value:
                # Include full type prefix for nested objects
                data += self._encode_value(item)
            return data
        elif isinstance(value, dict):
            if "_link" in value:  # Special case for link objects
                link_key = value["_link"]
                # Include full key format for links
                return struct.pack(">B", DataType.LINK) + self._encode_key(link_key)
            else:  # Regular map
                data = struct.pack(">BH", DataType.MAP, len(value))
                for field_name, field_value in value.items():
                    field_name_bytes = field_name.encode('utf-8')
                    data += struct.pack(">H", len(field_name_bytes)
                                        ) + field_name_bytes
                    # Include full type prefix for nested objects
                    data += self._encode_value(field_value)
                return data
        else:
            raise CrabDBError(f"Unsupported value type: {type(value)}")

    def _decode_value(self, data: bytes, offset: int = 0) -> tuple[Any, int]:
        """Decode a value from bytes, returning (value, new_offset)"""
        if offset >= len(data):
            raise CrabDBError("Unexpected end of data")

        data_type = data[offset]
        offset += 1

        if data_type == DataType.NULL:
            return None, offset
        elif data_type == DataType.INT:
            if offset + 8 > len(data):
                raise CrabDBError("Insufficient data for int")
            value = struct.unpack(">q", data[offset:offset + 8])[0]
            return value, offset + 8
        elif data_type == DataType.TEXT:
            if offset + 2 > len(data):
                raise CrabDBError("Insufficient data for text length")
            text_length = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            if offset + text_length > len(data):
                raise CrabDBError("Insufficient data for text")
            text = data[offset:offset + text_length].decode('utf-8')
            return text, offset + text_length
        elif data_type == DataType.LIST:
            if offset + 2 > len(data):
                raise CrabDBError("Insufficient data for list count")
            count = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            items = []
            for _ in range(count):
                item, offset = self._decode_value(data, offset)
                items.append(item)
            return items, offset
        elif data_type == DataType.MAP:
            if offset + 2 > len(data):
                raise CrabDBError("Insufficient data for map field count")
            field_count = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            result = {}
            for _ in range(field_count):
                # Read field name
                if offset + 2 > len(data):
                    raise CrabDBError(
                        "Insufficient data for field name length")
                name_length = struct.unpack(">H", data[offset:offset + 2])[0]
                offset += 2
                if offset + name_length > len(data):
                    raise CrabDBError("Insufficient data for field name")
                field_name = data[offset:offset + name_length].decode('utf-8')
                offset += name_length
                # Read field value
                field_value, offset = self._decode_value(data, offset)
                result[field_name] = field_value
            return result, offset
        elif data_type == DataType.LINK:
            if offset + 2 > len(data):
                raise CrabDBError("Insufficient data for link key length")
            key_length = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            if offset + key_length > len(data):
                raise CrabDBError("Insufficient data for link key")
            key = data[offset:offset + key_length].decode('utf-8')
            return {"_link": key}, offset + key_length
        else:
            raise CrabDBError(f"Unknown data type: {data_type}")

    def get(self, key: str, link_resolution_depth: Optional[int] = None) -> Any:
        """Get a value by key with optional link resolution"""
        payload = self._encode_key(key)

        # Add parameters if link resolution is requested
        if link_resolution_depth is not None:
            payload += struct.pack(">B", 1)  # Number of parameters
            payload += struct.pack(">BB", ParameterType.LINK_RESOLUTION,
                                   link_resolution_depth)
        else:
            payload += struct.pack(">B", 0)  # No parameters

        response = self._send_command(CommandType.GET, payload)
        value, _ = self._decode_value(response)
        return value

    def set(self, key: str, value: Any) -> None:
        """Set a key-value pair"""
        payload = self._encode_key(key)
        value_data = self._encode_value(value)
        payload += value_data

        self._send_command(CommandType.SET, payload)

    def delete(self, key: str) -> None:
        """Delete a key"""
        payload = self._encode_key(key)
        self._send_command(CommandType.DELETE, payload)

    def __enter__(self):
        """Context manager entry"""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.disconnect()
