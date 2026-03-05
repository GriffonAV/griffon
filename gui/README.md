# tech

Vite (React) + Tauri + Tailwind + Shadcn

# dependencies for tauri

sudo dnf install pkgconf-pkg-config javascriptcoregtk4.1-devel

sudo dnf install @development-tools pkgconf-pkg-config webkit2gtk4.1-devel
cargo build

# Directory Structure

```
gui/
├── src-tauri/
│   ├── src/
│   └── tauri.conf.json
├── public/                # Static assets (favicons, manifest, etc.)
└── src/
    ├── assets/            # Global images, fonts, and base CSS/Sass
    ├── bindings/          # Tauri IPC logic (wrappers for 'invoke' calls)
    ├── components/        # Shared UI components
    │   ├── ui/            # Atomic components (Button, Input, Modal)
    │   └── layout/        # Page wrappers (Sidebar, Navbar, Footer)
    ├── features/          # Domain-specific logic (The "Heart" of the app)
    │   ├── dashboard/     # Example feature
    │       ├── components/
    │       ├── hooks/
    │       ├── types.ts
    │       └── index.ts   # Public API for the feature
    ├── hooks/             # Global reusable React hooks
    ├── pages/             # Route-level components (Views)
    ├── providers/         # Context Providers (Theme, Auth, Tauri settings)

    ├── store/             # State management (Zustand, Redux, etc.)
    ├── types/             # Global TypeScript interfaces/definitions

    ├── lib/               # Chadcn utils
    ├── App.tsx            # Main application entry point
    └── main.tsx           # React DOM mounting
```
