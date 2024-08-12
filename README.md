<div align="center">

# salvo-captcha

A captcha middleware for [salvo](salvo.rs) framework. With fully customizable captchas generator, storage, and finders

[![salvo-captcha-video](https://i.suar.me/9NjJ1)](https://ibb.co/XVRVMZj)

</div>

## Add to your project

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
salvo-captcha = "0.2"
```

Or use [`cargo add`] to add the dependency to your `Cargo.toml`:

```sh
$ cargo add salvo-captcha
```

## Usage

See the [examples] directory for a complete example. You can also see the implemented generators, storages, and finders in the source code.

## Storage

There is a default storage, which is [`MemoryStorage`] it's a simple in-memory storage. You can implement your own storage by implementing the [`CaptchaStorage`] trait.

### Cacache Storage

A high-performance, concurrent, content-addressable disk cache. The storage is based on [`cacache-rs`] crate. to use it, you need to enable the `cacache-storage` feature.

```toml
[dependencies]
salvo-captcha = { version = "0.2", features = ["cacache-storage"] }
```

## Captcha Finder

We provide fully customizable query parameters, form fields, and headers to find the captcha token and the captcha answer. You can implement your own finder by implementing the [`CaptchaFinder`] trait.

## Captcha Generator

We provide [`SimpleCaptchaGenerator`] which is a simple captcha generator based on the [`captcha`] crate. You can implement your own captcha generator by implementing the [`CaptchaGenerator`] trait.

### Captcha name and difficulty

In this table, you can see the difference between the difficulties and the name of the captcha.

|      Name       |                 Easy                 |                Medium                |                 Hard                 |
| :-------------: | :----------------------------------: | :----------------------------------: | :----------------------------------: |
|     Normal      | ![Simple](https://i.suar.me/edwBG/s) | ![Simple](https://i.suar.me/NJmg0/s) | ![Simple](https://i.suar.me/OJK7M/s) |
| SlightlyTwisted | ![Simple](https://i.suar.me/1JaxG/s) | ![Simple](https://i.suar.me/l7zBl/s) | ![Simple](https://i.suar.me/qXAlx/s) |
|   VeryTwisted   | ![Simple](https://i.suar.me/dO78z/s) | ![Simple](https://i.suar.me/PXBwK/s) | ![Simple](https://i.suar.me/8edgE/s) |

## Mirrors

- Github (<https://github.com/TheAwiteb/salvo-captcha>)
- Codeberg (<https://codeberg.org/awiteb/salvo-captcha>)

### Main Repository

- My Git (<https://git.4rs.nl/awiteb/salvo-captcha>)

## License

This project is licensed under the MIT license for more details see [LICENSE] or <http://opensource.org/licenses/MIT>.

[`MemoryStorage`]: https://docs.rs/salvo-captcha/latest/salvo_captcha/struct.MemoryStorage.html
[`CaptchaStorage`]: https://docs.rs/salvo-captcha/latest/salvo_captcha/trait.CaptchaStorage.html
[`cacache-rs`]: https://github.com/zkat/cacache-rs
[`SimpleCaptchaGenerator`]: https://docs.rs/salvo-captcha/latest/salvo_captcha/struct.SimpleCaptchaGenerator.html
[`CaptchaGenerator`]: https://docs.rs/salvo-captcha/latest/salvo_captcha/trait.CaptchaGenerator.html
[`CaptchaFinder`]: https://docs.rs/salvo-captcha/latest/salvo_captcha/trait.CaptchaFinder.html
[examples]: https://git.4rs.nl/awiteb/salvo-captcha/src/branch/master/examples
[`captcha`]: https://github.com/daniel-e/captcha
[LICENSE]: https://git.4rs.nl/awiteb/salvo-captcha/src/branch/master/LICENSE
[`cargo add`]: https://doc.rust-lang.org/cargo/commands/cargo-add.html