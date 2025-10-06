#!/bin/bash
# Test script for logpile

echo "=== Test 1: Basic search with table output ==="
./target/release/logpile "ERROR" examples/sample.log --bucket 300
echo

echo "=== Test 2: CSV output ==="
./target/release/logpile "ERROR" examples/sample.log --bucket 300 --csv
echo

echo "=== Test 3: JSON output ==="
./target/release/logpile "WARN" examples/sample.log --bucket 300 --json
echo

echo "=== Test 4: Multiple patterns ==="
./target/release/logpile "ERROR" examples/sample.log --grep "WARN" --bucket 600
echo

echo "=== Test 5: Gzipped file ==="
./target/release/logpile "ERROR" examples/sample.log.gz --bucket 300
echo

echo "=== Test 6: Stdin input ==="
cat examples/sample.log | ./target/release/logpile "INFO" --bucket 600
echo

echo "=== Test 7: Auto bucket size ==="
./target/release/logpile "ERROR|WARN" examples/sample.log --bucket auto
echo

echo "=== Test 8: No default pattern (count all) ==="
./target/release/logpile --no-default-pattern examples/sample.log --bucket 600
echo

echo "=== Test 9: ASCII plot ==="
./target/release/logpile "ERROR" examples/sample.log --bucket 300 --plot
echo

echo "=== All tests completed! ==="


