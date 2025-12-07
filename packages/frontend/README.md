# Claudius Frontend

React + TypeScript frontend for the Claudius AI Research Assistant, built with Vite and Tauri.

## Features

- Modern React 18 with TypeScript
- Vite for fast development and building
- Tailwind CSS for styling with dark mode support
- React Router for navigation
- Tauri API integration for desktop app functionality
- Clean, card-based UI design

## Project Structure

```
src/
├── components/          # Reusable UI components
│   ├── Layout.tsx      # Main layout wrapper
│   ├── Sidebar.tsx     # Navigation sidebar
│   └── BriefingCard.tsx # Individual briefing display
├── pages/              # Route pages
│   ├── HomePage.tsx    # Today's briefings
│   ├── HistoryPage.tsx # Historical briefings with search
│   └── SettingsPage.tsx # Settings with tabs
├── hooks/              # Custom React hooks
│   └── useTauri.ts     # Tauri command hooks
├── types/              # TypeScript type definitions
│   └── index.ts        # Shared types
├── App.tsx             # Main app component with routing
├── main.tsx            # React entry point
└── index.css           # Global styles and Tailwind imports
```

## Getting Started

### Install Dependencies

```bash
npm install
```

### Development

Run the development server:

```bash
npm run dev
```

This will start Vite on `http://localhost:5173`.

### Build

Build for production:

```bash
npm run build
```

This will:
1. Run TypeScript compiler to check types
2. Build the optimized production bundle

### Preview Production Build

```bash
npm run preview
```

## Features by Page

### Home Page (`/`)
- Display today's briefings
- Refresh functionality
- Thumbs up/down feedback
- Expandable briefing content
- Source links

### History Page (`/history`)
- Search through all briefings
- Filter by topic, relevance, and date
- Same card interface as home page

### Settings Page (`/settings`)

**Interests Tab:**
- Add/remove research topics
- Enable/disable topics
- Topic descriptions

**MCP Servers Tab:**
- View available MCP servers
- Enable/disable servers
- Last used timestamps

**Research Settings Tab:**
- Configure research schedule (cron)
- Select AI model
- Set research depth
- Max sources per topic
- Enable/disable notifications
- Manual research trigger

## Tauri Integration

The app uses custom hooks to communicate with the Tauri backend:

- `useBriefings()` - Fetch and manage briefings
- `useTopics()` - Manage research topics
- `useMCPServers()` - Control MCP servers
- `useSettings()` - Configure research settings

## Styling

- **Framework:** Tailwind CSS 3.4
- **Theme:** Light/dark mode (system preference)
- **Colors:** Primary blue palette with gray neutrals
- **Components:** Custom utility classes in `index.css`

## TypeScript

Full TypeScript support with strict mode enabled. All types are defined in `src/types/index.ts` and match the backend data models.

## Icons

Using [Lucide React](https://lucide.dev) for consistent, clean iconography.

## Browser Support

Targets modern browsers (ES2020+). Built for desktop app use with Tauri.
