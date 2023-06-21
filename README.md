[![install with bioconda](https://img.shields.io/badge/install%20with-bioconda-brightgreen.svg?style=flat)](http://bioconda.github.io/recipes/annonars/README.html)
[![CI](https://github.com/bihealth/annonars/actions/workflows/rust.yml/badge.svg)](https://github.com/bihealth/annonars/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/bihealth/annonars/branch/main/graph/badge.svg?token=UjTNKN6kCI)](https://codecov.io/gh/bihealth/annonars)
[![Crates.io](https://img.shields.io/crates/d/annonars.svg)](https://crates.io/crates/annonars)
[![Crates.io](https://img.shields.io/crates/v/annonars.svg)](https://crates.io/crates/annonars)
[![Crates.io](https://img.shields.io/crates/l/annonars.svg)](https://crates.io/crates/annonars)

<a href="https://commons.wikimedia.org/wiki/File:Annona_squamosa_Blanco1.192.png"><img src="https://github.com/bihealth/annonars/blob/main/utils/img/annona-wikimedia.jpg?raw=true" width="200px" height="321px" align="right"></a>

# annonars

Genome annotation with Rust and RocksDB.

Also:

> Annona (from Taíno annon) is a genus of flowering plants in the pawpaw/sugar apple family, Annonaceae. It is the second largest genus in the family after Guatteria, containing approximately 166 species of mostly Neotropical and Afrotropical trees and shrubs.
>
> [Annona -- Wikipedia](https://en.wikipedia.org/wiki/Annona)

## Running the CLI

You can enable the annonars CLI by building the project with the `cli` feature (easiest done with `--all-features`):

```
# cargo run --all-features -- --help
```

## Working with TSV Files

When built with the `cli` feature, `annonars` allows you to to import variant annotations from TSV files into RocksDB databases.
This allows you to import variant annotation TSVs as provided by [CADD](https://cadd.gs.washington.edu/) or [dbNSFP](https://sites.google.com/site/jpopgen/dbNSFP).
Variants are specified in SPDI representation as described in [Holmes et al. 2020](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7523648/).
All variants in one file refer to the same genome build.

You can import TSV files using `tsv import`.
For example, to import the "CADD with all annotations" file, you can use the following:

```
# annonars tsv import \
    --path-in-tsv InDels_inclAnno.tsv.gz \
    --path-in-tsv whole_genome_SNVs_inclAnno.tsv.gz \
    --path-out-rocksdb cadd-rocksdb \
    --genome-release grch37 \
    --db-name cadd \
    --db-version 1.6 \
    --col-chrom Chrom \
    --col-start Pos \
    --col-ref Ref \
    --col-alt Alt \
    --skip-row-count=1 \
    --inference-row-count 100000 \
    --add-default-null-values
```

This will:

- Set the genome release of the database to `grch37`.
- Set the meta information about data name and version to `cadd` in version `1.6`.
- Use the columns `Chrom`, `Pos`, `Ref`, and `Alt` to specify the variant.
- The CADD files start with a copyright line above the columns header so we skip one row.
- Use the first 100,000 rows to infer the data types of the columns.
- Specify the default set of null values (`NA`, `.`, `-`) to be used for missing values.

When run, `annonars` will first try to infer the schema from the first 100,000 rows.
It will then import the data into a RocksDB database.
The resulting schema will be dumped in JSON format.
If necessary, you can also specify a file with the schema in JSON file to use as a seed for the schema inference.
You might need to do this if you see an `"Unknown"` type in the schema.
At the end, the database will be compacted, which may take some time but is necessary to reduce the size of the database and ensure that it can be read in read-only note.

After everything is done, you will have to manually look for a file matching `*.log` in the output RocksDB directory.
This is the write-ahead log (WAL) of RocksDB file and can be safely deleted (it should be zero-sized if everything went well).

Here is how you can import dbNSFP.
Note that you will have to build one RocksDB database per genome release that you want to use for lookup.

```
# annonars tsv import \
    $(for f in dbNSFP4.4a_variant.*.gz; do echo --path-in-tsv $f; done) \
    --path-out-rocksdb dbnsfp-rocksdb \
    --genome-release grch37 \
    --db-name dbnsfp \
    --db-version 4.4a \
    --col-chrom hg19_chr \
    --col-start hg19_pos(1-based) \
    --col-ref ref \
    --col-alt alt \
    --inference-row-count 100000 \
    --null-values=.
```

`annonars` can use tabix indices to speedup database building.
If there is a `.tbi` file for each of the input files then `annonars` will use it and perform import in a parallel fashion based on genome windows.
Otherwise, `annonars` will import all input files in parallel (yet read through each file sequentially).
By default, one thread for each CPU core on the system is used.
You can control the number of threads to use by setting the environment variable `RAYON_NUM_THREADS`.

You can query the rocksdb databases using `tsv query`, either based on a variant, a position (all variants at the position), or a region.
Note that `annonars` uses SPDI-style coordinates (1-based, inclusive) for all queries.
You can optionally prefix your query with a gnome release (comparison is done case insensitive) and `annonars` will check whether the database matches the genome release.

Examples:

```
# tsv query --path-rocksdb tests/tsv/example/data.tsv.gz.db --range GRCh37:1:1000:A:T
# tsv query --path-rocksdb tests/tsv/example/data.tsv.gz.db --pos GRCh37:1:1000
# tsv query --path-rocksdb tests/tsv/example/data.tsv.gz.db --range GRCh37:1:1000:1001
```

## Managing GitHub Project with Terraform

```
# export GITHUB_OWNER=bihealth
# export GITHUB_TOKEN=ghp_<thetoken>
# terraform import github_repository.annonars annonars

# cd utils/terraform
# terraform validate
# terraform fmt
# terraform plan
# terraform apply
```

## Developer Notes

The `v1` token in the protobuf schema refers to the **internal** version of the protocol buffer and not the version of, e.g., gnomAD.
