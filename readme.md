# Language Security Officer

the MySQL sql inspector and checker for security and performance.

**THE PROJECT IS STILL UNDER DEVELOPMENT AND UNSTABLE, PLEASE USE IT PRUDENTLY. AND OF COURSE THE API OF IT MAY BE CHANGED**

## Installation
```sh
$ cargo add lso
```


## Example

```shell
# setup mysql dsn
export SQL_DSN=mysql:root@local:database

# launching the officer
lso 
```


## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:
- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]


## License
This project is licensed under either of the following licenses, at your option:
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0])
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT])


[contributing]: https://github.com/kilerd/lso/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/kilerd/lso/labels/good%20first%20issue
[help-wanted]: https://github.com/kilerd/lso/labels/help%20wanted