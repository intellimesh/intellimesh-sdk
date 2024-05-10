<!-- markdown-link-check-disable -->
# cortex-gaurdian
**Intellimesh Offchain Worker that validates work submitted by Cognito workers ( video analytics )**

## Code walkthrough
The pallet includes use of off-chain workers, cryptographic functions, and transaction management. Let's break down the key components and concepts:

### 1. **Conditional Compilation Directive**
   - `#![cfg_attr(not(feature = "std"), no_std)]`: This line tells Rust to compile this module without the standard library (`std`) if the `std` feature is not enabled. This is typical in Substrate runtime development to ensure compatibility with the WebAssembly environment.

### 2. **Imports**
   - **`codec::{Decode, Encode}`**: Traits for encoding and decoding data structures to and from binary formats, essential for blockchain operations.
   - **`frame_support` and `frame_system`**: Part of Substrate's framework providing support for various blockchain functionalities like transactions, metadata, etc.
   - **`lite_json::json::JsonValue`**: A JSON parsing library used likely for handling JSON data in off-chain workers.
   - **`sp_core`, `sp_runtime`**: Foundational libraries in Substrate providing core functionalities like cryptographic operations and runtime definitions.
   - **`sp_std::vec::Vec`**: Enables use of the `Vec` data structure, similar to Rust's standard `Vec`.

### 3. **Off-chain Workers**
   - Allow blockchains to perform operations outside of the blockchain itself (off-chain), such as fetching data from external sources or heavy computations. The code imports several utilities for this purpose:
     - **Traits and Structs for Transactions**: `AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer, SigningTypes, SubmitTransaction`
     - **HTTP and Storage**: `http, StorageValueRef`

### 4. **Cryptographic and Transaction Elements**
   - **`KeyTypeId`**: Identifies specific cryptographic keys in the system.
   - **Transaction Validity**: `TransactionValidity, InvalidTransaction, ValidTransaction` are used to define and handle the validity of transactions within the blockchain.

### 5. **Runtime and Storage Errors**
   - **`MutateStorageError, StorageRetrievalError`**: Errors related to mutating and retrieving data from storage, respectively.
   - **`Duration`**: Handles time durations, critical in timing-related operations like timeouts.

### 6. **Traits and Miscellaneous**
   - **`Zero`**: A trait for types that have a concept of "zero", useful in scenarios where default or initial values are required.
   - **`RuntimeDebug`**: Enables debugging capabilities in a Substrate runtime environment, often stripped down for production builds.

This module is quite complex and is tailored for blockchain development where off-chain computation, cryptographic security, and efficient data handling are critical.
