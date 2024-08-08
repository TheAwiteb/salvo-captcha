<div align="center">

# salvo-captcha

A captcha middleware for [salvo](salvo.rs) framework. It uses [`captcha`](https://github.com/daniel-e/captcha) crate to generate captcha images.

[![salvo-captcha-video](https://i.suar.me/9NjJ1)](https://ibb.co/XVRVMZj)

</div>

## Add to your project

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
salvo-captcha = "0.1"
```

Or use [cargo-add](https://doc.rust-lang.org/cargo/commands/cargo-add.html) to add the dependency to your `Cargo.toml`:

```sh
$ cargo add salvo-captcha
```

## Usage

See the [examples](examples) directory for a complete example.

## Storage

The storage of the captcha is handled by a [`CaptchaStore`] trait. You can implement your own storage or use the [`cacache-rs`] by enabling the `cacache-storage` feature.

```toml
[dependencies]
salvo-captcha = { version = "0.1", features = ["cacache-storage"] }
```

## Captcha name and difficulty

In this table, you can see the difference between the difficulties and the name of the captcha.

|  Name  |                 Easy                 |                Medium                |                 Hard                 |
| :----: | :----------------------------------: | :----------------------------------: | :----------------------------------: |
| Amelia | ![Simple](https://i.suar.me/1JaxG/s) | ![Simple](https://i.suar.me/l7zBl/s) | ![Simple](https://i.suar.me/qXAlx/s) |
|  Lucy  | ![Simple](https://i.suar.me/edwBG/s) | ![Simple](https://i.suar.me/NJmg0/s) | ![Simple](https://i.suar.me/OJK7M/s) |
|  Mila  | ![Simple](https://i.suar.me/dO78z/s) | ![Simple](https://i.suar.me/PXBwK/s) | ![Simple](https://i.suar.me/8edgE/s) |

## Mirrors

- Github (https://github.com/TheAwiteb/salvo-captcha)
- Codeberg (https://codeberg.org/awiteb/salvo-captcha)

### Main Repository

- My Git (https://git.4rs.nl/awiteb/salvo-captcha)

## License

This project is licensed under the MIT license for more details see [LICENSE](LICENSE) or http://opensource.org/licenses/MIT.

[`CaptchaStore`]: https://docs.rs/salvo_captcha/0.1.0/salvo_captcha/trait.CaptchaStore.html
[`cacache-rs`]: https://github.com/zkat/cacache-rs
