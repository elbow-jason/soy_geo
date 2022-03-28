defmodule SoyGeo.DistanceTest do
  use ExUnit.Case

  alias SoyGeo.Distance

  describe "haversine_distance" do
    test "calculates pole to pole distance" do
      north_pole = {0.0, 90.0}
      south_pole = {0.0, -90.0}
      meters = Distance.haversine([north_pole, south_pole])
      # ~20020
      assert meters == 20_015_114.442035925
    end
  end
end
