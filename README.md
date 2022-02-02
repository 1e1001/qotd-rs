# qotd-rs
Implementation of a [Quote of the Day](https://en.wikipedia.org/wiki/QOTD) server in Rust. Supports reading quotes from either a text file, or the output of a program.
```sh
Usage: qotd-rs [provider] [...args]
Where provider is one of: file, cmd
```

Make sure to provide the whole path to the program when using the `cmd` provider.