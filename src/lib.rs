mod error;

pub use error::ZipCodeError;

use geo::{point, Contains};
use rstar::{RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A ZIP code polygon in the R-tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipShape {
    pub zip: String,
    pub shape: geo::Geometry,
    pub bbox: rstar::AABB<geo::Point>,
    /// Centroid for fallback
    pub centroid: geo::Point, // (lon, lat)
}

impl RTreeObject for ZipShape {
    type Envelope = rstar::AABB<geo::Point>;

    fn envelope(&self) -> Self::Envelope {
        self.bbox
    }
}

impl rstar::PointDistance for ZipShape {
    /// Distance to centroid - this is only used as a fallback
    fn distance_2(&self, point: &geo::Point) -> f64 {
        let dx = self.centroid.x() - point.x();
        let dy = self.centroid.y() - point.y();
        dx * dx + dy * dy
    }
}

/// Database for ZIP code lookups
pub struct ZipCodeDb {
    /// HashMap for ZIP -> centroid lookups (O(1))
    zip_to_coords: HashMap<String, geo::Point>,
    /// R-tree for spatial polygon queries (O(log n))
    rtree: RTree<ZipShape>,
}

impl ZipCodeDb {
    /// Load the embedded dataset
    pub fn new() -> Result<Self, ZipCodeError> {
        // For now, create an empty database
        // This will be populated when we have the processed data
        let zip_to_coords = HashMap::new();
        let rtree = RTree::new();

        Ok(Self {
            zip_to_coords,
            rtree,
        })
    }

    /// Load from a binary file (for use after processing data)
    pub fn from_file(path: &str) -> Result<Self, ZipCodeError> {
        let data = std::fs::read(path)
            .map_err(|e| ZipCodeError::DataLoadError(format!("Failed to read file: {}", e)))?;

        let config = bincode::config::standard();
        let (polygons, zip_to_coords): (Vec<ZipShape>, HashMap<String, geo::Point>) =
            bincode::serde::decode_from_slice(&data, config)
                .map_err(|e| ZipCodeError::DataLoadError(format!("Failed to deserialize: {}", e)))?
                .0;

        let rtree = RTree::bulk_load(polygons);

        Ok(Self {
            zip_to_coords,
            rtree,
        })
    }

    /// Get centroid coordinates for a ZIP code
    /// Returns (latitude, longitude)
    pub fn zip_to_centroid(&self, zip: &str) -> Result<geo::Point, ZipCodeError> {
        // Validate ZIP code format (5 digits)
        if !zip.chars().all(|c| c.is_ascii_digit()) || zip.len() != 5 {
            return Err(ZipCodeError::InvalidZipFormat(zip.to_string()));
        }

        self.zip_to_coords
            .get(zip)
            .copied()
            .ok_or_else(|| ZipCodeError::ZipNotFound(zip.to_string()))
    }

    /// Find ZIP code for given coordinates using point-in-polygon
    pub fn lat_lon_to_zip(&self, lat: f64, lon: f64) -> Result<String, ZipCodeError> {
        // Validate coordinates
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            return Err(ZipCodeError::InvalidCoordinates(lat, lon));
        }

        let point = point!(x: lon, y: lat);

        // Find all polygons whose bounding boxes contain the point
        let candidates: Vec<&ZipShape> = self
            .rtree
            .locate_in_envelope_intersecting(&rstar::AABB::from_point(point))
            .collect();

        // Check point-in-polygon for each candidate
        for polygon in candidates {
            if polygon.shape.contains(&geo::point!(x: lon, y: lat)) {
                return Ok(polygon.zip.clone());
            }
        }

        // Fallback: find nearest neighbor by centroid
        let nearest = self
            .rtree
            .nearest_neighbor_iter(&point)
            .next()
            .ok_or_else(|| ZipCodeError::DataLoadError("No ZIP codes in database".to_string()))?;

        Ok(nearest.zip.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_zip_format() {
        let db = ZipCodeDb::new().unwrap();
        assert!(db.zip_to_centroid("1234").is_err());
        assert!(db.zip_to_centroid("123456").is_err());
        assert!(db.zip_to_centroid("abcde").is_err());
    }

    #[test]
    fn test_invalid_coordinates() {
        let db = ZipCodeDb::new().unwrap();
        assert!(db.lat_lon_to_zip(91.0, 0.0).is_err());
        assert!(db.lat_lon_to_zip(0.0, 181.0).is_err());
        assert!(db.lat_lon_to_zip(-91.0, 0.0).is_err());
    }
}
