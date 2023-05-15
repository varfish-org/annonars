[![Crates.io](https://img.shields.io/crates/d/annonars.svg)](https://crates.io/crates/annonars)
[![Crates.io](https://img.shields.io/crates/v/annonars.svg)](https://crates.io/crates/annonars)
[![Crates.io](https://img.shields.io/crates/l/annonars.svg)](https://crates.io/crates/annonars)
[![CI](https://github.com/bihealth/annona-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/bihealth/annona-rs/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/bihealth/annona-rs/branch/main/graph/badge.svg?token=UjTNKN6kCI)](https://codecov.io/gh/bihealth/annona-rs)

<a href="https://commons.wikimedia.org/wiki/File:Annona_squamosa_Blanco1.192.png"><img src="utils/img/annona-wikimedia.jpg" width="200px" height="321px" align="right"></a>

# annona-rs

Genome annotation with Rust and RocksDB.

Also:

> Annona (from Taíno annon) is a genus of flowering plants in the pawpaw/sugar apple family, Annonaceae. It is the second largest genus in the family after Guatteria, containing approximately 166 species of mostly Neotropical and Afrotropical trees and shrubs.
>
> [Annona -- Wikipedia](https://en.wikipedia.org/wiki/Annona)

## Running the CLI

You can enable the annonars CLI by building the project with the `cli` feature:

```
# cargo run --all-features cli -- --help
```

## Managing GitHub Project with Terraform

```
# export GITHUB_OWNER=bihealth
# export GITHUB_TOKEN=ghp_<thetoken>
# terraform import github_repository.annona-rs annona-rs

# cd utils/terraform
# terraform validate
# terraform fmt
# terraform plan
# terraform apply
```
