use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about = "Download ZCTA shapefiles from US Census Bureau", long_about = None)]
struct Args {
    /// Output directory for downloaded files
    #[arg(short, long, default_value = "zip_data")]
    output: String,

    /// Year of ZCTA data to download (2010, 2020, etc.)
    #[arg(short, long, default_value = "2020")]
    year: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&args.output)?;

    // Construct the Census Bureau URL based on year
    let url = match args.year.as_str() {
        "2020" => "https://www2.census.gov/geo/tiger/TIGER2020/ZCTA520/tl_2020_us_zcta520.zip",
        "2010" => "https://www2.census.gov/geo/tiger/TIGER2010/ZCTA5/2010/tl_2010_us_zcta510.zip",
        _ => {
            eprintln!(
                "Error: Unsupported year '{}'. Supported years: 2010, 2020",
                args.year
            );
            std::process::exit(1);
        }
    };

    let filename = match args.year.as_str() {
        "2020" => "tl_2020_us_zcta520.zip",
        "2010" => "tl_2010_us_zcta510.zip",
        _ => unreachable!(),
    };

    let output_path = Path::new(&args.output).join(filename);

    println!("Downloading ZCTA data for year {}...", args.year);
    println!("URL: {}", url);
    println!("Output: {}", output_path.display());

    // Download the file
    println!("\nDownloading...");
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        eprintln!(
            "Error: HTTP request failed with status {}",
            response.status()
        );
        std::process::exit(1);
    }

    let total_size = response.content_length().unwrap_or(0);
    println!("File size: {} MB", total_size / 1_048_576);

    // Write to file
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);
    let content = response.bytes()?;
    writer.write_all(&content)?;
    writer.flush()?;

    println!("\nDownload complete!");

    // Extract the zip file
    println!("\nExtracting files...");
    let file = File::open(&output_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(&args.output).join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        println!("  Extracted: {}", file.name());
    }

    println!("\nExtraction complete!");
    Ok(())
}
