use clap::Parser;
use zipdip::ZipCodeDb;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Find the nearest ZIP code for given coordinates"
)]
struct Args {
    /// Latitude
    latitude: f64,

    /// Longitude
    longitude: f64,

    /// Path to the ZIP code database file
    #[arg(short, long, default_value = "zipcodes.bin")]
    database: String,
}

fn main() {
    let args = Args::parse();

    // Load the database
    let db = match ZipCodeDb::from_file(&args.database) {
        Ok(db) => db,
        Err(e) => {
            eprintln!(
                "Error: Failed to load database from '{}': {}",
                args.database, e
            );
            eprintln!(
                "\nMake sure you have built the database first. See README.md for instructions."
            );
            std::process::exit(1);
        }
    };

    match db.lat_lon_to_zip(args.latitude, args.longitude) {
        Ok(zip) => println!("{}", zip),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
