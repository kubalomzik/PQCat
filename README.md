# PQCat

PQCat (Post-Quantum Cryptography Classical Attack Tool) is a Rust-based framework for implementing and benchmarking classical attacks on post-quantum cryptography schemes. Currently it provides implementations of algorithms targeting code-based schemes:

- **Prange's Algorithm**
- **Stern's Algorithm**
- **Lee-Brickell Algorithm**
- **Ball Collision Decoding**
- **May-Meurer-Thomae (MMT) Algorithm**

The tool allows users to experiment with parameters, visualize results, and analyze performance characteristics such as execution time and decoding success.

## Setup

1. Make sure you have Rust installed.
2. Build the project.

   ```cargo build```

3. Run the tool

   - You can execute the binary directly:

   ```./target/release/pqcat```

   - Or use `cargo run` during development:

   ```cargo run [algorithm] [optional: parameters]```

   - Consider installing the tool to use it globally:

   ```cargo install --path .```

   - Then you can run it from anywhere with:

   ```pqcat [algorithm]```

    For example:

   ```pqcat stern --n 6 --k 3 --w 2```
