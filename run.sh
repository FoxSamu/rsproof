cargo build -r

for entry in ./benchmark/*.in
do
  echo "==================================================================="
  echo "$entry"

  echo "-- prefer_empty ---------------------------------------------------"
  time ./target/release/rsproof prove -f "$entry" -H prefer_empty -s 5000 -v > "$entry.prefer_empty.out"
  echo ""
  echo "-- depth ----------------------------------------------------------"
  time ./target/release/rsproof prove -f "$entry" -H depth -s 5000 -v > "$entry.depth.out"
  echo ""
  echo "-- disjunct_count -------------------------------------------------"
  time ./target/release/rsproof prove -f "$entry" -H disjunct_count -s 5000 -v > "$entry.disjunct_count.out"
  echo ""
  echo "-- symbol_count ---------------------------------------------------"
  time ./target/release/rsproof prove -f "$entry" -H symbol_count -s 5000 -v > "$entry.symbol_count.out"
  echo ""
  echo "-- disjunct_count_plus_depth -------------------------------------------------"
  time ./target/release/rsproof prove -f "$entry" -H disjunct_count_plus_depth -s 5000 -v > "$entry.disjunct_count_plus_depth.out"
  echo ""
  echo "-- symbol_count_plus_depth ---------------------------------------------------"
  time ./target/release/rsproof prove -f "$entry" -H symbol_count_plus_depth -s 5000 -v > "$entry.symbol_count_plus_depth.out"
  echo ""
  echo "==================================================================="
done