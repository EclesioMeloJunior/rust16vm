#!/bin/bash

cargo build --release

./target/release/asm output.bin ./testdata/$1
./target/release/vm output.bin --debug