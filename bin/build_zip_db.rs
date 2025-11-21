use anyhow::{Context, Result};
use clap::Parser;
use geo::geometry::Geometry;
use geo::{BoundingRect, Centroid};
use rstar::AABB;
use std::collections::HashMap;
use zipdip::ZipShape;

#[derive(Parser, Debug)]
#[command(author, version, about = "Process ZCTA shapefiles into binary format", long_about = None)]
struct Args {
    /// Input shapefile path (e.g. foo.shp)
    #[arg(short, long)]
    input: String,

    /// Output binary file path
    #[arg(short, long, default_value = "zipcodes.bin")]
    output: String,
}

fn process_shape(zip: &str, shape: Geometry) -> ZipShape {
    let centroid = shape
        .centroid()
        .expect("zip areas should have non-empty centroids");
    let bbox = shape
        .bounding_rect()
        .expect("zip areas should have non-empty bbox");

    ZipShape {
        zip: zip.to_string(),
        shape,
        bbox: AABB::from_corners(bbox.min().into(), bbox.max().into()),
        centroid,
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Reading shapefile: {}", args.input);

    // Read the shapefile
    let mut reader = shapefile::Reader::from_path(&args.input)
        .context(format!("Failed to open shapefile at '{}'", args.input))?;

    let mut polygons: Vec<ZipShape> = Vec::new();
    let mut centroids: HashMap<String, geo::Point> = HashMap::new();
    let mut skipped = 0;
    let mut processed = 0;

    // Iterate through all shapes and records
    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.context("Failed to read shape and record from shapefile")?;

        // Try to get the ZIP code from common ZCTA field names
        let zip = if let Some(shapefile::dbase::FieldValue::Character(Some(zip))) =
            record.get("ZCTA5CE20")
        {
            zip.clone()
        } else if let Some(shapefile::dbase::FieldValue::Character(Some(zip))) =
            record.get("ZCTA5CE10")
        {
            zip.clone()
        } else if let Some(shapefile::dbase::FieldValue::Character(Some(zip))) = record.get("ZCTA5")
        {
            zip.clone()
        } else if let Some(shapefile::dbase::FieldValue::Character(Some(zip))) =
            record.get("GEOID20")
        {
            zip.clone()
        } else if let Some(shapefile::dbase::FieldValue::Character(Some(zip))) =
            record.get("GEOID10")
        {
            zip.clone()
        } else {
            eprintln!("Warning: Could not find ZIP code field in record");
            skipped += 1;
            continue;
        };

        // Convert shapefile shape to geo_types::Geometry using native conversion
        let geometry: Geometry = match shape.try_into() {
            Ok(geom) => geom,
            Err(e) => {
                eprintln!("Warning: Failed to convert shape for ZIP {}: {:?}", zip, e);
                skipped += 1;
                continue;
            }
        };

        // Process polygon(s)
        let zip_shape = process_shape(&zip, geometry);
        centroids.insert(zip.clone(), zip_shape.centroid);
        polygons.push(zip_shape);
        processed += 1;
        if processed % 1000 == 0 {
            println!("Processed {} ZIP codes...", processed);
        }
    }

    println!("\nProcessed {} ZIP codes", processed);
    if skipped > 0 {
        println!("Skipped {} records", skipped);
    }

    // Serialize to binary format
    println!("Writing binary file: {}", args.output);
    let config = bincode::config::standard();
    let data_to_serialize = (polygons, centroids);
    let serialized = bincode::serde::encode_to_vec(&data_to_serialize, config)
        .context("Failed to serialize data to binary format")?;
    let size = serialized.len();

    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&args.output).parent() {
        std::fs::create_dir_all(parent).context(format!(
            "Failed to create output directory '{}'",
            parent.display()
        ))?;
    }

    std::fs::write(&args.output, serialized)
        .context(format!("Failed to write output file '{}'", args.output))?;

    println!("Done! Output size: {} bytes", size);

    Ok(())
}
