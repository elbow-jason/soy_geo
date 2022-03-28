// use geo::prelude::*;
use geo::{
    Coordinate, LineString as GeoLineString, Point as GeoPoint, Polygon as GeoPolygon,
    Rect as GeoRect,
};

use geo_types::private_utils::line_string_bounding_rect;

use rstar::primitives::Rectangle as RstarRect;
use rstar::{Point as RstarPoint, PointDistance, RTreeObject, AABB};
use rustler::{Decoder, Encoder, Env, NifResult, Term};
use rustler::{Error as NifError, ListIterator, NifUntaggedEnum};
use std::iter::FromIterator;

use crate::F64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pt {
    pub inner: GeoPoint<F64>,
}

impl<'a> Decoder<'a> for Pt {
    fn decode(term: Term<'a>) -> NifResult<Pt> {
        let (x, y): (F64, F64) = term.decode()?;
        if x > F64::from(180.0) {
            return Err(NifError::Term(Box::new(format!(
                "longitude was greater than 180.0 degrees: {:?}",
                x.0
            ))));
        }
        if x < F64::from(-180.0) {
            return Err(NifError::Term(Box::new(format!(
                "longitude was less than -180.0 degrees: {:?}",
                x.0
            ))));
        }
        if y > F64::from(90.0) {
            return Err(NifError::Term(Box::new(format!(
                "latitude was greater than 90.0 degrees: {:?}",
                y.0
            ))));
        }
        if y < F64::from(-90.0) {
            return Err(NifError::Term(Box::new(format!(
                "latitude was less than -90.0 degrees: {:?}",
                y.0
            ))));
        }
        let inner = GeoPoint::new(x, y);
        Ok(Pt { inner })
    }
}

impl Encoder for Pt {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        (self.inner.x(), self.inner.y()).encode(env)
    }
}

impl RstarPoint for Pt {
    type Scalar = F64;
    const DIMENSIONS: usize = 2;

    fn generate(generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        let inner = GeoPoint::<F64>::generate(generator);
        Pt { inner }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        self.inner.nth(index)
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        self.inner.nth_mut(index)
    }
}

// impl Eq for Pt {}

impl From<Pt> for Coordinate<F64> {
    fn from(p: Pt) -> Coordinate<F64> {
        p.inner.into()
    }
}

impl From<&Pt> for Coordinate<F64> {
    fn from(p: &Pt) -> Coordinate<F64> {
        p.inner.into()
    }
}

impl From<Coordinate<F64>> for Pt {
    fn from(c: Coordinate<F64>) -> Pt {
        Pt { inner: c.into() }
    }
}

