# zipdip

A Rust library for converting between US ZIP codes and geographic coordinates (latitude/longitude).

## Important Caveats

**ZIP codes are NOT geographic areas.** ZIP codes are postal delivery routes created by the USPS for mail sorting and delivery. They represent routes (lines), not bounded geographic regions (polygons).

Any "ZIP code boundary" is a reverse-engineered approximation.

### Source Data

This library uses **ZIP Code Tabulation Areas (ZCTAs)** from the US Census Bureau, which are approximate areal representations created by aggregating Census blocks. ZCTAs are the best publicly available approximation, but they are still approximations.

### Accuracy Limitations

- Results may not match the actual ZIP code for a given location
- ZCTAs are updated every 10 years; real ZIP codes change continuously
- PO Box-only ZIP codes and single-building ZIP codes may not be represented accurately
- Geographic centroids may fall in unpopulated areas

## Data Source

US Census Bureau TIGER/Line Shapefiles (ZCTAs):
https://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.html

## Quick Start

## Build the Database

```bash
# Download and extract Census data (downloads to ./zip_data/)
./download-zip-data.sh

# Build the database from the downloaded Census data (outputs to ./zipcodes.bin)
cargo build --release --features build-db
./target/release/build-zip-db --input zip_data/tl_2020_us_zcta520.shp
```

### Query

These commands expect to find the above zip database at ./zipcodes.bin
Otherwise, specify the path with `-d`/`--database=path/to/zipcodes.bin`

### zip2centroid

Get the centroid coordinates for a ZIP code.
Note: a centroid might be outside of its geometry, as in the case of a "C" shaped geometry.

```bash
$ ./target/release/zip2centroid 94102
37.779293 -122.419243

$ ./target/release/zip2centroid 10001
40.750633 -73.996210
```

### latlon2zip

Find the nearest ZIP code for given coordinates:

Note: negative number requires a `--` before the arguments

```bash
$ ./target/release/latlon2zip -- 37.7793 -122.4193
94102

$ ./target/release/latlon2zip -- 40.7589 -73.9851
10036
```

## Performance

The queries take about 500ms for me, which is slow.
The lookup itself is quick, but it loads the entire spatial database into memory every launch.

Some more performant alternatives if necessary:

1. Batch input to amortize cost of db load (probably easiest to write)
2. Run database in long running server, and query it.
3. Use a db format suitable for paging (e.g. sqlite + spatial) (probably easiest to use)

## License

MIT OR Apache-2.0
