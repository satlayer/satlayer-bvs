# SatLayer Developer Docs

Developer docs for SatLayer, for developers and operators running Bitcoin Validated Services.

## How are the docs organized?

All BVS documentations are placed under the `./docs` folder in this mono-repo.
This allows for easy access and editing of the documentation contextual to the source code.

As some documentation is best co-located with the source code (README.md files),
—we link them by using symbolic links: `ln -s ../../README.md page.md`—
to ensure that the documentation is always up to date, especially when the code changes.
