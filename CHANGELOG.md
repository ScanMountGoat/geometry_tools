# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.6.0 - 2024-07-04
### Changed
* Updated glam.
* Changed `calculate_tangents` and `calculate_tangents_bitangents` to be generic over the index type.
* Changed functions to be generic over vector types that can be converted to `glam::Vec3A`.

## 0.5.0 - 2024-02-18
### Changed
* Updated glam.

## 0.4.2 - 2022-12-28
### Changed
* Updated glam.

## 0.4.1 - 2022-08-12
### Changed
* Updated glam.

## 0.4.0 - 2022-02-11
### Changed
* Updated glam.

## 0.3.1 - 2022-01-02
### Added
* Added function `calculate_bounding_sphere_from_spheres`.

## 0.3.0 - 2021-09-05
### Added
* Added `calculate_tangents` for calculating tangents with bitangent sign.
* Added `calculate_tangents_bitangents` for calculating tangent and bitangent vectors.

## 0.2.0 - 2021-05-31
### Added
* Added `calculate_bounding_sphere_from_points` for calculating bounds from points.
* Added `calculate_aabb_from_points` for calculating axis-aligned bounding boxes.

## 0.1.0 - 2021-05-31
First public release.