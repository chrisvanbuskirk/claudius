# Quick Start Guide

Get the Claudius frontend up and running in minutes.

## Prerequisites

- Node.js 18+ and npm
- The Claudius backend (Tauri app) should be configured

## Installation

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
npm run dev
```

3. Open your browser to `http://localhost:5173`

## First Time Setup

When you first run the app:

1. **Add Topics** - Go to Settings > Interests and add topics you want to research
2. **Configure MCP** - Go to Settings > MCP Servers and enable the servers you want to use
3. **Set Schedule** - Go to Settings > Research Settings and configure when to run research
4. **Run Research** - Click "Run Research Now" to generate your first briefings

## Project Overview

### Main Components

- **HomePage** (`/`) - View today's briefings
- **HistoryPage** (`/history`) - Search past briefings
- **SettingsPage** (`/settings`) - Configure everything

### Key Features

1. **Briefing Cards**
   - Expandable content
   - Thumbs up/down feedback
   - Source links
   - Relevance indicators (high/medium/low)

2. **Topic Management**
   - Add unlimited research topics
   - Enable/disable topics
   - Add descriptions for better results

3. **Research Control**
   - Scheduled automatic research
   - Manual trigger anytime
   - Configurable depth and sources

## Development Tips

### Hot Reload
Vite provides instant hot module replacement. Just save and see changes immediately.

### TypeScript
All types are in `src/types/index.ts`. The project uses strict TypeScript mode.

### Styling
- Uses Tailwind CSS utility classes
- Dark mode supported automatically (system preference)
- Custom components defined in `src/index.css`

### Tauri Integration
Backend calls are abstracted in `src/hooks/useTauri.ts`. Example:

```typescript
import { useBriefings } from '../hooks/useTauri';

function MyComponent() {
  const { briefings, loading, getTodaysBriefings } = useBriefings();

  useEffect(() => {
    getTodaysBriefings();
  }, []);

  return <div>{/* render briefings */}</div>;
}
```

## Building for Production

Build the optimized bundle:
```bash
npm run build
```

Output will be in the `dist/` directory.

Preview the production build:
```bash
npm run preview
```

## Troubleshooting

### Development server won't start
- Check if port 5173 is already in use
- Make sure dependencies are installed (`npm install`)

### Types errors
- Run `npm run build` to see TypeScript errors
- Check that all types match the backend models

### Tauri commands failing
- Make sure the backend is running
- Check that command names match between frontend and backend
- Look at browser console for error details

## Next Steps

1. Customize the color scheme in `tailwind.config.js`
2. Add more MCP servers in your backend config
3. Experiment with different AI models in settings
4. Set up a research schedule that works for you

## Resources

- [Vite Documentation](https://vitejs.dev)
- [React Documentation](https://react.dev)
- [Tailwind CSS](https://tailwindcss.com)
- [Tauri Documentation](https://tauri.app)
- [Lucide Icons](https://lucide.dev)
