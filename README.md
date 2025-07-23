# frosted

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-blue?logo=rust)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

**frosted** is a blazing-fast, Rust-powered replacement for [freezed](https://pub.dev/packages/freezed), designed specifically to address the performance issues of freezed in large projects. Unlike freezed, which is notoriously slow because it reprocesses everything on each run, frosted watches your files and only generates code where and when it's actually needed. This makes it especially well-suited for large codebases, where incremental, targeted code generation is essential for developer productivity.

frosted also generates the `copyWith` extension for your classes, making it easy to work with immutable data structures in Dart.

---

## ✨ Features

- **Replacement for freezed**: Designed to be a drop-in alternative, but much faster and more efficient.
- **Automatic file watching**: Monitors your project directory for changes and triggers code generation instantly.
- **Magic token parsing**: Only processes files containing a configurable magic token (default: `+mk:`).
- **Customizable output**: Generates files with a configurable extension (default: `.copy.gen.dart`).
- **Configurable watched file extensions**: Only watches and processes files with specified extensions (default: `dart`).
- **Generates `copyWith` extension**: Automatically creates `copyWith` methods for your classes.
- **Fast and lightweight**: Built in Rust for speed and reliability.

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or newer)

### Installation

Clone the repository and build the binary:

```sh
git clone https://github.com/yourusername/frosted.git
cd frosted
cargo build --release
```

The binary will be located at `target/release/frosted`.

---

## ⚡ Usage

Run frosted in your project directory:

```sh
./target/release/frosted --directory ./test
```

### Options

| Flag         | Description                                      | Default            |
|--------------|--------------------------------------------------|--------------------|
| `-d`, `--directory` | Directory to watch for changes                | (required)         |
| `-m`, `--magic-token` | Magic token to identify files to parse        | `+mk:`             |
| `-o`, `--file-extension` | Extension for generated files               | `copy.gen.dart`    |
| `-e`, `--file-extensions` | File extensions to watch (comma-separated)  | `dart`             |

#### Examples

Watch only Dart files (default):

```sh
./target/release/frosted -d ./test
```

Watch multiple file types (e.g., Dart and model files):

```sh
./target/release/frosted -d ./test -e dart,model
```

---

## 🛠️ How It Works

1. **Watches** the specified directory (recursively) for file changes.
2. **Filters** events to only files with the specified extensions.
3. **Parses** files containing the magic token.
4. **Generates** Dart code using a Handlebars template, outputting to a file with the specified extension.
5. **Automatically generates `copyWith` methods** for your classes.
6. **Ignores** files that are already generated (matching the output extension).

---

## 📝 Example

Suppose you have a Dart file with the magic token:

```dart
// +mk:
class Example {
  final int value;
}
```

When you save this file, frosted will generate a corresponding `copy.gen.dart` file in the same directory, including a `copyWith` extension for your class.

---

## 📦 Dependencies

- [notify](https://crates.io/crates/notify) (file watching)
- [clap](https://crates.io/crates/clap) (CLI parsing)
- [anyhow](https://crates.io/crates/anyhow) (error handling)

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for details.

---

## 🙏 Acknowledgements

- Inspired by [freezed](https://pub.dev/packages/freezed) and the Dart code generation ecosystem.
