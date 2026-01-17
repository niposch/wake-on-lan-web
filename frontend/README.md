# 🎨 WoL Frontend

Modern React UI built with performance and aesthetics in mind.

## ⚡ Tech Stack
* **Runtime/Package Manager:** [Bun](https://bun.sh)
* **Build Tool:** [Vite](https://vitejs.dev)
* **Framework:** React (TypeScript)
* **Styling:** [Tailwind CSS v4](https://tailwindcss.com)
* **Components:** [shadcn/ui](https://ui.shadcn.com)

## Commands

| Command | Description |
| :--- | :--- |
| `bun install` | Install dependencies |
| `bun dev` | Start dev server (Port 5173) |
| `bun run build` | Compile for production (Output to `/dist`) |
| `bun lint` | Run ESLint |

## Components (shadcn/ui)

To add new components, use the shadcn CLI via bunx:

```bash
bunx --bun shadcn@latest add <component-name>
# Example:
bunx --bun shadcn@latest add card dialog dropdown-menu

```

## Proxy Setup

The `vite.config.ts` is configured to proxy `/api` requests to `http://127.0.0.1:3000`. Ensure the Rust backend is running on port 3000 during development.

