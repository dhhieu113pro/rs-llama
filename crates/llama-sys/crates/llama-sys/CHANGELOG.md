# Changelog

## [0.2.0](https://github.com/dhhieu113pro/rs-llama/compare/rs-llama-sys-v0.1.2...rs-llama-sys-v0.2.0) (2026-08-25)


### Features

* auto-detect llama.cpp GPU backend ([98bfd62](https://github.com/dhhieu113pro/rs-llama/commit/98bfd622f6628e4eb74e97a3a7c751ed858375a1))
* auto-select llama.cpp GPU backend ([1b888ea](https://github.com/dhhieu113pro/rs-llama/commit/1b888ea7cc403d690ce28d1f1ed07f1481897a8b))
* define Vulkan toolchain readiness policy ([efc72b6](https://github.com/dhhieu113pro/rs-llama/commit/efc72b6b5ea6e4f1d6b29a01aa11df22f08e9294))
* expose compiled backend metadata ([96e36fc](https://github.com/dhhieu113pro/rs-llama/commit/96e36fc5510af9767d4d3535e49ac768d263d108))
* implement backend selection policy ([43dc52a](https://github.com/dhhieu113pro/rs-llama/commit/43dc52ad2ceb8f903695e130df741bea3ee3e5c4))


### Bug Fixes

* bump version to 0.1.2 for crates.io publish ([d9d994d](https://github.com/dhhieu113pro/rs-llama/commit/d9d994d42c324bf9b6becc1e4675e0a47bdf6493))
* bump version to 0.1.2 for crates.io publish ([61354c9](https://github.com/dhhieu113pro/rs-llama/commit/61354c98b55abaf792edfd83aecca5a2018fe113))
* disable OpenMP on Windows to fix ARM64 linking ([eefbb38](https://github.com/dhhieu113pro/rs-llama/commit/eefbb382e68cfe758c99fba8ed54a04c8dfce9d8))
* disable OpenMP on Windows to fix ARM64 linking ([855fc2a](https://github.com/dhhieu113pro/rs-llama/commit/855fc2a011a082164caacefefb2a4f803f07d3c4))
* require complete Vulkan toolchain for auto selection ([3c96381](https://github.com/dhhieu113pro/rs-llama/commit/3c96381a15d33c5566db5f773b261408229fbbd7))
* use detected GPU SDK paths for linking ([35e6dc5](https://github.com/dhhieu113pro/rs-llama/commit/35e6dc549bf7184753423dc86855ab0c548c29a3))
* **windows:** pass MSVC include paths to bindgen (stdbool.h not found) ([ee381a5](https://github.com/dhhieu113pro/rs-llama/commit/ee381a5331bbc4bf4ef6e365c6eb45f36dda85a6))
