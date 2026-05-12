#!/bin/sh

sed -i \
	-e 's/pallet_checkpoint::/pallet_asset::checkpoint::/g' \
	-e 's/pallet_capital_distribution::/pallet_corporate_actions::distribution::/g' \
	-e 's/pallet_corporate_ballot::/pallet_corporate_actions::ballot::/g' \
	./pallets/weights/src/pallet_*.rs

