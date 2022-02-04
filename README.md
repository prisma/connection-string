<h1 align="center">connection-string</h1>
<div align="center">
  <strong>
    Connection string parsing in Rust
  </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/connection-string">
    <img src="https://img.shields.io/crates/v/connection-string.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/connection-string">
    <img src="https://img.shields.io/crates/d/connection-string.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/connection-string">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- npmjs.com version -->
  <a href="https://badge.fury.io/js/@pimeys%2Fconnection-string">
    <img src="https://badge.fury.io/js/@pimeys%2Fconnection-string.svg"
      alt="npm version" height="18">
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/connection-string">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/prisma/connection-string/releases">
      Releases
    </a>
    <span> | </span>
    <a href="https://github.com/prisma/connection-string/blob/main.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

## Installation for Rust
```sh
$ cargo add connection-string
```

## Usage for JavaScript
The crate is available in npm as `@pimeys/connection-string`. Usage patters try
to follow the Rust version as close as possible. Please see the [Rust
docs](https://docs.rs/connection-string) for more information.

JDBC:

``` javascript
const j = new JdbcString("jdbc:sqlserver://localhost\\INSTANCE:1433;database=master;user=SA;password={my_password;123}");

console.log(j.server_name()); // "localhost"
console.log(j.port()); // 1433
console.log(j.instance_name()); // "INSTANCE"
console.log(j.get("database")); // "master"
console.log(j.get("password")); // "my_password;123" (see escaping)
console.log(j.keys()); // ["database", "user", "password"]
console.log(j.set("password", "a;;new;;password")); // "my_password;123" (returns the old value, if available)

// "jdbc:sqlserver://localhost\INSTANCE:1433;user=SA;database=master;password=a{;;}new{;;}password"
console.log(j.to_string())
```

ADO.net:

``` javascript
const a = new AdoNetString("server=tcp:localhost,1433;user=SA;password=a{;;}new{;;}password");

console.log(a.get("password")); // a;;new;;password
console.log(a.set("user", "john")); // "SA" (returns the old value, if available)

// "server=tcp:localhost,1433;user=john;password=a{;;}new{;;}password"
console.log(j.to_string())
```

## Safety
This crate uses ``#![deny(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/prisma/connection-string/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/prisma/connection-string/labels/good%20first%20issue
[help-wanted]: https://github.com/prisma/connection-string/labels/help%20wanted

## Building

The build procedure and dependencies are defined in the provided
[flake.nix](flake.nix) file. Please install the unstable Nix with flakes support
([Linux](https://nixos.wiki/wiki/Nix_Installation_Guide), [macOS](https://gist.github.com/sagittaros/32dc6ffcbc423dc0fed7eef24698d5ca)).

The WASM module can be built with:

```bash
nix build
```

This creates a link `result` to the current directory, containing a NodeJS
package with the Rust code compiled as WASM bytecode.

## Testing

Run the tests with the nix subcommand:

```bash
nix run .#test
```

## Publishing

The `updatePackageVersion` command changes the package version to the Rust `Cargo.toml` and
JavaScript `package.json` at the same time:

```bash
nix run .#updatePackageVersion 0.1.14
```

Don't forget to add the tag before publishing the library:

```bash
git tag v0.1.14
```

The publishing can be done separately or together with the `publish` command:

```bash
nix run .#publishRust
nix run .#publishJavascript
```

or

```bash
nix run .#publish
```

Please be sure you have the corresponding publishing rights in crates and npmjs.

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
