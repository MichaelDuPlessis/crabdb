#!/usr/bin/env python3
"""
Simple test script for CrabDB Python client
"""

from crabdb_client import CrabDBClient, CrabDBError


def test_basic_operations():
    """Test basic database operations"""
    print("Testing CrabDB Python Client...")

    try:
        with CrabDBClient() as client:
            print("✓ Connected to CrabDB")

            # Test setting and getting different data types
            test_cases = [
                ("null_value", None),
                ("int_value", 42),
                ("text_value", "Hello, CrabDB!"),
                ("list_value", [1, 2, "three", None]),
                ("map_value", {"name": "Alice", "age": 30, "active": True}),
            ]

            for key, value in test_cases:
                client.set(key, value)
                retrieved = client.get(key)
                assert retrieved == value, f"Mismatch for {
                    key}: {retrieved} != {value}"
                print(f"✓ {key}: {value}")

            # Test link creation and resolution
            client.set("user1", {"name": "Bob", "role": "admin"})
            client.set("link_to_user", {"_link": "user1"})

            # Get link without resolution
            link_obj = client.get("link_to_user")
            assert link_obj == {
                "_link": "user1"}, f"Link object mismatch: {link_obj}"
            print("✓ Link creation")

            # Get link with resolution
            resolved = client.get("link_to_user", 1)
            expected = {"name": "Bob", "role": "admin"}
            assert resolved == expected, f"Link resolution mismatch: {
                resolved} != {expected}"
            print("✓ Link resolution")

            # Test deletion
            client.delete("null_value")
            result = client.get("null_value")
            assert result is None, f"Expected None for deleted key, got: {result}"
            print("✓ Deletion (returns None for non-existent keys)")

            print("\nAll tests passed! 🎉")

    except CrabDBError as e:
        print(f"❌ CrabDB Error: {e}")
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        return False

    return True


if __name__ == "__main__":
    success = test_basic_operations()
    exit(0 if success else 1)
