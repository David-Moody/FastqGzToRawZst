This is a micro program for ingesting GZIP-compressed sequencing data, where only the reads are of importance for downstream analysis.

By discarding all other information, and directly going from GZIP to ZStandard compression tons of storage space is saved. It is also significantly quicker to process overall.

The program assumes no line wrapping is used, with each FASTA entry being 4 lines. There is no limit on the length of each read.

## Building

- Have Rust configured
- Clone the repository
- `cargo build --release`
