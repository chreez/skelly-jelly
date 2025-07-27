# Cute Figurine Module

The visual companion component for Skelly-Jelly - a melty skeleton that provides ambient support through animations and state-based expressions.

## Features

- 🎨 WebGL-powered animations with Three.js
- 🧠 State-driven behavior based on ADHD states
- 💬 Text bubble messages with personality
- 🎮 Drag-and-drop positioning
- ⚡ Performance-optimized (<2% CPU usage)
- ♿ Fully accessible with keyboard navigation

## Quick Start

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

## Architecture

### Component Structure
```
SkellyCompanion
├── AnimationCanvas (WebGL rendering)
├── TextBubble (Message display)
├── ControlPanel (User settings)
└── DragHandle (Position control)
```

### State Management
- **Zustand** for reactive state management
- **Event Bus** for module communication
- **Local Storage** for preference persistence

### Animation System
- **Three.js** for WebGL rendering
- **Custom shaders** for melting effects
- **Blend tree** for smooth transitions
- **Particle system** for celebrations

## Usage

```tsx
import { SkellyCompanion } from '@skelly-jelly/cute-figurine';

function App() {
  return (
    <SkellyCompanion
      initialState={{
        mood: 'happy',
        energy: 80
      }}
      onStateChange={(state) => {
        console.log('Companion state:', state);
      }}
    />
  );
}
```

## Development

### Storybook
```bash
npm run storybook
```

### Testing
```bash
# Unit tests
npm test

# Visual regression tests
npm run test:visual

# Test coverage
npm test -- --coverage
```

### Performance Profiling
1. Open Chrome DevTools
2. Go to Performance tab
3. Start recording
4. Interact with component
5. Stop and analyze

## API Reference

See [API Documentation](./docs/API.md) for detailed component APIs.

## Contributing

1. Follow the existing code style
2. Write tests for new features
3. Ensure performance targets are met
4. Update documentation

## License

MIT - Part of the Skelly-Jelly project