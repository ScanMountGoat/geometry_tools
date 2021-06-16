# geometry_tools
This library provides efficient implementations for calculating normals, tangents, bitangents, and bounding data in Rust. 
The library depends on [glam](https://github.com/bitshifter/glam-rs) to utilize SIMD for the vector and matrix math on supported platforms.
Only single precision floating point values are currently supported since the emphasis is on generating vertex and mesh data for game assets.
