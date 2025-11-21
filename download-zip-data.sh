#!/bin/bash
set -e

OUTPUT_DIR="${1:-zip_data}"
URL="https://www2.census.gov/geo/tiger/TIGER2020/ZCTA520/tl_2020_us_zcta520.zip"
FILENAME="tl_2020_us_zcta520.zip"

mkdir -p "$OUTPUT_DIR"

echo "Downloading ZCTA 2020 data..."
echo "URL: $URL"
echo "Output: $OUTPUT_DIR/$FILENAME"

curl -L -o "$OUTPUT_DIR/$FILENAME" "$URL"

echo "Extracting files..."
unzip -o "$OUTPUT_DIR/$FILENAME" -d "$OUTPUT_DIR"

echo "Done!"
