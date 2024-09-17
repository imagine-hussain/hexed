# hexed

**Hexed.rs** is a high performance, native hex editor written in rust.

It's optimised to provide a live view on large (10GB+) files, holding 60fps and
avoiding a large memory footprint.


## Features
- **Hex Viewer:** Displays file content in hex format alongside an ASCII representation.
- **Optimized for Large Files:** Uses efficient file handling techniques to load large files in chunks without consuming excessive memory.
- **File Watcher:** Monitors changes to the active file and updates the display accordingly.
- **Cross-platform:** Runs on Linux, Windows, and macOS.

## Installation

### Prerequisites
- Rust (stable version recommended). You can install Rust via [rustup](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Steps
1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/hexed-rs.git
   cd hexed-rs
   ```

2. You have a few choices for running / installation.

**Recommended:**
You can install using `cargo`:
   ```bash
   cargo install --path .
   ```

You can also build manually, and then run the binary directly, or add it to your path
   ```bash
   cargo build --release
   ./target/release/hexed
   ```

You can run the program without installing, using `cargo run`

   ```bash
   cargo run --release
   ```


## Running the Application

To run Hexed.rs, simply use the following command:

```bash
hexed --filename <path-to-file>
```

Note that you can omit the `filename` and use the file-picker to select the file
at runtime.

### Examples:

Base Hex View:
![image](https://github.com/user-attachments/assets/90d60219-7c04-4814-99e8-cd46844b9265)

File Picker
![image](https://github.com/user-attachments/assets/0e99b297-1b1a-4d85-8fd9-7660c8171e7b)



### To Do
- Read variable length data types
- Custom DSL for parsing your own data types
- Add a demo of the hot-reloading

