# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

File system browser web application built with Dioxus. Uses the File System Access API to browse local directories in the browser with a tree-view interface.

## use third-party crates
- always use context7 mcp for get information of third-party crates **[IMPORTANT]**

## Development Commands

```bash
# build app
dx build
```

## Architecture

Single-file application (`src/main.rs`) with these components:

- **App**: Root component with document setup
- **FileSystemBrowser**: Main component with folder selection and tree display
- **FileSystemEntry**: Recursive data structure for files/directories
- **File System API**: JavaScript interop via `wasm-bindgen` and `web-sys`

## Key Implementation Notes

- Uses `showDirectoryPicker()` JavaScript API through WASM bindings
- Custom async directory iteration with JavaScript reflection
- Dioxus signals for state management (folder selection, file structure)
- Sorts directories first, then alphabetically
- Error handling for file system permissions and operations

## Project Structure

```
src/main.rs          # Complete application logic
assets/              # Static assets (CSS, icons)
Dioxus.toml         # Dioxus platform configuration
clippy.toml         # Custom lint rules for Dioxus signals
```

## Clippy Rules

Custom rules prevent holding Dioxus signal borrows across await points to avoid runtime panics.
