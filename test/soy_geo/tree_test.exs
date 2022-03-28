defmodule SoyGeo.TreeTest do
  use ExUnit.Case

  alias SoyGeo.Tree
  require SoyGeo.Tree

  describe "is_point/1" do
    test "is true for {float(), float()} tuple" do
      assert Tree.is_point({1.0, 1.0}) == true
    end

    test "errors for non-points" do
      assert_raise(ArgumentError, fn ->
        # trick the compiler to stop warnings
        val = Enum.random([1.0, 1])
        Tree.is_point(val)
      end)
    end
  end

  describe "new/0" do
    test "returns a reference" do
      assert is_reference(Tree.new())
    end
  end

  describe "upsert_many/2" do
    test "returns previous id_geoms for a list of {integer(), point()} geoms" do
      tree = Tree.new()
      many1 = [{1, {2.0, 2.0}}, {2, {3.0, 3.0}}]
      many2 = [{1, {5.0, 5.0}}, {2, {5.0, 5.0}}]
      assert [{1, nil}, {2, nil}] == Tree.upsert_many(tree, many1)
      assert many1 == Tree.upsert_many(tree, many2)
    end

    test "returns previous id_geoms for a list of {integer(), rect()} geoms" do
      tree = Tree.new()
      many1 = [{1, {{2.0, 2.0}, {3.0, 3.0}}}]
      many2 = [{1, {{5.0, 5.0}, {5.0, 5.0}}}]
      assert [{1, nil}] == Tree.upsert_many(tree, many1)
      assert many1 == Tree.upsert_many(tree, many2)
    end

    test "returns previous id_geoms for a list of {integer(), line_string()} geoms" do
      tree = Tree.new()
      many1 = [{1, [{2.0, 2.0}, {3.0, 3.0}]}]
      many2 = [{1, [{5.0, 5.0}, {5.0, 5.0}]}]
      assert [{1, nil}] == Tree.upsert_many(tree, many1)
      assert many1 == Tree.upsert_many(tree, many2)
    end

    test "returns previous id_groms for a list of {integer(), polygon()} geoms" do
      tree = Tree.new()
      many1 = [{1, [[{2.0, 2.0}, {3.0, 10.0}, {4.0, 4.0}, {2.0, 2.0}]]}]
      many2 = [{1, [[{5.0, 5.0}, {3.0, 10.0}, {4.0, 4.0}, {5.0, 5.0}]]}]
      assert [{1, nil}] == Tree.upsert_many(tree, many1)
      assert many1 == Tree.upsert_many(tree, many2)
    end

    test "the previous version of an unclosed polygon is closed" do
      tree = Tree.new()
      many1 = [{1, [[{2.0, 2.0}, {3.0, 10.0}, {4.0, 4.0}]]}]
      closed1 = [{1, [[{2.0, 2.0}, {3.0, 10.0}, {4.0, 4.0}, {2.0, 2.0}]]}]
      many2 = [{1, [[{5.0, 5.0}, {3.0, 10.0}, {4.0, 4.0}]]}]
      assert [{1, nil}] == Tree.upsert_many(tree, many1)
      assert closed1 == Tree.upsert_many(tree, many2)
    end
  end

  defp some_id_points do
    [
      {1.0, 1.0},
      {1.1, 1.0},
      {1.0, 1.1},
      {1.1, 1.1},
      {{2.0, 2.0}, {2.0, 2.4}},
      {6.0, 6.0},
      {6.1, 6.0},
      {6.0, 6.1}
    ]
    |> Enum.with_index(1)
    |> Enum.map(fn {pt, id} -> {id, pt} end)
  end

  describe "near/3" do
    test "returns items that are close to the given point" do
      tree = Tree.new()
      points = some_id_points()
      _prevs = Tree.upsert_many(tree, points)
      # 1cm away
      assert Tree.near(tree, {1.1, 1.1000009}, 0.01) == []
      # 10m away
      assert Tree.near(tree, {1.1, 1.1000009}, 10.0) == [{4, {1.1, 1.1}}]
      # 100m away
      assert Tree.near(tree, {1.1, 1.1000009}, 100.0) == [{4, {1.1, 1.1}}]
      # 1km away
      assert Tree.near(tree, {1.1, 1.1000009}, 1000.0) == [{4, {1.1, 1.1}}]
      # 10km away
      assert Tree.near(tree, {1.1, 1.1000009}, 10000.0) == [{4, {1.1, 1.1}}]
      # 100km away
      assert Tree.near(tree, {1.1, 1.1000009}, 100_000.0) == [
               {4, {1.1, 1.1}},
               {2, {1.1, 1.0}},
               {3, {1.0, 1.1}},
               {1, {1.0, 1.0}}
             ]

      # 1000km away (1 million meters)
      assert Tree.near(tree, {1.1, 1.1000009}, 1_000_000.0) == [
               {8, {6.0, 6.1}},
               {7, {6.1, 6.0}},
               {6, {6.0, 6.0}},
               {5, {{2.0, 2.0}, {2.0, 2.4}}},
               {4, {1.1, 1.1}},
               {2, {1.1, 1.0}},
               {3, {1.0, 1.1}},
               {1, {1.0, 1.0}}
             ]
    end
  end

  describe "intersects" do
    test "returns the rect for a point on the edge of a rect" do
      tree = Tree.new()
      rect = {{0.0, 0.0}, {1.0, 1.0}}
      _ = Tree.upsert_many(tree, [{100, rect}])
      point = {0.5, 0.0}
      assert [{100, rect}] == Tree.intersects(tree, point)
    end

    test "returns the rect for a point inside the rect" do
      tree = Tree.new()
      rect = {{0.0, 0.0}, {1.0, 1.0}}
      _ = Tree.upsert_many(tree, [{100, rect}])
      point = {0.5, 0.5}
      assert [{100, rect}] == Tree.intersects(tree, point)
    end

    test "returns an overlapping rect" do
      tree = Tree.new()
      rect1 = {{0.0, 0.0}, {1.0, 1.0}}
      _ = Tree.upsert_many(tree, [{100, rect1}])
      rect2 = {{0.5, 0.5}, {1.5, 1.5}}
      assert [{100, rect1}] == Tree.intersects(tree, rect2)
    end

    test "returns crossing linestrings" do
      tree = Tree.new()
      # line1 and line2 make an X shape
      line1 = [{0.0, 0.0}, {1.0, 1.0}]
      line2 = [{1.0, 0.0}, {0.0, 1.0}]
      _ = Tree.upsert_many(tree, [{100, line1}])
      assert [{100, line1}] == Tree.intersects(tree, line2)
    end

    test "returns polygons that touch at a point" do
      tree = Tree.new()
      poly1 = [[{0.0, 0.0}, {1.0, 1.0}, {1.0, 0.0}, {0.0, 0.0}]]
      poly2 = [[{1.0, 1.0}, {2.2, 2.0}, {2.0, 1.0}, {1.0, 1.0}]]
      _ = Tree.upsert_many(tree, [{100, poly1}])
      assert [{100, poly1}] == Tree.intersects(tree, poly2)
    end

    test "returns an empty list when there are no intersecting geoms" do
      tree = Tree.new()
      rect1 = {{0.0, 0.0}, {1.0, 1.0}}
      _ = Tree.upsert_many(tree, [{100, rect1}])
      rect2 = {{5.0, 5.0}, {6.0, 6.0}}
      assert [] == Tree.intersects(tree, rect2)
    end
  end

  describe "lookup_many" do
    test "returns the associated geoms for the given ids" do
      poly1 = [[{0.0, 0.0}, {1.0, 1.0}, {1.0, 0.0}, {0.0, 0.0}]]
      poly2 = [[{1.0, 1.0}, {2.2, 2.0}, {2.0, 1.0}, {1.0, 1.0}]]
      tree = Tree.new()
      _ = Tree.upsert_many(tree, [{100, poly1}, {200, poly2}])
      assert [{100, poly1}, {200, poly2}] == Tree.lookup_many(tree, [100, 200])
    end

    test "returns {id, nil} for a missing id" do
      poly1 = [[{0.0, 0.0}, {1.0, 1.0}, {1.0, 0.0}, {0.0, 0.0}]]
      poly2 = [[{1.0, 1.0}, {2.2, 2.0}, {2.0, 1.0}, {1.0, 1.0}]]
      tree = Tree.new()
      _ = Tree.upsert_many(tree, [{100, poly1}, {200, poly2}])
      assert [{100, poly1}, {200, poly2}, {300, nil}] == Tree.lookup_many(tree, [100, 200, 300])
    end
  end

  describe "remove_many/2" do
    test "removes and returns given ids" do
      poly1 = [[{0.0, 0.0}, {1.0, 1.0}, {1.0, 0.0}, {0.0, 0.0}]]
      poly2 = [[{1.0, 1.0}, {2.2, 2.0}, {2.0, 1.0}, {1.0, 1.0}]]
      poly3 = [[{1.0, 1.1}, {2.2, 2.1}, {2.0, 1.1}, {1.0, 1.1}]]
      tree = Tree.new()
      _ = Tree.upsert_many(tree, [{100, poly1}, {200, poly2}, {300, poly3}])
      assert [{100, poly1}, {200, poly2}, {300, poly3}] == Tree.lookup_many(tree, [100, 200, 300])
      assert [{200, poly2}, {100, poly1}] == Tree.remove_many(tree, [200, 100])
      assert [{100, nil}, {200, nil}, {300, poly3}] == Tree.lookup_many(tree, [100, 200, 300])
      assert [{300, poly3}] == Tree.remove_many(tree, [300])
      assert [{100, nil}, {200, nil}, {300, nil}] == Tree.lookup_many(tree, [100, 200, 300])
    end
  end
end
