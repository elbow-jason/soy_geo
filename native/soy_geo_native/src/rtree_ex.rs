use geo::prelude::*;
use hashbrown::HashMap;
use rstar::primitives::GeomWithData;
use rstar::{RTree as GenRTree, RTreeObject};
use rustler::{Decoder, Encoder, Env, NifResult, Term};
use rustler::{NifTuple, NifUntaggedEnum};

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{GeomEx, Pt, F64};

#[derive(NifTuple, Debug, Clone, PartialEq, Eq)]
pub struct GeomExWithData {
    data: u64,
    geom: GeomEx,
}

impl GeomExWithData {
    fn new(data: u64, geom: GeomEx) -> Self {
        GeomExWithData { data, geom }
    }
}

impl From<&TreeGeomWithData> for GeomExWithData {
    fn from(tree_geom: &TreeGeomWithData) -> GeomExWithData {
        GeomExWithData::new(tree_geom.data, tree_geom.geom().clone())
    }
}

impl From<TreeGeomWithData> for GeomExWithData {
    fn from(tree_geom: TreeGeomWithData) -> GeomExWithData {
        GeomExWithData::new(tree_geom.data, GeomEx::from(tree_geom.geom().clone()))
    }
}

impl From<GeomExWithData> for TreeGeomWithData {
    fn from(g: GeomExWithData) -> TreeGeomWithData {
        TreeGeomWithData::new(g.geom.into(), g.data)
    }
}

type TreeGeomWithData = GeomWithData<GeomEx, u64>;
type RTree = GenRTree<TreeGeomWithData>;

pub struct RTreeEx {
    inner: RwLock<RTreeInner>,
}

impl RTreeEx {
    pub fn new() -> RTreeEx {
        RTreeEx {
            inner: RwLock::new(RTreeInner::new()),
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<RTreeInner> {
        self.inner.write().unwrap()
    }

    pub fn read(&self) -> RwLockReadGuard<RTreeInner> {
        self.inner.read().unwrap()
    }
}

unsafe impl Sync for RTreeEx {}
unsafe impl Send for RTreeEx {}

pub struct MissingGeom(u64);

rustler::atoms! {
    nil,
}

impl Encoder for MissingGeom {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        (self.0, nil()).encode(env)
    }
}

impl<'a> Decoder<'a> for MissingGeom {
    fn decode(_term: Term<'a>) -> NifResult<MissingGeom> {
        panic!("missing geom should not be decoded")
    }
}

#[derive(NifUntaggedEnum)]
pub enum GeomLookup {
    Missing(MissingGeom),
    Found(GeomExWithData),
}

pub struct RTreeInner {
    _map: HashMap<u64, GeomEx>,
    _rtree: RTree,
}

impl RTreeInner {
    fn new() -> RTreeInner {
        RTreeInner {
            _map: HashMap::new(),
            _rtree: RTree::new(),
        }
    }

    pub fn upsert_many(&mut self, items: Vec<GeomExWithData>) -> Vec<GeomLookup> {
        let mut prevs = Vec::with_capacity(items.len());
        for item in items {
            let (id, new_geom) = (item.data.clone(), item.geom.clone());
            let new_geom_with_data = item.into();
            let prev = self.remove_id(id);
            prevs.push(prev);
            self._rtree.insert(new_geom_with_data);
            self._map.insert(id, new_geom.into());
        }
        prevs
    }

    pub fn all_at_point(&self, pt: Pt) -> Vec<GeomExWithData> {
        let point = pt.into();
        self._rtree
            .locate_all_at_point(&point)
            .map(GeomExWithData::from)
            .collect()
    }

    pub fn near(&self, pt: Pt, meters: F64) -> Vec<GeomExWithData> {
        let other_pt = pt.inner.haversine_destination(F64::from(45.0), meters);
        let distance = pt.inner.envelope().distance_2(&other_pt);
        self._rtree
            .locate_within_distance(pt, distance)
            .map(GeomExWithData::from)
            .collect()
    }

    pub fn intersects(&self, geom_ex: &GeomEx) -> Vec<GeomExWithData> {
        let envelope = geom_ex.envelope();
        self._rtree
            .locate_in_envelope_intersecting(&envelope)
            .map(GeomExWithData::from)
            .collect()
    }

    pub fn lookup(&self, ids: &[u64]) -> Vec<GeomLookup> {
        ids.iter()
            .map(|id| match self._map.get(id) {
                Some(geom) => GeomLookup::Found(GeomExWithData::new(*id, geom.clone().into())),
                None => GeomLookup::Missing(MissingGeom(*id)),
            })
            .collect()
    }

    pub fn remove(&mut self, ids: &[u64]) -> Vec<GeomLookup> {
        let mut prevs = Vec::with_capacity(ids.len());
        for id in ids {
            let entry = self.remove_id(*id);
            prevs.push(entry);
        }
        prevs
    }

    fn remove_id(&mut self, id: u64) -> GeomLookup {
        match self._map.remove(&id) {
            Some(geom) => {
                let geom_query = GeomWithData::new(geom, id);
                let prev = self
                    ._rtree
                    .remove(&geom_query)
                    .expect("geom was in tree map, but not rtree");
                GeomLookup::Found(GeomExWithData::from(prev))
            }
            None => GeomLookup::Missing(MissingGeom(id)),
        }
    }
}

pub fn load(env: rustler::Env, _: rustler::Term) -> bool {
    rustler::resource!(RTreeEx, env);
    true
}
