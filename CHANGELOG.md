# Changelog

## [0.32.0](https://github.com/bihealth/annonars/compare/v0.31.2...v0.32.0) (2023-12-28)


### Features

* gnomAD v4 frequency import ([#275](https://github.com/bihealth/annonars/issues/275)) ([#368](https://github.com/bihealth/annonars/issues/368)) ([2a7e098](https://github.com/bihealth/annonars/commit/2a7e098924db0ae5023c3831fdbcc8d171259f51))
* integrate gnomAD v4 gene constraints ([#367](https://github.com/bihealth/annonars/issues/367)) ([#370](https://github.com/bihealth/annonars/issues/370)) ([2021af3](https://github.com/bihealth/annonars/commit/2021af38a22b733aad1419a98fc26ac588570c88))

## [0.31.2](https://github.com/bihealth/annonars/compare/v0.31.1...v0.31.2) (2023-12-22)


### Bug Fixes

* ncbi gene ID lookup ([#362](https://github.com/bihealth/annonars/issues/362)) ([#363](https://github.com/bihealth/annonars/issues/363)) ([3f2505f](https://github.com/bihealth/annonars/commit/3f2505fd2636cc87fdbb6ae80408f77fcc411914))

## [0.31.1](https://github.com/bihealth/annonars/compare/v0.31.0...v0.31.1) (2023-12-18)


### Bug Fixes

* add missing files for test ([#355](https://github.com/bihealth/annonars/issues/355)) ([5037df5](https://github.com/bihealth/annonars/commit/5037df5b52e3ec3f69626b783f95a41bcb949160))
* allow case insensitive gene search in server ([#354](https://github.com/bihealth/annonars/issues/354)) ([#357](https://github.com/bihealth/annonars/issues/357)) ([701b58c](https://github.com/bihealth/annonars/commit/701b58c8c63607b2da4f0cd4915cd7b6c4f604d7))
* remove requirements of 2+ elements in gene lookup ([#348](https://github.com/bihealth/annonars/issues/348)) ([#358](https://github.com/bihealth/annonars/issues/358)) ([e3f8c3b](https://github.com/bihealth/annonars/commit/e3f8c3b4876e1a7195b8ebc69ca76c96a3e8686c))

## [0.31.0](https://github.com/bihealth/annonars/compare/v0.30.1...v0.31.0) (2023-12-14)


### Features

* adding support for new "flagged" review status ([#352](https://github.com/bihealth/annonars/issues/352)) ([61d5248](https://github.com/bihealth/annonars/commit/61d52482128a69fdb08c5c20bf80aa46d09cfcf8))

## [0.30.1](https://github.com/bihealth/annonars/compare/v0.30.0...v0.30.1) (2023-12-01)


### Bug Fixes

* enum mapping from protobuf ([#345](https://github.com/bihealth/annonars/issues/345)) ([7dfccaa](https://github.com/bihealth/annonars/commit/7dfccaabba46556ac2c60afa4eb2f30b87fa3ffe))

## [0.30.0](https://github.com/bihealth/annonars/compare/v0.29.4...v0.30.0) (2023-11-30)


### Features

* implementing REST access to ClinVar SV data ([#343](https://github.com/bihealth/annonars/issues/343)) ([e1975c6](https://github.com/bihealth/annonars/commit/e1975c6f164567c72a4401f27c48e42b2bb17b66))

## [0.29.4](https://github.com/bihealth/annonars/compare/v0.29.3...v0.29.4) (2023-11-24)


### Bug Fixes

* adding Debug trait to regions query IntervalTree ([#337](https://github.com/bihealth/annonars/issues/337)) ([2e014f4](https://github.com/bihealth/annonars/commit/2e014f4b9721e60fae4839dbb6e21febb3f48c76))
* log to stderr ([#336](https://github.com/bihealth/annonars/issues/336)) ([aefa8ae](https://github.com/bihealth/annonars/commit/aefa8ae5cdbebf68aef1bf4129e33f575a2d8cfe))

## [0.29.3](https://github.com/bihealth/annonars/compare/v0.29.2...v0.29.3) (2023-11-24)


### Bug Fixes

* another issue in enum protobuf conversion (with regions) ([#335](https://github.com/bihealth/annonars/issues/335)) ([aeeace3](https://github.com/bihealth/annonars/commit/aeeace35c76c502d39cdf1eac6aeb88fb2f938f0))
* resolve issue with enum conversion to protobuf ones ([#333](https://github.com/bihealth/annonars/issues/333)) ([517ba37](https://github.com/bihealth/annonars/commit/517ba377fa37a1743c37c637a37cf78ccd7287df))

## [0.29.2](https://github.com/bihealth/annonars/compare/v0.29.1...v0.29.2) (2023-11-23)


### Bug Fixes

* adding some missing interval-related conversions ([#331](https://github.com/bihealth/annonars/issues/331)) ([7730d07](https://github.com/bihealth/annonars/commit/7730d07f9cee8a45bea932679907e7a03d0e5256))

## [0.29.1](https://github.com/bihealth/annonars/compare/v0.29.0...v0.29.1) (2023-11-22)


### Bug Fixes

* adding missing INFO/CPX_TYPE values for gnomAD-SV v4.0 ([#329](https://github.com/bihealth/annonars/issues/329)) ([3d628d8](https://github.com/bihealth/annonars/commit/3d628d88af58e3a5c02daa9e158164df093a8d02))

## [0.29.0](https://github.com/bihealth/annonars/compare/v0.28.0...v0.29.0) (2023-11-22)


### Bug Fixes

* use prefixing conventions for enums ([#326](https://github.com/bihealth/annonars/issues/326)) ([6e4aab9](https://github.com/bihealth/annonars/commit/6e4aab9476cefdc03d137f8defcaaadd73186a6a))

## [0.28.0](https://github.com/bihealth/annonars/compare/v0.27.0...v0.28.0) (2023-11-22)


### Features

* adding information from DECIPHER HI ([#323](https://github.com/bihealth/annonars/issues/323)) ([#324](https://github.com/bihealth/annonars/issues/324)) ([06b51a6](https://github.com/bihealth/annonars/commit/06b51a64f599bee58a1326e5a4dbdcfb0fbf3803))

## [0.27.0](https://github.com/bihealth/annonars/compare/v0.26.1...v0.27.0) (2023-11-21)


### Features

* add multi-cohort support for gnomAD CNV v4 ([#322](https://github.com/bihealth/annonars/issues/322)) ([1d0cd26](https://github.com/bihealth/annonars/commit/1d0cd260b9226598c024448bf281687f17eb0f9e))
* clingen gene dosage pathogenicity for 37 and 38 ([#320](https://github.com/bihealth/annonars/issues/320)) ([cca54b4](https://github.com/bihealth/annonars/commit/cca54b4777d89d9e4f4102ea67901d14b3f63f12))
* implement clingen gene dosage pathogenicity information ([#316](https://github.com/bihealth/annonars/issues/316)) ([#317](https://github.com/bihealth/annonars/issues/317)) ([b1aff38](https://github.com/bihealth/annonars/commit/b1aff383071b3a124f03e6581eb4728a26917a4e))
* implement support for DOMINO annotation of genes ([#224](https://github.com/bihealth/annonars/issues/224)) ([#318](https://github.com/bihealth/annonars/issues/318)) ([fe855a6](https://github.com/bihealth/annonars/commit/fe855a66a1c1c752ed2d29ad9b8e107650c869a5))
* implementing ClinGen region dosage annotation ([#282](https://github.com/bihealth/annonars/issues/282)) ([#319](https://github.com/bihealth/annonars/issues/319)) ([57e1408](https://github.com/bihealth/annonars/commit/57e14087e0ecf30ca2a2d94a0afc663510b4ead4))
* provide proto3 JSON serialization for prost structs ([#301](https://github.com/bihealth/annonars/issues/301)) ([#314](https://github.com/bihealth/annonars/issues/314)) ([e3e7fa1](https://github.com/bihealth/annonars/commit/e3e7fa15c5604511b2f1b9ef058547a57fd6b3db))

## [0.26.1](https://github.com/bihealth/annonars/compare/v0.26.0...v0.26.1) (2023-11-21)


### Build System

* cleanup serde_with dependency ([#312](https://github.com/bihealth/annonars/issues/312)) ([932b840](https://github.com/bihealth/annonars/commit/932b840845b8adfc66820fdc7a78c97aff8690da))

## [0.26.0](https://github.com/bihealth/annonars/compare/v0.25.0...v0.26.0) (2023-11-20)


### Features

* adding number of ExAC CNV cases as constant ([#305](https://github.com/bihealth/annonars/issues/305)) ([7689744](https://github.com/bihealth/annonars/commit/7689744a917755995e52a6d08c6ddf6e39afeb39))
* adding PartialEq/Clone for gnomad_sv::cli::query::Record ([#306](https://github.com/bihealth/annonars/issues/306)) ([fe5f978](https://github.com/bihealth/annonars/commit/fe5f978348aa21658ca5bae098da3eb975a12875))
* serializing record enums with snake_case ([#302](https://github.com/bihealth/annonars/issues/302)) ([#303](https://github.com/bihealth/annonars/issues/303)) ([7f76293](https://github.com/bihealth/annonars/commit/7f76293a2c0989d701fe2374e9f42e96484f3f66))

## [0.25.0](https://github.com/bihealth/annonars/compare/v0.24.5...v0.25.0) (2023-11-18)


### Features

* add support for RefSeq functional data ([#299](https://github.com/bihealth/annonars/issues/299)) ([#300](https://github.com/bihealth/annonars/issues/300)) ([5aa6f63](https://github.com/bihealth/annonars/commit/5aa6f636ea2e3ed1a1bc33c07601a6e5a5014d7d))
* adding 50bp filter for REF/ALT on clinvar-sv import ([#293](https://github.com/bihealth/annonars/issues/293)) ([#294](https://github.com/bihealth/annonars/issues/294)) ([831830a](https://github.com/bihealth/annonars/commit/831830aa1ba46e0323afbd0f96fcae58d954d2a2))
* adding by-accession colum family for clinvar-minimal ([#289](https://github.com/bihealth/annonars/issues/289)) ([#296](https://github.com/bihealth/annonars/issues/296)) ([807abaf](https://github.com/bihealth/annonars/commit/807abaf3d3334ecd040af746e980dcfe81382808))
* adding support for accession query in dbSNP ([#288](https://github.com/bihealth/annonars/issues/288)) ([#295](https://github.com/bihealth/annonars/issues/295)) ([c1ebece](https://github.com/bihealth/annonars/commit/c1ebece39f8cb636d485b8c5460054361de853c6))
* adding support for clinvar-sv data ([#227](https://github.com/bihealth/annonars/issues/227)) ([#290](https://github.com/bihealth/annonars/issues/290)) ([1837899](https://github.com/bihealth/annonars/commit/18378993f6bc63354239a7a0794eb3d73b086940))
* adding support for gnomAD-SV ([#291](https://github.com/bihealth/annonars/issues/291)) ([#297](https://github.com/bihealth/annonars/issues/297)) ([8195101](https://github.com/bihealth/annonars/commit/81951018461795b042c9aa625ce3e5c7e3fa269a))
* range and accession queries for gnomad-sv ([#298](https://github.com/bihealth/annonars/issues/298)) ([db2cb67](https://github.com/bihealth/annonars/commit/db2cb67c8aee3e667fa565c31240baa8588352dd))

## [0.24.5](https://github.com/bihealth/annonars/compare/v0.24.4...v0.24.5) (2023-11-08)


### Build System

* **deps:** bump the noodles group with 3 updates ([#274](https://github.com/bihealth/annonars/issues/274)) ([8c30958](https://github.com/bihealth/annonars/commit/8c309584173b512a9ae7815d7203a3ede0b96954))

## [0.24.4](https://github.com/bihealth/annonars/compare/v0.24.3...v0.24.4) (2023-10-23)


### Bug Fixes

* bump noodles-vcf to v0.43 ([#267](https://github.com/bihealth/annonars/issues/267)) ([1b7a75c](https://github.com/bihealth/annonars/commit/1b7a75c030ec5c29aa1a8d96d1add61ba3a467a0))

## [0.24.3](https://github.com/bihealth/annonars/compare/v0.24.2...v0.24.3) (2023-10-23)


### Bug Fixes

* bump noodles dependencies ([#264](https://github.com/bihealth/annonars/issues/264)) ([ccf4bbd](https://github.com/bihealth/annonars/commit/ccf4bbda0d3b080507b365fb08e843785c11b248))

## [0.24.2](https://github.com/bihealth/annonars/compare/v0.24.1...v0.24.2) (2023-10-21)


### Bug Fixes

* moving from hgvs to biocommons_bioutils dependency ([#262](https://github.com/bihealth/annonars/issues/262)) ([48beaa9](https://github.com/bihealth/annonars/commit/48beaa97d194086087bed548d0e80e9235895c95))

## [0.24.1](https://github.com/bihealth/annonars/compare/v0.24.0...v0.24.1) (2023-10-18)


### Bug Fixes

* more robust import of clinvar variants ([#260](https://github.com/bihealth/annonars/issues/260)) ([72c8267](https://github.com/bihealth/annonars/commit/72c8267352f1329c897efff7699ef072cea1bf7b))

## [0.24.0](https://github.com/bihealth/annonars/compare/v0.23.1...v0.24.0) (2023-10-18)


### Features

* allow returning None for query interfaces ([#255](https://github.com/bihealth/annonars/issues/255)) ([#256](https://github.com/bihealth/annonars/issues/256)) ([b4bf349](https://github.com/bihealth/annonars/commit/b4bf3492ed0252d13139787dfb284d9dc76fb431))
* ensure that RocksDB dbs are opened with absolute path ([#252](https://github.com/bihealth/annonars/issues/252)) ([#257](https://github.com/bihealth/annonars/issues/257)) ([bf74e69](https://github.com/bihealth/annonars/commit/bf74e6956ee21dab1b4a865a4be03fdd50e17792))
* making more code public in tsv query interface ([#253](https://github.com/bihealth/annonars/issues/253)) ([dd4eecc](https://github.com/bihealth/annonars/commit/dd4eecce3291bfae903e9b3a96d92d41bbd25537))
* properly represent clinvar VCV/RCV structure in protobufs ([#242](https://github.com/bihealth/annonars/issues/242)) ([#259](https://github.com/bihealth/annonars/issues/259)) ([9095773](https://github.com/bihealth/annonars/commit/90957736c08106d31de4ce92aaf191d068526bc8))


### Bug Fixes

* make "clinvar-genes import" more robust ([#247](https://github.com/bihealth/annonars/issues/247)) ([#258](https://github.com/bihealth/annonars/issues/258)) ([efbe123](https://github.com/bihealth/annonars/commit/efbe123bacca8921fcf15ffcf99fb3c16e0a8673))

## [0.23.1](https://github.com/bihealth/annonars/compare/v0.23.0...v0.23.1) (2023-10-16)


### Bug Fixes

* bumping noodles dependencies ([#245](https://github.com/bihealth/annonars/issues/245)) ([9fe23c8](https://github.com/bihealth/annonars/commit/9fe23c874d45c413085d9eb3559531b822b8daf7))

## [0.23.0](https://github.com/bihealth/annonars/compare/v0.22.0...v0.23.0) (2023-10-16)


### Features

* making more query_for_variant functions public ([#243](https://github.com/bihealth/annonars/issues/243)) ([151a7b6](https://github.com/bihealth/annonars/commit/151a7b64d45ca5ba4529a2b568c5bdd1bd9872f2))

## [0.22.0](https://github.com/bihealth/annonars/compare/v0.21.1...v0.22.0) (2023-10-13)


### Features

* expose open_rocksdb() functions ([#240](https://github.com/bihealth/annonars/issues/240)) ([9f9fd2d](https://github.com/bihealth/annonars/commit/9f9fd2d3fb148c7c78f2e413d24ad172e3d6a7c8))

## [0.21.1](https://github.com/bihealth/annonars/compare/v0.21.0...v0.21.1) (2023-10-12)


### Bug Fixes

* use indexmap v2 in serde_with ([#237](https://github.com/bihealth/annonars/issues/237)) ([8c57c35](https://github.com/bihealth/annonars/commit/8c57c35b026d277808cc1659d529f0ff62840b01))

## [0.21.0](https://github.com/bihealth/annonars/compare/v0.20.0...v0.21.0) (2023-10-02)


### Features

* release 0.21.0 with breaking dependency updates ([676886e](https://github.com/bihealth/annonars/commit/676886e710e1837c69d512f1ec70354ce1b05d07))

## [0.20.0](https://github.com/bihealth/annonars/compare/v0.19.0...v0.20.0) (2023-09-18)


### Features

* bumping dependencies (in particular noodles-vcf) ([#215](https://github.com/bihealth/annonars/issues/215)) ([f455b5e](https://github.com/bihealth/annonars/commit/f455b5eeddee63fcc9355e168b7ff4b824db631c))

## [0.19.0](https://github.com/bihealth/annonars/compare/v0.18.0...v0.19.0) (2023-09-13)


### Features

* bump rocksdb-utils-lookup for hierarchical index/filter ([#211](https://github.com/bihealth/annonars/issues/211)) ([c3decd5](https://github.com/bihealth/annonars/commit/c3decd56ee5dcd4bd4eb1679049bb9685b12d1ae))
* make GTEx support more space efficient via quantiles/enums ([#214](https://github.com/bihealth/annonars/issues/214)) ([e583343](https://github.com/bihealth/annonars/commit/e5833435d1b62bf2f726dd9690e63094c2048d9e))


### Bug Fixes

* pick up clinvar-genes database in Dockerfile entrypoint ([#212](https://github.com/bihealth/annonars/issues/212)) ([c945f26](https://github.com/bihealth/annonars/commit/c945f267339cfe862dd3bcc55ff364377d82f7b9))

## [0.18.0](https://github.com/bihealth/annonars/compare/v0.17.0...v0.18.0) (2023-09-12)


### Features

* adding support for clinvar-genes ([#202](https://github.com/bihealth/annonars/issues/202)) ([#205](https://github.com/bihealth/annonars/issues/205)) ([857c5dd](https://github.com/bihealth/annonars/commit/857c5dd1e9a46839f68e8a8f75a76c35e4819288))
* adding support for gtex gene expression ([#126](https://github.com/bihealth/annonars/issues/126)) ([#210](https://github.com/bihealth/annonars/issues/210)) ([033041e](https://github.com/bihealth/annonars/commit/033041e9bcffc3af31e3bb52dd0c3767d8a090de))
* switching to clinvar-data-jsonl for clinvar-minimal ([#202](https://github.com/bihealth/annonars/issues/202)) ([#203](https://github.com/bihealth/annonars/issues/203)) ([0e17128](https://github.com/bihealth/annonars/commit/0e171283269218973c20aceb62f6621b7217425c))

## [0.17.0](https://github.com/bihealth/annonars/compare/v0.16.0...v0.17.0) (2023-08-31)


### Features

* adding /genes/lookup endpoint ([#193](https://github.com/bihealth/annonars/issues/193)) ([eeb5753](https://github.com/bihealth/annonars/commit/eeb57530ada50898cf860348f3a69f7eff1abf79))
* adding /genes/search to annonars ([#191](https://github.com/bihealth/annonars/issues/191)) ([bc39d84](https://github.com/bihealth/annonars/commit/bc39d8443946a456ebf06aeddd5e73bd3bc96ec6))

## [0.16.0](https://github.com/bihealth/annonars/compare/v0.15.0...v0.16.0) (2023-08-28)


### ⚠ BREAKING CHANGES

* fixing overrides in clingen import ([#184](https://github.com/bihealth/annonars/issues/184))

### Bug Fixes

* fixing overrides in clingen import ([#184](https://github.com/bihealth/annonars/issues/184)) ([e1e9e9f](https://github.com/bihealth/annonars/commit/e1e9e9f49606647f476a2fde1036b310629de260))

## [0.15.0](https://github.com/bihealth/annonars/compare/v0.14.1...v0.15.0) (2023-08-25)


### Features

* adding import of clingen gene curation ([#145](https://github.com/bihealth/annonars/issues/145)) ([#178](https://github.com/bihealth/annonars/issues/178)) ([5d50940](https://github.com/bihealth/annonars/commit/5d509405bd685b172d92e0a2b1be1b8db2657d15))
* adding pHaplo, pTriplo, sHet as seen in DECIPHER ([#128](https://github.com/bihealth/annonars/issues/128)) ([#180](https://github.com/bihealth/annonars/issues/180)) ([1ac1a64](https://github.com/bihealth/annonars/commit/1ac1a646684ae0a9a07d0eb588bc149f1e8748e7))
* import of OMIM disease names ([#165](https://github.com/bihealth/annonars/issues/165)) ([#181](https://github.com/bihealth/annonars/issues/181)) ([7632438](https://github.com/bihealth/annonars/commit/7632438e8a8d29f57dbbe69819dd989b562c49b8))
* import of Orphanet disease names ([#165](https://github.com/bihealth/annonars/issues/165)) ([#182](https://github.com/bihealth/annonars/issues/182)) ([ff44a10](https://github.com/bihealth/annonars/commit/ff44a10b9b6228f2adc7093ed7f3825c6e158b1c))

## [0.14.1](https://github.com/bihealth/annonars/compare/v0.14.0...v0.14.1) (2023-07-31)


### Bug Fixes

* path to CADD in docker entrypoint script ([#161](https://github.com/bihealth/annonars/issues/161)) ([bb1b884](https://github.com/bihealth/annonars/commit/bb1b884ca3a17a2903d24a484145e50c0468e444))

## [0.14.0](https://github.com/bihealth/annonars/compare/v0.13.0...v0.14.0) (2023-07-28)


### Features

* using snake_case rather than kebab-case ([#158](https://github.com/bihealth/annonars/issues/158)) ([d8b0836](https://github.com/bihealth/annonars/commit/d8b08364a777db46fd089c796addd8451b34ed14))


### Bug Fixes

* remove redundancy in ucsc conservation import ([#159](https://github.com/bihealth/annonars/issues/159)) ([e749269](https://github.com/bihealth/annonars/commit/e7492692865af0abdc0b9106e1bc8d8ee3132adb))

## [0.13.0](https://github.com/bihealth/annonars/compare/v0.12.9...v0.13.0) (2023-07-27)


### Features

* serving clinvar information through REST API ([#155](https://github.com/bihealth/annonars/issues/155)) ([46cbe74](https://github.com/bihealth/annonars/commit/46cbe74a17723b7b02a9e5d04be37c9c6ea42c19))


### Bug Fixes

* proper decoding in variant annotation server ([#151](https://github.com/bihealth/annonars/issues/151)) ([48f996b](https://github.com/bihealth/annonars/commit/48f996b1246d5c905adac5e732eae6d69aaf9a01))

## [0.12.9](https://github.com/bihealth/annonars/compare/v0.12.8...v0.12.9) (2023-07-10)


### Bug Fixes

* docker startup with genes ([#129](https://github.com/bihealth/annonars/issues/129)) ([a7fbfef](https://github.com/bihealth/annonars/commit/a7fbfef17375e29cf0a1635e93b69f23a8323fd7))

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
