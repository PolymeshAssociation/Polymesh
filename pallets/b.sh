dirs=(*)
for dir in "${dirs[@]}"; do
  cd "$dir"
  echo $PWD
  cd ..
done
