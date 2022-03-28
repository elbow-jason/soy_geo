defmodule SoyGeo.Native do
  use Rustler, otp_app: :soy_geo, crate: "soy_geo_native"

  @type point() :: {float, float}
  @type rect() :: {{float, float}, {float, float}}
  @type line_string() :: [point]
  @type polygon() :: [[point]]

  @type geom() :: point() | rect() | line_string() | polygon()
  @type id_geom() :: {non_neg_integer(), geom()}

  defp err, do: :erlang.nif_error(:soy_geo_nif_not_loaded)

  # When your NIF is loaded, it will override this function.
  def tree_new(), do: err()
  def tree_upsert_many(_tree, _id_geoms), do: err()
  def tree_all_located_at(_tree, _point), do: err()
  def tree_near(_tree, _point, _meters), do: err()
  def tree_lookup_many(_tree, _ids), do: err()
  def tree_remove_many(_tree, _ids), do: err()
  def tree_intersects(_tree, _geom), do: err()

  # geo calculations
  def geo_haversine_distance(_points), do: err()

  # parsing
  def parse_latlon(_text), do: err()
  def parse_geojson(_text), do: err()
end
