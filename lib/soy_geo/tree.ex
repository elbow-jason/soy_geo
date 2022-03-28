defmodule SoyGeo.Tree do
  alias SoyGeo.Native

  defguard is_point(p) when tuple_size(p) == 2 and is_float(elem(p, 0)) and is_float(elem(p, 1))

  def new do
    Native.tree_new()
  end

  def upsert_many(tree, id_geoms) when is_list(id_geoms) do
    Native.tree_upsert_many(tree, id_geoms)
  end

  def all_located_at(tree, point) when is_point(point) do
    Native.tree_all_located_at(tree, point)
  end

  def near(tree, point, meters) when is_point(point) and is_number(meters) and meters >= 0 do
    Native.tree_near(tree, point, meters * 1.0)
  end

  def intersects(tree, geom) do
    Native.tree_intersects(tree, geom)
  end

  def lookup_many(tree, ids) when is_list(ids) do
    Native.tree_lookup_many(tree, ids)
  end

  def remove_many(tree, ids) when is_list(ids) do
    Native.tree_remove_many(tree, ids)
  end
end
