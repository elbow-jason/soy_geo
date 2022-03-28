alias SoyGeo.Tree

tree1 = Tree.new()
tree2 = Tree.new()
_ = Tree.upsert_many(tree2, [{1, {1.0, 1.0}}, {2, {2.0, 2.0}}])
tree3 = Tree.new()
_ = Tree.upsert_many(tree3, [{1, {{1.0, 1.0}, {2.0, 2.0}}}, {2, {{0.0, 1.0}, {2.0, 2.0}}}])
tree4 = Tree.new()

rand_x = fn -> :rand.uniform() end
rand_y = fn -> :rand.uniform() end
rand_id = fn -> :rand.uniform(1_000_000_000_000_000) end

rand_point = Stream.repeatedly(fn -> {rand_id.(), {rand_x.(), rand_y.()}} end)


items = Enum.take(rand_point, 100)
Enum.each(1..10, fn _ ->
  items2 = Enum.take(rand_point, 100)
  Tree.upsert_many(tree2, items2)
end)

benches = %{
  "upsert_many" => fn -> Tree.upsert_many(tree1, [{1, {1.0, 1.0}}, {2, {2.0, 2.0}}]) end,
  "all_located_at" => fn -> Tree.all_located_at(tree2, {1.0, 1.0}) end,
  "near" => fn -> Tree.near(tree3, {1.0, 1.0}, 1_000_000) end,
  "intersects" => fn -> Tree.intersects(tree3, [{0.0, 0.0}, {4.0, 4.0}]) end,
  "lookup_many" => fn -> Tree.lookup_many(tree2, [1, 2]) end,
  "upsert_many_100_rand" => fn -> Tree.upsert_many(tree4, items) end
}

# IO.puts("p == 1 ---------------------------------------------------------")
Benchee.run(benches, parallel: 1)
# IO.puts("p == 2 ---------------------------------------------------------")
# Benchee.run(benches, parallel: 2)
# IO.puts("p == 3 ---------------------------------------------------------")
# Benchee.run(benches, parallel: 4)
# IO.puts("fin ------------------------------------------------------------")
