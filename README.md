# sbt-client

A thin client for [sbt](https://www.scala-sbt.org/).

Inspired by [Eugene's PR](https://github.com/sbt/sbt/pull/4227), I decided to
write a thin client without the JVM startup overhead.

## How to install

There are a couple of ways to install `sbt-client`:

1. If you are on a Mac and you trust me enough to download a random binary file
   when I tell you to, you can [download the binary from the GitHub
   release](https://github.com/cb372/sbt-client/releases/download/v0.1.0/sbt-client).

2. If you don't mind installing Rust, you can download the latest version by
   installing from source.

### Installing from source

1. [Install Rust](https://www.rust-lang.org/en-US/install.html) (on a Mac this
   is simply `brew install rust`)

2. Check that `~/.cargo/bin` is on your `$PATH`.

3. Clone this repo.

4. Run `cargo install`. This will create `~/.cargo/bin/sbt-client`.

## How to use

In the root directory of an sbt project, run `sbt-client <some sbt command>.

e.g. `sbt-client clean`.

If sbt is not running, it will automatically start it for you and keep it
running in the background. So the next time you run `sbt-client` it will be much
more snappy.

## Examples

`sbt-client clean` starting an sbt server:

![Starting an sbt server](doc/images/starting-sbt-server.png)

`sbt-client clean` again, now that the server is running:

![sbt-client clean](doc/images/clean.png)

`sbt-client compile` displaying compilation errors:

![Compilation errors](doc/images/compilation-errors.png)

## Performance

```
$ time sbt-client clean
[info] Processing
[success] Done
sbt-client clean  0.00s user 0.00s system 5% cpu 0.075 total
```

## Compatibility

* Developed and tested on MacOS
* Should also work on Linux
* Will NOT work on Windows
