# onca-rs
Game engine in rust

# Building

The project can be built by using:

```
cargo build
```
---
**NOTE**

On windows, make sure you are using the MSVC build tools

---

# Minimum hardware requirements

## Hard requirements

OS:
- Windows 10 64-bit update 1909 or later

Archtecture
- x86-64

## Soft requirements

- x86-64 with AVX

## Minimum known supported GPUs

When using the software RAL, the GPU is not used.

- NVIDIA Turing 16 architecture of later
- AMD GNC 5 architecture of later
- INTEL Arc or later

For more detail about possible other supported hardware, check the readme of the RAL for required features

---
**NOTE**

The minimum requirements are not planned to be lowered at any time in the future

---

---
**NOTE**

At the moment, there is no plans to support platforms other than windows and linux (apple may be added in the future).

Mobile might be considered in the future, but has no plans, as supporting mobile, specifically when it comes to graphics, would either limit the available features or would require a lot more runtime checks and codepaths depending on the availability of features.

---