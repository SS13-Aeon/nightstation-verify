# Nightstation Verify Bot

Discord bot to handle verification for SS13 servers

## Usage

The easiest way to start is to extract the program into a directory and run the
executable with no arguments. You will be guided through the bot creation
process and all of the configuration options. Once finished, the bot will
automatically start running. Any future runs will reuse your configuration.

```txt
nightstation-verify [OPTIONS]

Options:
  -c, --config <CONFIG>  Config file [default: config.ron]
  -d, --data <DATA>      Data directory [default: data]
  -n, --no-run           Don't run after loading or creating config
  -C, --copyright        Print copyright notice
  -S, --source           Print source code link
  -v, --verbose...       Increase verbosity level
  -h, --help             Print help
  -V, --version          Print version
```

## Development

This project uses [Cargo](https://doc.rust-lang.org/cargo/) as a build system.

You can run the bot with `cargo run` and make deployment builds with
`cargo build --release --target <TARGET>`.

## License

```txt
Nightstation verification bot
Copyright (C) 2023  Nightstation contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
