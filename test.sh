#!/bin/bash

output_file="clients.csv"
golden_output_file="golden_clients.csv"

# Run test and generate CSV output
cargo run --release -- transactions.csv > "${output_file}"

# Load golden output for comparison
if ! diff -q "$output_file" "$golden_output_file" > /dev/null; then
    echo "Test failed: Output does not match golden output."
    diff "$output_file" "$golden_output_file"
    exit 1
else
    echo "Test passed: Output matches golden output."
fi

# Clean up
rm "$output_file"
