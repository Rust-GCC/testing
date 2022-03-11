#!/bin/sh

gccrs_ui_dir=/tmp/gccrs-ui
yml=gccrs-ui.yml.new

if [ "$1" = "prepare" ]; then
  mkdir $gccrs_ui_dir || echo "Skipping directory creation"

  for file in $(find src/test/ui/ -name '*.rs'); do
    printf "\rCopying file to location: $gccrs_ui_dir/$file"
    mkdir -p $gccrs_ui_dir/$(dirname $file)
    cp $file $gccrs_ui_dir/$file
  done;
fi;

echo "tests:" > $yml

for file in $(find $gccrs_ui_dir -name '*.rs'); do
  printf "\rAdding file to testsuite ($yml): $file"
  echo "  - name: Compile $file" >> $yml
  echo "    binary: ./build/gcc/rust1" >> $yml
  echo "    timeout: 5" >> $yml
  echo "    args:" >> $yml
  echo "      - \"$file\"" >> $yml
done;
