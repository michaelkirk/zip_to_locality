use clap::Parser;
use zipdip::ZipCodeDb;

#[derive(Parser, Debug)]
#[command(author, version, about = "Get centroid coordinates for a ZIP code")]
struct Args {
    /// ZIP code to look up
    zipcode: String,

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

    match db.zip_to_centroid(&args.zipcode) {
        Ok(point) => println!("{} {}", point.y(), point.x()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
