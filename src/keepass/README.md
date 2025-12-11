# KeePass v1 (KDB) File Format Reference

This document describes the full binary structure of a **KeePass v1 (.kdb)** database.  
It is tailored for implementors building parsers/writers (including embedded systems).

KeePass v1 is a flat, streaming binary format consisting of a plaintext header followed by an AES-encrypted payload containing groups and entries.

---

## 1. High-Level File Structure

```
+------------------------------+
|        HEADER (PLAINTEXT)    |
+------------------------------+
|     ENCRYPTED PAYLOAD        |
|     (AES-256-CBC)            |
|   +-----------------------+  |
|   | GROUP TABLE (N groups)|  |
|   +-----------------------+  |
|   | ENTRY TABLE (M entries)| |
|   +-----------------------+  |
|   | END MARKER             | |
|   +------------------------+ |
+------------------------------+
```

Groups and entries use the same TLV structure, but they appear in **separate sections**, and their counts are given in the header.

---

## 2. Header (Plaintext)

The header is unencrypted and always appears first.

```
MAGIC_1 (u32)               = 0x9AA2D903
MAGIC_2 (u32)               = 0xB54BFB66

VERSION (u32)               = (minor << 16 | major)
ENCRYPTION FLAGS (u32)
VERSION2 (u32)              = typically 0

MASTER SEED (16 bytes)
ENCRYPTION IV (16 bytes)

TRANSFORM SEED (32 bytes)
TRANSFORM ROUNDS (u32)

STREAM START BYTES (32 bytes)

GROUP COUNT (u32)           = number of groups N
ENTRY COUNT (u32)           = number of entries M
```

These counts determine how many groups and entries you must parse later.

---

## 3. Encrypted Payload (AES-256-CBC)

The payload is encrypted using:

```
AES_KEY = SHA256( master_seed || transformed_user_key )
```

Where `transformed_user_key` is produced by repeatedly encrypting the user key using AES-ECB for `TRANSFORM_ROUNDS`.

After decryption, the payload consists of:

```
[Group 1]
[Group 2]
...
[Group N]

[Entry 1]
[Entry 2]
...
[Entry M]

[Final End Marker]
```

---

## 4. TLV (Type-Length-Value) Encoding

Groups and entries share the same TLV encoding:

```
field_type : u16 (LE)
field_size : u32 (LE)
field_data : [field_size] bytes
```

Strings include a **null terminator** inside the field data.

Every group and entry ends with:

```
Type = 0xFFFF
Size = 0x00000000
```

---

## 5. Group Table (N Groups)

Parse **GroupCount** groups in order.

### Group Field Types

```
0x0001 → Group ID (u32)
0x0002 → Group Name (UTF-8, null-terminated)
0x0003 → Creation Time (time_t)
0x0004 → Last Modification Time (time_t)
0x0005 → Icon ID (u32)
0x0006 → Level (u16)
0xFFFF → End of Group
```

**Level** defines how groups are nested (indentation), not parent pointers.

---

## 6. Entry Table (M Entries)

After all groups are parsed, read **EntryCount** entries.

### Entry Field Types

```
0x0001 → UUID (16 bytes)
0x0002 → Group ID (u32)
0x0003 → Title (UTF-8, null-terminated)
0x0004 → Username (UTF-8)
0x0005 → Password (UTF-8)
0x0006 → URL (UTF-8)
0x0007 → Notes (UTF-8)

0x0008 → Creation Time (time_t)
0x0009 → Last Modification Time (time_t)
0x000A → Last Access Time (time_t)
0x000B → Expiration Time (time_t)
0x000C → Byte Description (UTF-8)
0x000D → Byte Data (raw bytes)

0xFFFF → End of Entry
```

Entries point to groups using `Group ID`.

---

## 7. End-of-Database Marker

After the last entry:

```
Type = 0xFFFF
Size = 0x00000000
```

This marker is required.

---

## 8. Parsing Procedure Summary

```
read header

for each group in 0..GroupCount:
    read TLV fields until type == 0xFFFF

for each entry in 0..EntryCount:
    read TLV fields until type == 0xFFFF

read final 0xFFFF database terminator
```

---

## 9. Notes for Implementors

- Group and entry TLV field types overlap; **context** (group section vs entry section) determines meaning.
- All timestamps are stored as 32-bit Unix `time_t`.
- Strings are null-terminated inside the TLV binary.
- Adding or removing groups/entries requires rebuilding the entire payload; KDB is not a random-access format.

---

This reference is complete and suitable for building a KDB parser/writer from scratch.
