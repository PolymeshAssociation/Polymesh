#!/bin/sh

grep -r 'storage_migration_ver!' pallets/ | \
	sed -e 's/.src.*!./: /g' -e 's/);//g' | \
	sort >/tmp/max_version.txt

grep -r StorageVersion pallets/ | grep new | grep -v "STORAGE_VERSION" | \
	sed -e 's/.src.*::new./: /g' -e 's/..: Version.*//g' -e 's/));//g' | \
	sort >/tmp/new_version.txt

diff /tmp/max_version.txt /tmp/new_version.txt || {
	echo "Failed version check"
	exit 1
}
