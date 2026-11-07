# // ./make-plugin.sh name should take target/debug/lib[name].so and plugins/[name] and zip them together
# // -r is for release, then we take in target/release/lib[name].so
# // the name of the zip have _debug or _release suffix

#!/bin/bash

name=$1

if [ "$2" = "-r" ]; then
    suffix="release"
else
    suffix="debug"
fi

so_file="target/${suffix}/lib${name}.so"
toml_file="plugins/${name}/lib${name}.toml"
zip_file="plugins/${name}_${suffix}.zip"

if [ ! -f "$so_file" ]; then
    echo "Error: $so_file not found"
    exit 1
fi

zip -j "$zip_file" "$so_file" "$toml_file"
echo "Created $zip_file"
