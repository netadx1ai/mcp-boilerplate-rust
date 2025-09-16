# MCP Boilerplate Rust - MVP Specification

## 1. Project Goal

To implement a functional Minimum Viable Product (MVP) of the MCP boilerplate template in Rust. The MVP will serve as a "vertical slice," demonstrating an end-to-end working application with both simple and advanced use cases.

## 2. Scope

The MVP will focus on the foundational components and a representative set of examples.

### In Scope:

-   **`mcp-core` Library**:
    -   Core protocol message types: `McpRequest`, `McpResponse`, `McpError`.
    -   Core traits: `McpServer`, `McpTool`.
-   **`mcp-transport` Library**:
    -   **STDIO** (Standard Input/Output) transport.
    -   **HTTP** transport (RESTful with JSON payloads) using a library like `axum` or `actix-web`.
-   **`mcp-server` Library**:
    -   A basic server runner that can host different transport layers.
-   **Example Implementations**:
    -   **`filesystem`**: A simple server with a `read_file(path: String)` tool.
    -   **`image-generation`**: Scaffolding for an AI image generation server.
    -   **`blog-generation`**: Scaffolding for an AI blog post creation server.
    -   **`creative-content`**: Scaffolding for a combined creative content server.
    -   *(Note: The AI examples will be scaffolded with placeholder logic, demonstrating the structure without requiring full AI model integration for the MVP).*

### Out of Scope for MVP:

-   WebSocket or direct TCP transports.
-   Advanced server features (middleware, comprehensive configuration files, plugins).
-   Full implementation of AI logic; placeholder functions will be used.
-   The `mcp-client` library.
-   The full `mcp-cli` tool.
-   Comprehensive test suites and documentation beyond a basic README.

## 3. Architecture

The project will be structured as a Rust workspace containing the following crates:

```
mcp-boilerplate-rust/
├── Cargo.toml         # Workspace configuration
├── mcp-core/          # Core MCP protocol implementation
├── mcp-transport/     # Transport layer implementations (STDIO, HTTP)
├── mcp-server/        # Basic server framework
└── examples/
    ├── filesystem/        # File operations server
    ├── image-generation/  # AI image generation server (scaffold)
    ├── blog-generation/   # AI blog post creation server (scaffold)
    └── creative-content/  # Combined creative content server (scaffold)
```

## 4. Success Criteria

By the end of the implementation session, the project must meet the following criteria:

1.  The project compiles successfully using `cargo build`.
2.  The `filesystem` example is runnable via both STDIO and HTTP transports.
3.  The `filesystem` server correctly processes a `read_file` request and returns the file's contents or an error.
4.  The AI example servers (`image-generation`, `blog-generation`, `creative-content`) are runnable.
5.  When an AI example server is called via HTTP, it returns a valid, hardcoded placeholder response, demonstrating that the routing and server structure are working.