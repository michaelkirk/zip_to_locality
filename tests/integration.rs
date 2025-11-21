use zipdip::{ZipCodeDb, ZipCodeError};

#[test]
fn test_invalid_zip_format() {
    let db = ZipCodeDb::new().unwrap();

    // Too short
    assert!(matches!(
        db.zip_to_centroid("1234"),
        Err(ZipCodeError::InvalidZipFormat(_))
    ));

    // Too long
    assert!(matches!(
        db.zip_to_centroid("123456"),
        Err(ZipCodeError::InvalidZipFormat(_))
    ));

    // Non-numeric
    assert!(matches!(
        db.zip_to_centroid("abcde"),
        Err(ZipCodeError::InvalidZipFormat(_))
    ));

    // Mixed
    assert!(matches!(
        db.zip_to_centroid("12a45"),
        Err(ZipCodeError::InvalidZipFormat(_))
    ));
}

#[test]
fn test_invalid_coordinates() {
    let db = ZipCodeDb::new().unwrap();

    // Latitude out of range
    assert!(matches!(
        db.lat_lon_to_zip(91.0, 0.0),
        Err(ZipCodeError::InvalidCoordinates(_, _))
    ));

    assert!(matches!(
        db.lat_lon_to_zip(-91.0, 0.0),
        Err(ZipCodeError::InvalidCoordinates(_, _))
    ));

    // Longitude out of range
    assert!(matches!(
        db.lat_lon_to_zip(0.0, 181.0),
        Err(ZipCodeError::InvalidCoordinates(_, _))
    ));

    assert!(matches!(
        db.lat_lon_to_zip(0.0, -181.0),
        Err(ZipCodeError::InvalidCoordinates(_, _))
    ));
}

#[test]
fn test_empty_database() {
    let db = ZipCodeDb::new().unwrap();

    // Valid format but not in database
    assert!(matches!(
        db.zip_to_centroid("94102"),
        Err(ZipCodeError::ZipNotFound(_))
    ));

    // No data for reverse lookup
    assert!(matches!(
        db.lat_lon_to_zip(37.7793, -122.4193),
        Err(ZipCodeError::DataLoadError(_))
    ));
}

// This test will only work once we have actual data loaded
#[test]
#[ignore]
fn test_known_zip_codes() {
    // Load from the data file (requires processed data)
    let db = ZipCodeDb::from_file("zip_data/zipcodes.bin").unwrap();

    // Test San Francisco ZIP code
    let point = db.zip_to_centroid("94102").unwrap();
    assert!((point.y() - 37.78).abs() < 0.1);
    assert!((point.x() - (-122.42)).abs() < 0.1);

    // Test New York City ZIP code
    let point = db.zip_to_centroid("10001").unwrap();
    assert!((point.y() - 40.75).abs() < 0.1);
    assert!((point.x() - (-73.99)).abs() < 0.1);

    // Test Beverly Hills ZIP code
    let point = db.zip_to_centroid("90210").unwrap();
    assert!((point.y() - 34.10).abs() < 0.1);
    assert!((point.x() - (-118.41)).abs() < 0.1);
}

// This test will only work once we have actual data loaded
#[test]
#[ignore]
fn test_reverse_lookup() {
    let db = ZipCodeDb::from_file("zip_data/zipcodes.bin").unwrap();

    // Test San Francisco coordinates
    let zip = db.lat_lon_to_zip(37.7793, -122.4193).unwrap();
    assert!(zip.starts_with("941")); // Should be in the 941xx range

    // Test New York City coordinates
    let zip = db.lat_lon_to_zip(40.7589, -73.9851).unwrap();
    assert!(zip.starts_with("100")); // Should be in the 100xx range
}