impl From<GeoPoint<f64>> for Pt {
    fn from(p: GeoPoint<f64>) -> Pt {
        let inner = GeoPoint::new(p.x().into(), p.y().into());
        Pt { inner }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RectEx {
    inner: RstarRect<Pt>,
}

impl RectEx {
    fn from_coords(min: Coordinate<F64>, max: Coordinate<F64>) -> RectEx {
        RectEx {
            inner: RstarRect::from_corners(min.into(), max.into()),
        }
    }
}

impl From<RstarRect<Pt>> for RectEx {
    fn from(inner: RstarRect<Pt>) -> RectEx {
        RectEx { inner }
    }
}

// impl From<&Rect> for RstarRect<Pt> {
//     fn from(r: &Rect) -> RstarRect<Pt> {
//         r.inner.clone()
//     }
// }

impl From<RectEx> for RstarRect<Pt> {
    fn from(r: RectEx) -> RstarRect<Pt> {
        r.inner
    }
}

impl From<&RectEx> for RstarRect<Pt> {
    fn from(r: &RectEx) -> RstarRect<Pt> {
        r.inner.clone()
    }
}

impl RTreeObject for RectEx {
    type Envelope = AABB<Pt>;

    fn envelope(&self) -> Self::Envelope {
        self.inner.envelope()
    }
}

impl<'a> Decoder<'a> for RectEx {
    fn decode(term: Term<'a>) -> NifResult<RectEx> {
        let (p1, p2): (Pt, Pt) = term.decode()?;
        let inner = RstarRect::from_corners(p1, p2);
        Ok(RectEx { inner })
    }
}

impl Encoder for RectEx {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        (self.inner.lower(), self.inner.upper()).encode(env)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineStringEx {
    pub inner: GeoLineString<F64>,
}

impl<'a> Decoder<'a> for LineStringEx {
    fn decode(term: Term<'a>) -> NifResult<LineStringEx> {
        let it: ListIterator = term.decode()?;
        let mut coords = Vec::new();
        for item in it {
            let pt: Pt = item.decode()?;
            coords.push(Coordinate::from(pt.inner))
        }
        if coords.is_empty() {
            return Err(NifError::Term(Box::new(format!(
                "line_string cannot be empty"
            ))));
        }
        let inner = GeoLineString::from_iter(coords.into_iter());
        Ok(LineStringEx { inner })
    }
}

impl Encoder for LineStringEx {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let points: Vec<Pt> = self.inner.0.iter().map(|c| Pt::from(*c)).collect();
        points.encode(env)
    }
}

impl RTreeObject for LineStringEx {
    type Envelope = AABB<Pt>;

    fn envelope(&self) -> Self::Envelope {
        linestring_to_aabb_pt(&self.inner)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolygonEx {
    inner: GeoPolygon<F64>,
}

impl Encoder for PolygonEx {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let mut out: Vec<Vec<Pt>> = Vec::with_capacity(self.inner.interiors().len() + 1);
        out.push(linestring_to_vec_pt(self.inner.exterior()));
        for shape in self.inner.interiors() {
            out.push(linestring_to_vec_pt(shape));
        }
        out.encode(env)
    }
}

impl<'a> Decoder<'a> for PolygonEx {
    fn decode(term: Term<'a>) -> NifResult<PolygonEx> {
        let lines: Vec<Vec<Pt>> = term.decode()?;
        if lines.is_empty() {
            return Err(NifError::Atom("polygon lines cannot be empty"));
        }
        if lines[0].is_empty() {
            return Err(NifError::Atom("polygon exterior cannot be empty"));
        }
        let exterior = pts_to_linestring(&lines[0][..]);
        let interiors = lines[1..]
            .iter()
            .map(|pts_vec| pts_to_linestring(&pts_vec[..]))
            .collect();
        let inner = GeoPolygon::new(exterior, interiors);

        Ok(PolygonEx { inner })
    }
}

impl RTreeObject for PolygonEx {
    type Envelope = AABB<Pt>;

    fn envelope(&self) -> Self::Envelope {
        linestring_to_aabb_pt(self.inner.exterior())
    }
}

fn pts_to_linestring(pts: &[Pt]) -> GeoLineString<F64> {
    GeoLineString::from_iter(pts.iter().map(|pt| Coordinate::from(pt.inner)))
}
fn linestring_to_vec_pt(ls: &GeoLineString<F64>) -> Vec<Pt> {
    ls.coords().map(|c| Pt::from(*c)).collect()
}

fn linestring_to_aabb_pt(ls: &GeoLineString<F64>) -> AABB<Pt> {
    let grect: GeoRect<F64> = line_string_bounding_rect(ls).unwrap();
    let min: Coordinate<F64> = grect.min();
    let max: Coordinate<F64> = grect.max();
    RectEx::from_coords(min, max).envelope()
}

#[derive(NifUntaggedEnum, Debug, Clone, PartialEq, Eq)]
pub enum GeomEx {
    Pt(Pt),
    Rect(RectEx),
    LineString(LineStringEx),
    Polygon(PolygonEx),
}

impl RTreeObject for GeomEx {
    type Envelope = AABB<Pt>;

    fn envelope(&self) -> Self::Envelope {
        match self {
            GeomEx::Pt(p) => p.envelope(),
            GeomEx::Rect(r) => r.envelope(),
            GeomEx::LineString(ls) => ls.envelope(),
            GeomEx::Polygon(p) => p.envelope(),
        }
    }
}

impl PointDistance for GeomEx {
    fn distance_2(&self, pt: &Pt) -> F64 {
        match self {
            GeomEx::Pt(p) => p.distance_2(pt),
            GeomEx::Rect(r) => r.envelope().distance_2(pt),
            GeomEx::LineString(ls) => ls.envelope().distance_2(pt),
            GeomEx::Polygon(p) => p.envelope().distance_2(pt),
        }
    }
}
