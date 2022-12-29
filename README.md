# onca-rs
Game engine in rust

# Building

## Windows

Windows requires a manifest to be added to the resulting .exe to use UTF-8 encoding, onca will otherwise panic, as it requires UTF-8 to run properly.

### VS Code

When using VS Code, use the included '`build onca`' task in '`tasks.json`'.

You can either run if manually through the command pallette using 
```
VS Code Command Pallette > Tasks: Run Task
```
and selecting '`build onca`' in the dropdown.

Or you can set up '`build onca`' as the configured build task, which can be 
done using:
```
VS Code Command Pallette > Tasks: Configure Default Build Task
```
and then using via the build task shortcut or using:
```
VS Code Command Pallette > Tasks: Build Task
```

### Other editors

when using an other editor, you must first build the project using 

```
cargo build
```
After `cargo` has finished building, you need to run the following or you won't be able to start the engine
```
cargo run -p onca_post_build -- <path-to-manifest>
```
The manifest is located in the root of the project

## Other OSes

```
cargo build
```

# Minimum hardware requirements

## Hard requirements

OS:
- Windows 10 64-bit update 1903 or higher

Archtecture
- x86-64

## Soft requirements

- x86-64 with AVX

---
**NOTE**

The minimum requirements are not planned to be lowered at any time in the future

---