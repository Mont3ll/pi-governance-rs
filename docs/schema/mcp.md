# MCP Response Schema

Schema is release-candidate level and intended to stabilize at v1.0.0.

MCP tool responses include text `content`, object-shaped `structuredContent`, and `isError`. List-style tools return objects with named arrays and `count`, for example `{ "records": [], "count": 0 }`.

The MCP server default namespace comes from `pi --namespace <name> mcp-stdio`; tool arguments may override namespace where supported.
