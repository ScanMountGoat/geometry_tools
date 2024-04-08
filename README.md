# geometry_tools
[![Latest Version](https://img.shields.io/crates/v/geometry_tools.svg)](https://crates.io/crates/geometry_tools) [![docs.rs](https://docs.rs/geometry_tools/badge.svg)](https://docs.rs/geometry_tools)

This library provides efficient implementations for calculating normals, tangents, bitangents, and bounding data in Rust. The library depends on [glam](https://github.com/bitshifter/glam-rs) to utilize SIMD for the vector and matrix math on supported platforms.  Most functions support any type that can be converted into `glam::Vec3A`. This allows `glam::Vec3A` and `glam::Vec4` to have identical performance. Using `glam::Vec3` will have slightly reduced performance due to conversions to aligned types.
