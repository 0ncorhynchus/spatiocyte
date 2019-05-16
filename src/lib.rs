// use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ParticleID(u64, u64);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Species(String);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Coordinate(usize);

pub struct HCPLatticeSize {
    row: usize,
    col: usize,
    layer: usize,
}

#[derive(Debug)]
pub enum Error {
    OutOfRange(Coordinate),
    ParticleNotFound(Coordinate),
    InvalidLocation(Coordinate, Coordinate),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SpeciesID(usize);

#[derive(Clone, PartialEq, Debug)]
enum TrackingType {
    Tracking(Vec<(ParticleID, Coordinate)>),
    Count(usize),
}

#[derive(Clone, PartialEq, Debug)]
struct SpeciesCache {
    species: Species,
    location: Option<SpeciesID>,
    cache: TrackingType,
}

impl SpeciesCache {
    fn remove(&mut self, coordinate: Coordinate) {
        match &mut self.cache {
            TrackingType::Tracking(cache) => {
                for i in 0..cache.len() {
                    if cache[i].1 == coordinate {
                        cache.remove(i);
                        break;
                    }
                }
            }
            TrackingType::Count(count) => {
                *count -= 1;
            }
        }
    }

    fn add(&mut self, coordinate: Coordinate) {
        match &mut self.cache {
            TrackingType::Tracking(cache) => {
                cache.push((ParticleID(0,0), coordinate));
            }
            TrackingType::Count(count) => {
                *count += 1;
            }
        }
    }

    fn move_to(&mut self, from: Coordinate, to: Coordinate) {
        if let TrackingType::Tracking(cache) = &mut self.cache {
            for (_pid, coordinate) in cache {
                if *coordinate == from {
                    *coordinate = to;
                    break;
                }
            }
        }
    }
}

pub struct HCPLatticeSpace {
    voxel_radius: f64,
    size: HCPLatticeSize,
    voxels: Box<[Option<SpeciesID>]>,
    species_cache: Vec<SpeciesCache>,
}

impl HCPLatticeSpace {
    pub fn new(voxel_radius: f64, size: HCPLatticeSize) -> Self {
        let num_voxels = size.row * size.col * size.layer;
        Self {
            voxel_radius,
            size,
            voxels: vec![None; num_voxels].into_boxed_slice(),
            species_cache: Vec::new(),
        }
    }

    pub fn get_voxel_radius(&self) -> f64 {
        self.voxel_radius
    }

    pub fn find_particle(&self, pid: ParticleID) -> Option<(&Species, Coordinate)> {
        for species in &self.species_cache {
            if let TrackingType::Tracking(cache) = &species.cache {
                for (id, coordinate) in cache {
                    if *id == pid {
                        return Some((&species.species, *coordinate));
                    }
                }
            }
        }
        None
    }

    fn get_species_id_at(&self, coordinate: Coordinate) -> Result<Option<SpeciesID>> {
        self.voxels
            .get(coordinate.0)
            .ok_or(Error::OutOfRange(coordinate))
            .map(|id| *id)
    }

    // fn get_species_cache(&self, id: SpeciesID) -> &SpeciesCache {
    //     &self.species_cache[id.0]
    // }

    fn get_species_cache_mut(&mut self, id: SpeciesID) -> &mut SpeciesCache {
        &mut self.species_cache[id.0]
    }

    pub fn move_particle(&mut self, from: Coordinate, to: Coordinate) -> Result<()> {
        let from_species_id = self
            .get_species_id_at(from)?
            .ok_or(Error::ParticleNotFound(from))?;
        let to_species_id = self.get_species_id_at(to)?;

        let from_species_cache = self.get_species_cache_mut(from_species_id);

        if from_species_cache.location != to_species_id {
            return Err(Error::InvalidLocation(from, to));
        }

        from_species_cache.move_to(from, to);

        if let Some(to_species_id) = to_species_id {
            let to_species_cache = self.get_species_cache_mut(to_species_id);
            to_species_cache.remove(to);
            to_species_cache.add(from);
        }

        self.voxels.swap(from.0, to.0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
