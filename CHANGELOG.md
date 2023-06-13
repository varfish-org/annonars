# Changelog

### [0.10.1](https://www.github.com/bihealth/annona-rs/compare/v0.10.0...v0.10.1) (2023-06-13)


### Bug Fixes

* remove unused dependencies ([#81](https://www.github.com/bihealth/annona-rs/issues/81)) ([5f861c4](https://www.github.com/bihealth/annona-rs/commit/5f861c4a654614ae7861e12cc83bad30f5902ac0))

## [0.10.0](https://www.github.com/bihealth/annona-rs/compare/v0.9.0...v0.10.0) (2023-06-12)


### Features

* thread safety in hgvs dependency ([#78](https://www.github.com/bihealth/annona-rs/issues/78)) ([e642397](https://www.github.com/bihealth/annona-rs/commit/e642397bad4a88702ed146c3d7027f3d6c81df9a))

## [0.9.0](https://www.github.com/bihealth/annona-rs/compare/v0.8.0...v0.9.0) (2023-06-09)


### Code Refactoring

* replace rocks_utils by rocksdb-utils-lookup crate ([#76](https://www.github.com/bihealth/annona-rs/issues/76)) ([52ccb96](https://www.github.com/bihealth/annona-rs/commit/52ccb96cc766ac2d3fb32eea0b98dcce781cfc91))

## [0.8.0](https://www.github.com/bihealth/annona-rs/compare/v0.7.0...v0.8.0) (2023-06-08)


### Features

* port over clinvar-minimal from mehari ([#73](https://www.github.com/bihealth/annona-rs/issues/73)) ([#74](https://www.github.com/bihealth/annona-rs/issues/74)) ([5720ff3](https://www.github.com/bihealth/annona-rs/commit/5720ff378cc7257d641f8afe183cf46d31b0ad6a))


### Bug Fixes

* various import issues occurring with varfish-db-downloader ([#71](https://www.github.com/bihealth/annona-rs/issues/71)) ([52296f9](https://www.github.com/bihealth/annona-rs/commit/52296f99a2e91bf05f64dab32dc762a4cc09cf93))

## [0.7.0](https://www.github.com/bihealth/annona-rs/compare/v0.6.0...v0.7.0) (2023-06-06)


### Features

* port over mehari freq counts code ([#67](https://www.github.com/bihealth/annona-rs/issues/67)) ([a99a9bb](https://www.github.com/bihealth/annona-rs/commit/a99a9bbaa31e764e456156d03789c8efdec552ab))

## [0.6.0](https://www.github.com/bihealth/annona-rs/compare/v0.5.1...v0.6.0) (2023-06-01)


### Features

* adding "db-utils dump-meta" command ([#56](https://www.github.com/bihealth/annona-rs/issues/56)) ([#60](https://www.github.com/bihealth/annona-rs/issues/60)) ([92f30c2](https://www.github.com/bihealth/annona-rs/commit/92f30c20b0ed25bf1e2694e25a35bde109f2ed39))
* storing TSV lines as string to reduce storage size ([#57](https://www.github.com/bihealth/annona-rs/issues/57)) ([#58](https://www.github.com/bihealth/annona-rs/issues/58)) ([3a77eb6](https://www.github.com/bihealth/annona-rs/commit/3a77eb615d5805062c5cd0595277c4d950fea92d))

### [0.5.1](https://www.github.com/bihealth/annona-rs/compare/v0.5.0...v0.5.1) (2023-05-24)


### Bug Fixes

* writing gnomad-version meta info for gnomad-mtdna ([#54](https://www.github.com/bihealth/annona-rs/issues/54)) ([a051d7e](https://www.github.com/bihealth/annona-rs/commit/a051d7e8add800d44a658c29ec5a7a31a8624e7a))

## [0.5.0](https://www.github.com/bihealth/annona-rs/compare/v0.4.0...v0.5.0) (2023-05-24)


### Features

* parallelize "db-utils copy" for BED files ([#52](https://www.github.com/bihealth/annona-rs/issues/52)) ([e061410](https://www.github.com/bihealth/annona-rs/commit/e0614106b40fc597d0730d99b0d3cb83a4b8c965))

## [0.4.0](https://www.github.com/bihealth/annona-rs/compare/v0.3.0...v0.4.0) (2023-05-24)


### ⚠ BREAKING CHANGES

* store list of ucsc conservation records (#48) (#49)

### Bug Fixes

* store list of ucsc conservation records ([#48](https://www.github.com/bihealth/annona-rs/issues/48)) ([#49](https://www.github.com/bihealth/annona-rs/issues/49)) ([813de6f](https://www.github.com/bihealth/annona-rs/commit/813de6f26feec8105c8c9570451d7909085d70dd))


### Miscellaneous Chores

* adjusting release version ([02e7ffe](https://www.github.com/bihealth/annona-rs/commit/02e7ffe21f0aae18a472844acace3389e271c0b3))

## [0.3.0](https://www.github.com/bihealth/annona-rs/compare/v0.2.4...v0.3.0) (2023-05-23)


### Features

* reducing window size to 100k to make par_tbi faster ([#46](https://www.github.com/bihealth/annona-rs/issues/46)) ([e69257e](https://www.github.com/bihealth/annona-rs/commit/e69257e6c59e81f0d1e29026777679bc4bcdab1e))

### [0.2.4](https://www.github.com/bihealth/annona-rs/compare/v0.2.3...v0.2.4) (2023-05-23)


### Bug Fixes

* losening dependencies ([#44](https://www.github.com/bihealth/annona-rs/issues/44)) ([bf22efd](https://www.github.com/bihealth/annona-rs/commit/bf22efdfa62c61770726a75a8b856869943f7115))

### [0.2.3](https://www.github.com/bihealth/annona-rs/compare/v0.2.2...v0.2.3) (2023-05-23)


### Bug Fixes

* tsv parsing index problems ([#41](https://www.github.com/bihealth/annona-rs/issues/41)) ([ff14b54](https://www.github.com/bihealth/annona-rs/commit/ff14b5433d4f789125c2b9fe8079824734ade9aa))

### [0.2.2](https://www.github.com/bihealth/annona-rs/compare/v0.2.1...v0.2.2) (2023-05-23)


### Bug Fixes

* allow "db-utils copy" without genome-release meta entry ([#39](https://www.github.com/bihealth/annona-rs/issues/39)) ([773896e](https://www.github.com/bihealth/annona-rs/commit/773896e61751215b7b67c214f15751f0b76d3b04))

### [0.2.1](https://www.github.com/bihealth/annona-rs/compare/v0.2.0...v0.2.1) (2023-05-23)


### Bug Fixes

* "db-utils copy" now accepts "--all" and "--path-beds" ([#37](https://www.github.com/bihealth/annona-rs/issues/37)) ([0b50060](https://www.github.com/bihealth/annona-rs/commit/0b5006003dd5a0b28c5730b17e5ea40558bbda82))

## [0.2.0](https://www.github.com/bihealth/annona-rs/compare/v0.1.0...v0.2.0) (2023-05-23)


### Features

* add "db-utils copy" command ([#30](https://www.github.com/bihealth/annona-rs/issues/30)) ([#31](https://www.github.com/bihealth/annona-rs/issues/31)) ([f918a27](https://www.github.com/bihealth/annona-rs/commit/f918a275e80d9c6a18a464d79346d5430248c3d5))
* implement import and query for gnomAD-mtDNA ([#16](https://www.github.com/bihealth/annona-rs/issues/16)) ([#24](https://www.github.com/bihealth/annona-rs/issues/24)) ([95ea15d](https://www.github.com/bihealth/annona-rs/commit/95ea15d44856c19414e2bbdb3b19473b842ca18f))
* implement import and query for UCSC conservation ([#11](https://www.github.com/bihealth/annona-rs/issues/11)) ([#14](https://www.github.com/bihealth/annona-rs/issues/14)) ([3fc2f25](https://www.github.com/bihealth/annona-rs/commit/3fc2f257901055e86dc66b8cd3519e7215c55afd))
* implement import/query of dbsnp ([#17](https://www.github.com/bihealth/annona-rs/issues/17)) ([#21](https://www.github.com/bihealth/annona-rs/issues/21)) ([b027382](https://www.github.com/bihealth/annona-rs/commit/b027382e65ab92eb7b5bdc44be0c219b08aa9976))
* import and query for gnomAD {ex,gen}omes ([#18](https://www.github.com/bihealth/annona-rs/issues/18)) ([#25](https://www.github.com/bihealth/annona-rs/issues/25)) ([0e63d12](https://www.github.com/bihealth/annona-rs/commit/0e63d123fb9efdf8067ab27d63b53f9e694849c8))
* import and query for HelixMtDb VCF ([#15](https://www.github.com/bihealth/annona-rs/issues/15)) ([#23](https://www.github.com/bihealth/annona-rs/issues/23)) ([9dfa520](https://www.github.com/bihealth/annona-rs/commit/9dfa52027e37c548a7945580995bdac03c6a0f47))
* use explicit indicatif progress bars ([#32](https://www.github.com/bihealth/annona-rs/issues/32)) ([#33](https://www.github.com/bihealth/annona-rs/issues/33)) ([2ceb2c6](https://www.github.com/bihealth/annona-rs/commit/2ceb2c6ed9584d314504438a49b6d60013fb5390))

## 0.1.0 (2023-05-16)


### Features

* import of TSV files ([#1](https://www.github.com/bihealth/annona-rs/issues/1)) ([#4](https://www.github.com/bihealth/annona-rs/issues/4)) ([e0a2402](https://www.github.com/bihealth/annona-rs/commit/e0a24029872af214ca0b2d636a7dbf677deac2fc))
* querying of TSV files via CLI ([#2](https://www.github.com/bihealth/annona-rs/issues/2)) ([#7](https://www.github.com/bihealth/annona-rs/issues/7)) ([ceb908d](https://www.github.com/bihealth/annona-rs/commit/ceb908d893e4e2f570409911d5c794f99bbaa87b))
