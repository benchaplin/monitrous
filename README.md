# monitrous

Monitor a list of URLs by capturing screenshots and comparing them.

## Build from source

Prerequisites:

- Install Rust and Cargo (https://doc.rust-lang.org/cargo/getting-started/installation.html)

Clone the repo:

```bash
$ git clone https://github.com/benchaplin/monitrous.git
```

Build the binary:

```bash
$ cd monitrous
$ cargo build -r
```

The `monitrous` binary is now ready for use in the directory: `monitrous/target/release`.

## Usage

### `capture`

Capture screenshots of a list of URLs.

To start, create a text file of URLs to monitor, separated by line. For example:

**urls.txt**

```
https://reddit.com/r/news
https://reddit.com/r/gifs
```

Pass the following args to screenshot the URLs listed in `urls.txt`, and put the resulting PNG files in the directory `my-screenshots`.

```bash
$ monitrous capture urls.txt my-screenshots
```

### `compare`

Compare the screenshots in one directory to another (will compare by filename).

Pass the following args to compare the screenshots in `2022-11-27` to the screenshots in `2022-11-28`.

```bash
$ monitrous compare 2022-11-27 2022-11-28
```
