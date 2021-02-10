# Migration Tests

This test suite is used to test the storage migrations in runtime upgrades.

## Notes

- The old pallets in the Cargo.toml should point to last release.
- If `CACHE` file is present in this directory, the tests use that as the state or else they download Alcyone state from a remote node.
- If you add any storage migration in runtime upgrade, you should add a corresponding test here.
