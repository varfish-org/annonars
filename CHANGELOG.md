# Changelog

## [0.12.8](https://github.com/bihealth/annonars/compare/v0.12.7...v0.12.8) (2023-07-04)


### Bug Fixes

* properly configure dependabot for noodles ([#127](https://github.com/bihealth/annonars/issues/127)) ([656d297](https://github.com/bihealth/annonars/commit/656d297d5bc5675574d3daf7a4f9addec4d22233))

## [0.12.7](https://github.com/bihealth/annonars/compare/v0.12.6...v0.12.7) (2023-06-23)


### Bug Fixes

* "db-utils copy" for chr prefixes ([#105](https://github.com/bihealth/annonars/issues/105)) ([a8d9f00](https://github.com/bihealth/annonars/commit/a8d9f0031940b9c647f84dc7f34f91abadb6f96d))

## [0.12.6](https://github.com/bihealth/annonars/compare/v0.12.5...v0.12.6) (2023-06-22)


### Bug Fixes

* issue with "db-utils copy" on chrY ([#103](https://github.com/bihealth/annonars/issues/103)) ([93d08df](https://github.com/bihealth/annonars/commit/93d08dfd284201e7664463c4693500ef337a6663))

## [0.12.5](https://github.com/bihealth/annonars/compare/v0.12.4...v0.12.5) (2023-06-20)


### Bug Fixes

* add missing libsqlite3-0 to Docker image ([#100](https://github.com/bihealth/annonars/issues/100)) ([dcf0f3e](https://github.com/bihealth/annonars/commit/dcf0f3e9b4cf3a38374636c55e88304661617a8e))

## [0.12.4](https://github.com/bihealth/annonars/compare/v0.12.3...v0.12.4) (2023-06-19)


### Bug Fixes

* docker build version in CI ([#98](https://github.com/bihealth/annonars/issues/98)) ([93f0707](https://github.com/bihealth/annonars/commit/93f07075c4cea1361541525c9d47f5ddd4fd173a))

## [0.12.3](https://github.com/bihealth/annonars/compare/v0.12.2...v0.12.3) (2023-06-19)


### Build System

* some small fixes to CI ([#96](https://github.com/bihealth/annonars/issues/96)) ([b72d249](https://github.com/bihealth/annonars/commit/b72d24902a82dbe73ab828ceef8a67dd07a2b0f2))

## [0.12.2](https://github.com/bihealth/annonars/compare/v0.12.1...v0.12.2) (2023-06-19)


### Build System

* fix docker builds ([#93](https://github.com/bihealth/annonars/issues/93)) ([225be0b](https://github.com/bihealth/annonars/commit/225be0b09d4f2fe87b1f02c1f9a82af45fa295de))

## [0.12.2](https://github.com/bihealth/annonars/compare/v0.12.1...v0.12.2) (2023-06-19)


### Build System

* fix docker builds ([#93](https://github.com/bihealth/annonars/issues/93)) ([3cf065f](https://github.com/bihealth/annonars/commit/3cf065facfed5a19e00a554c3dd2ac88e8d2bd02))

### [0.12.1](https://www.github.com/bihealth/annonars/compare/v0.12.0...v0.12.1) (2023-06-17)


### Build System

* adjust Docker builds for PRs and branches ([#91](https://www.github.com/bihealth/annonars/issues/91)) ([0a84014](https://www.github.com/bihealth/annonars/commit/0a84014a3bb08ef2f6b2b569bdd8994b63f7bb51))

## [0.12.0](https://www.github.com/bihealth/annonars/compare/v0.11.0...v0.12.0) (2023-06-16)


### Features

* port over genes db from worker ([#86](https://www.github.com/bihealth/annonars/issues/86)) ([#87](https://www.github.com/bihealth/annonars/issues/87)) ([608a36b](https://www.github.com/bihealth/annonars/commit/608a36bf7716ebe63f0a1624d7f9553403cef15d))

## [0.11.0](https://www.github.com/bihealth/annonars/compare/v0.10.0...v0.11.0) (2023-06-14)


### Features

* moved REST API server code from worker ([#80](https://www.github.com/bihealth/annonars/issues/80)) ([#83](https://www.github.com/bihealth/annonars/issues/83)) ([cd97a44](https://www.github.com/bihealth/annonars/commit/cd97a44035b1fed96152e4a8f080ccd6ce8e9446))


### Bug Fixes

* remove unused dependencies ([#81](https://www.github.com/bihealth/annonars/issues/81)) ([5f861c4](https://www.github.com/bihealth/annonars/commit/5f861c4a654614ae7861e12cc83bad30f5902ac0))

## [0.10.0](https://www.github.com/bihealth/annonars/compare/v0.9.0...v0.10.0) (2023-06-12)


### Features

* thread safety in hgvs dependency ([#78](https://www.github.com/bihealth/annonars/issues/78)) ([e642397](https://www.github.com/bihealth/annonars/commit/e642397bad4a88702ed146c3d7027f3d6c81df9a))

## [0.9.0](https://www.github.com/bihealth/annonars/compare/v0.8.0...v0.9.0) (2023-06-09)


### Code Refactoring

* replace rocks_utils by rocksdb-utils-lookup crate ([#76](https://www.github.com/bihealth/annonars/issues/76)) ([52ccb96](https://www.github.com/bihealth/annonars/commit/52ccb96cc766ac2d3fb32eea0b98dcce781cfc91))

## [0.8.0](https://www.github.com/bihealth/annonars/compare/v0.7.0...v0.8.0) (2023-06-08)


### Features

* port over clinvar-minimal from mehari ([#73](https://www.github.com/bihealth/annonars/issues/73)) ([#74](https://www.github.com/bihealth/annonars/issues/74)) ([5720ff3](https://www.github.com/bihealth/annonars/commit/5720ff378cc7257d641f8afe183cf46d31b0ad6a))


### Bug Fixes

* various import issues occurring with varfish-db-downloader ([#71](https://www.github.com/bihealth/annonars/issues/71)) ([52296f9](https://www.github.com/bihealth/annonars/commit/52296f99a2e91bf05f64dab32dc762a4cc09cf93))

## [0.7.0](https://www.github.com/bihealth/annonars/compare/v0.6.0...v0.7.0) (2023-06-06)


### Features

* port over mehari freq counts code ([#67](https://www.github.com/bihealth/annonars/issues/67)) ([a99a9bb](https://www.github.com/bihealth/annonars/commit/a99a9bbaa31e764e456156d03789c8efdec552ab))

## [0.6.0](https://www.github.com/bihealth/annonars/compare/v0.5.1...v0.6.0) (2023-06-01)


### Features

* adding "db-utils dump-meta" command ([#56](https://www.github.com/bihealth/annonars/issues/56)) ([#60](https://www.github.com/bihealth/annonars/issues/60)) ([92f30c2](https://www.github.com/bihealth/annonars/commit/92f30c20b0ed25bf1e2694e25a35bde109f2ed39))
* storing TSV lines as string to reduce storage size ([#57](https://www.github.com/bihealth/annonars/issues/57)) ([#58](https://www.github.com/bihealth/annonars/issues/58)) ([3a77eb6](https://www.github.com/bihealth/annonars/commit/3a77eb615d5805062c5cd0595277c4d950fea92d))

### [0.5.1](https://www.github.com/bihealth/annonars/compare/v0.5.0...v0.5.1) (2023-05-24)


### Bug Fixes

* writing gnomad-version meta info for gnomad-mtdna ([#54](https://www.github.com/bihealth/annonars/issues/54)) ([a051d7e](https://www.github.com/bihealth/annonars/commit/a051d7e8add800d44a658c29ec5a7a31a8624e7a))

## [0.5.0](https://www.github.com/bihealth/annonars/compare/v0.4.0...v0.5.0) (2023-05-24)


### Features

* parallelize "db-utils copy" for BED files ([#52](https://www.github.com/bihealth/annonars/issues/52)) ([e061410](https://www.github.com/bihealth/annonars/commit/e0614106b40fc597d0730d99b0d3cb83a4b8c965))

## [0.4.0](https://www.github.com/bihealth/annonars/compare/v0.3.0...v0.4.0) (2023-05-24)


### ⚠ BREAKING CHANGES

* store list of ucsc conservation records (#48) (#49)

### Bug Fixes

* store list of ucsc conservation records ([#48](https://www.github.com/bihealth/annonars/issues/48)) ([#49](https://www.github.com/bihealth/annonars/issues/49)) ([813de6f](https://www.github.com/bihealth/annonars/commit/813de6f26feec8105c8c9570451d7909085d70dd))


### Miscellaneous Chores

* adjusting release version ([02e7ffe](https://www.github.com/bihealth/annonars/commit/02e7ffe21f0aae18a472844acace3389e271c0b3))

## [0.3.0](https://www.github.com/bihealth/annonars/compare/v0.2.4...v0.3.0) (2023-05-23)


### Features

* reducing window size to 100k to make par_tbi faster ([#46](https://www.github.com/bihealth/annonars/issues/46)) ([e69257e](https://www.github.com/bihealth/annonars/commit/e69257e6c59e81f0d1e29026777679bc4bcdab1e))

### [0.2.4](https://www.github.com/bihealth/annonars/compare/v0.2.3...v0.2.4) (2023-05-23)


### Bug Fixes

* losening dependencies ([#44](https://www.github.com/bihealth/annonars/issues/44)) ([bf22efd](https://www.github.com/bihealth/annonars/commit/bf22efdfa62c61770726a75a8b856869943f7115))

### [0.2.3](https://www.github.com/bihealth/annonars/compare/v0.2.2...v0.2.3) (2023-05-23)


### Bug Fixes

* tsv parsing index problems ([#41](https://www.github.com/bihealth/annonars/issues/41)) ([ff14b54](https://www.github.com/bihealth/annonars/commit/ff14b5433d4f789125c2b9fe8079824734ade9aa))

### [0.2.2](https://www.github.com/bihealth/annonars/compare/v0.2.1...v0.2.2) (2023-05-23)


### Bug Fixes

* allow "db-utils copy" without genome-release meta entry ([#39](https://www.github.com/bihealth/annonars/issues/39)) ([773896e](https://www.github.com/bihealth/annonars/commit/773896e61751215b7b67c214f15751f0b76d3b04))

### [0.2.1](https://www.github.com/bihealth/annonars/compare/v0.2.0...v0.2.1) (2023-05-23)


### Bug Fixes

* "db-utils copy" now accepts "--all" and "--path-beds" ([#37](https://www.github.com/bihealth/annonars/issues/37)) ([0b50060](https://www.github.com/bihealth/annonars/commit/0b5006003dd5a0b28c5730b17e5ea40558bbda82))

## [0.2.0](https://www.github.com/bihealth/annonars/compare/v0.1.0...v0.2.0) (2023-05-23)


### Features

* add "db-utils copy" command ([#30](https://www.github.com/bihealth/annonars/issues/30)) ([#31](https://www.github.com/bihealth/annonars/issues/31)) ([f918a27](https://www.github.com/bihealth/annonars/commit/f918a275e80d9c6a18a464d79346d5430248c3d5))
* implement import and query for gnomAD-mtDNA ([#16](https://www.github.com/bihealth/annonars/issues/16)) ([#24](https://www.github.com/bihealth/annonars/issues/24)) ([95ea15d](https://www.github.com/bihealth/annonars/commit/95ea15d44856c19414e2bbdb3b19473b842ca18f))
* implement import and query for UCSC conservation ([#11](https://www.github.com/bihealth/annonars/issues/11)) ([#14](https://www.github.com/bihealth/annonars/issues/14)) ([3fc2f25](https://www.github.com/bihealth/annonars/commit/3fc2f257901055e86dc66b8cd3519e7215c55afd))
* implement import/query of dbsnp ([#17](https://www.github.com/bihealth/annonars/issues/17)) ([#21](https://www.github.com/bihealth/annonars/issues/21)) ([b027382](https://www.github.com/bihealth/annonars/commit/b027382e65ab92eb7b5bdc44be0c219b08aa9976))
* import and query for gnomAD {ex,gen}omes ([#18](https://www.github.com/bihealth/annonars/issues/18)) ([#25](https://www.github.com/bihealth/annonars/issues/25)) ([0e63d12](https://www.github.com/bihealth/annonars/commit/0e63d123fb9efdf8067ab27d63b53f9e694849c8))
* import and query for HelixMtDb VCF ([#15](https://www.github.com/bihealth/annonars/issues/15)) ([#23](https://www.github.com/bihealth/annonars/issues/23)) ([9dfa520](https://www.github.com/bihealth/annonars/commit/9dfa52027e37c548a7945580995bdac03c6a0f47))
* use explicit indicatif progress bars ([#32](https://www.github.com/bihealth/annonars/issues/32)) ([#33](https://www.github.com/bihealth/annonars/issues/33)) ([2ceb2c6](https://www.github.com/bihealth/annonars/commit/2ceb2c6ed9584d314504438a49b6d60013fb5390))

## 0.1.0 (2023-05-16)


### Features

* import of TSV files ([#1](https://www.github.com/bihealth/annonars/issues/1)) ([#4](https://www.github.com/bihealth/annonars/issues/4)) ([e0a2402](https://www.github.com/bihealth/annonars/commit/e0a24029872af214ca0b2d636a7dbf677deac2fc))
* querying of TSV files via CLI ([#2](https://www.github.com/bihealth/annonars/issues/2)) ([#7](https://www.github.com/bihealth/annonars/issues/7)) ([ceb908d](https://www.github.com/bihealth/annonars/commit/ceb908d893e4e2f570409911d5c794f99bbaa87b))
