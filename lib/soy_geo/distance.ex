defmodule SoyGeo.Distance do
  alias SoyGeo.Native

  def haversine(points) when is_list(points) do
    Native.geo_haversine_distance(points)
  end
end
