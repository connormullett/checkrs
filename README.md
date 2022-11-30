
# CheckRS

Simple Sha256 checksum generation and verification tool.


## Usage

`checkrs [FLAGS] <input files>...`

To save the generated checksums, redirect to file using the following
on linux:

`checkrs foo.txt > my_checksums.checksum`

File extension doesn't matter.

Use `checkrs --help` for help and flags information.

## Installation

Requires rust to be installed before following the installation steps.

From source:

 - `git clone https://github.com/connormullett/checkrs.git`
 - `cd checkrs`
 - `cargo install --path .`

