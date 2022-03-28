use geo::prelude::*;
use geojson::GeoJson;
use rustler::ResourceArc;
use std::iter::FromIterator;

mod float64;
pub use float64::F64;

mod rtree_ex;
use rtree_ex::{GeomExWithData, GeomLookup, RTreeEx};

mod geom_ex;
use geom_ex::{GeomEx, Pt};

type Tree = ResourceArc<RTreeEx>;

rustler::atoms! {
    ok,
}

#[rustler::nif]
fn tree_new() -> ResourceArc<RTreeEx> {
    ResourceArc::new(RTreeEx::new())
}

#[rustler::nif(schedule = "DirtyCpu")]
fn tree_upsert_many(r_tree_ex: Tree, items: Vec<GeomExWithData>) -> Vec<GeomLookup> {
    let mut tree = r_tree_ex.write();
    tree.upsert_many(items)
}

#[rustler::nif]
fn tree_all_located_at(r_tree_ex: Tree, p: Pt) -> Vec<GeomExWithData> {
    let tree = r_tree_ex.read();
    tree.all_at_point(p)
}

#[rustler::nif]
fn tree_near(r_tree_ex: Tree, p: Pt, meters: F64) -> Vec<GeomExWithData> {
    let tree = r_tree_ex.read();
    tree.near(p, meters)
}

#[rustler::nif]
fn tree_lookup_many(r_tree_ex: Tree, ids: Vec<u64>) -> Vec<GeomLookup> {
    let tree = r_tree_ex.read();
    tree.lookup(&ids[..])
}

#[rustler::nif]
fn tree_remove_many(r_tree_ex: Tree, ids: Vec<u64>) -> Vec<GeomLookup> {
    let mut tree = r_tree_ex.write();
    tree.remove(&ids[..])
}

#[rustler::nif]
fn tree_intersects(r_tree_ex: Tree, geom_ex: GeomEx) -> Vec<GeomExWithData> {
    let tree = r_tree_ex.read();
    tree.intersects(&geom_ex)
}

#[rustler::nif]
fn geo_haversine_distance(points: Vec<Pt>) -> F64 {
    let ls: geo::LineString<F64> = geo::LineString::from_iter(points.into_iter().map(|p| p.inner));
    ls.haversine_length()
}

#[rustler::nif]
fn parse_latlon(text: String) -> Option<Pt> {
    latlon::parse(&text[..]).ok().map(|p| Pt::from(p))
}

#[rustler::nif]
fn parse_geojson(text: String) -> Option<GeomEx> {
    use geojson::Value as V;
    let geometry = match text.parse::<GeoJson>() {
        Ok(GeoJson::Geometry(g)) => g,
        _ => return None,
    };
    match geometry.value {
        V::Point(p) => {
            let inner = geo::Point::new(p[0].into(), p[1].into());
            Some(GeomEx::Pt(Pt { inner }))
        }
        // V::LineString(ls) => {
        //     let inner = ls.into_iter().map(|pos|)
        // }
        _ => None,
    }
}

pub fn load(env: rustler::Env, term: rustler::Term) -> bool {
    rtree_ex::load(env, term);
    true
}

rustler::init!(
    "Elixir.SoyGeo.Native",
    [
        tree_new,
        tree_upsert_many,
        tree_all_located_at,
        tree_near,
        tree_intersects,
        tree_lookup_many,
        tree_remove_many,
        geo_haversine_distance,
        parse_latlon,
        parse_geojson,
    ],
    load = load
);
// rustler::init!("Elixir.SoyGeo.Native", [tree_new], load = load);
