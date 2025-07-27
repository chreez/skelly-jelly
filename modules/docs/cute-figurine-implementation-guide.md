# Cute Figurine Implementation Guide

## Overview

This guide provides step-by-step instructions for implementing the Cute Figurine module, including starter code, file structure, and development workflow.

## Prerequisites

- Node.js 18+ and npm/yarn
- Rust toolchain (for Tauri)
- Basic knowledge of TypeScript, React, and WebGL
- Familiarity with the Skelly-Jelly architecture

## Project Setup

### 1. Module Structure

Create the following directory structure within the Skelly-Jelly project:

```bash
modules/cute-figurine/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── BUILD.bazel
├── README.md
├── src/
│   ├── index.ts
│   ├── components/
│   │   ├── SkellyCompanion/
│   │   │   ├── SkellyCompanion.tsx
│   │   │   ├── SkellyCompanion.test.tsx
│   │   │   ├── SkellyCompanion.stories.tsx
│   │   │   └── index.ts
│   │   ├── AnimationCanvas/
│   │   │   ├── AnimationCanvas.tsx
│   │   │   ├── useWebGLRenderer.ts
│   │   │   └── index.ts
│   │   ├── TextBubble/
│   │   │   ├── TextBubble.tsx
│   │   │   ├── TextBubble.styles.ts
│   │   │   └── index.ts
│   │   └── ControlPanel/
│   │       ├── ControlPanel.tsx
│   │       ├── ControlPanel.styles.ts
│   │       └── index.ts
│   ├── animation/
│   │   ├── AnimationEngine.ts
│   │   ├── AnimationStates.ts
│   │   ├── BlendTree.ts
│   │   ├── TransitionManager.ts
│   │   └── index.ts
│   ├── rendering/
│   │   ├── WebGLRenderer.ts
│   │   ├── ShaderManager.ts
│   │   ├── ParticleSystem.ts
│   │   ├── TextureLoader.ts
│   │   └── index.ts
│   ├── state/
│   │   ├── companionStore.ts
│   │   ├── messageQueue.ts
│   │   ├── preferenceStore.ts
│   │   └── index.ts
│   ├── services/
│   │   ├── EventBusService.ts
│   │   ├── ResourceManager.ts
│   │   ├── PerformanceMonitor.ts
│   │   └── index.ts
│   ├── assets/
│   │   ├── sprites/
│   │   ├── shaders/
│   │   ├── animations/
│   │   └── sounds/
│   ├── hooks/
│   │   ├── useAnimationFrame.ts
│   │   ├── useCompanionState.ts
│   │   ├── useDragAndDrop.ts
│   │   └── useAccessibility.ts
│   ├── types/
│   │   ├── animation.types.ts
│   │   ├── state.types.ts
│   │   ├── events.types.ts
│   │   └── index.ts
│   └── utils/
│       ├── easing.ts
│       ├── math.ts
│       ├── performance.ts
│       └── index.ts
├── tests/
│   ├── integration/
│   ├── visual/
│   └── performance/
└── public/
    └── assets/
```

### 2. Package Configuration

**package.json:**
```json
{
  "name": "@skelly-jelly/cute-figurine",
  "version": "0.1.0",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "test": "vitest",
    "test:visual": "playwright test",
    "storybook": "storybook dev -p 6006",
    "lint": "eslint src --ext .ts,.tsx",
    "type-check": "tsc --noEmit"
  },
  "dependencies": {
    "three": "^0.160.0",
    "zustand": "^4.5.0",
    "immer": "^10.0.3",
    "framer-motion": "^11.0.0",
    "@react-three/fiber": "^8.15.0",
    "@react-three/drei": "^9.92.0",
    "leva": "^0.9.35"
  },
  "devDependencies": {
    "@types/three": "^0.160.0",
    "@vitejs/plugin-react": "^4.2.0",
    "vite": "^5.0.0",
    "vitest": "^1.1.0",
    "@playwright/test": "^1.40.0",
    "@storybook/react-vite": "^7.6.0",
    "typescript": "^5.3.0"
  }
}
```

**tsconfig.json:**
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

## Core Implementation

### 1. Main Component

**src/components/SkellyCompanion/SkellyCompanion.tsx:**
```tsx
import React, { useEffect, useRef } from 'react';
import { useCompanionStore } from '@/state/companionStore';
import { AnimationCanvas } from '../AnimationCanvas';
import { TextBubble } from '../TextBubble';
import { ControlPanel } from '../ControlPanel';
import { useEventBus } from '@/hooks/useEventBus';
import { useDragAndDrop } from '@/hooks/useDragAndDrop';
import type { CompanionProps } from '@/types';

export const SkellyCompanion: React.FC<CompanionProps> = ({
  initialState,
  position: initialPosition,
  onStateChange,
  animationConfig = {
    fps: 30,
    quality: 'medium',
    enableShaders: true,
    reducedMotion: false
  }
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  
  // State management
  const {
    mood,
    energy,
    position,
    transparency,
    currentMessage,
    updatePosition,
    setAnimationConfig
  } = useCompanionStore();
  
  // Event bus integration
  useEventBus();
  
  // Drag and drop
  const { isDragging, dragHandlers } = useDragAndDrop({
    onPositionChange: updatePosition,
    containerRef
  });
  
  // Initialize
  useEffect(() => {
    if (initialState) {
      useCompanionStore.setState(initialState);
    }
    if (initialPosition) {
      updatePosition(initialPosition);
    }
    setAnimationConfig(animationConfig);
  }, []);
  
  // Notify parent of state changes
  useEffect(() => {
    onStateChange?.({ mood, energy });
  }, [mood, energy, onStateChange]);
  
  return (
    <div
      ref={containerRef}
      className="skelly-companion-container"
      style={{
        position: 'fixed',
        left: position.x,
        top: position.y,
        opacity: transparency,
        cursor: isDragging ? 'grabbing' : 'grab',
        userSelect: 'none',
        zIndex: 9999
      }}
      {...dragHandlers}
    >
      <AnimationCanvas
        width={200}
        height={200}
        config={animationConfig}
      />
      
      {currentMessage && (
        <TextBubble
          message={currentMessage}
          position="top"
        />
      )}
      
      <ControlPanel />
    </div>
  );
};
```

### 2. State Management

**src/state/companionStore.ts:**
```ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type { CompanionState, MoodState, ActivityState } from '@/types';

interface CompanionStore extends CompanionState {
  // State updates
  updateMood: (mood: MoodState) => void;
  updateEnergy: (energy: number) => void;
  updateMeltLevel: (level: number) => void;
  setActivity: (activity: ActivityState) => void;
  
  // Position management
  updatePosition: (position: Partial<Position>) => void;
  savePosition: () => void;
  
  // Animation config
  setAnimationConfig: (config: AnimationConfig) => void;
  
  // Message handling
  currentMessage: Message | null;
  queueMessage: (message: Message) => void;
  clearMessage: () => void;
  
  // Computed
  canIntervene: () => boolean;
  getCurrentAnimation: () => string;
}

export const useCompanionStore = create<CompanionStore>()(
  subscribeWithSelector(
    immer((set, get) => ({
      // Initial state
      id: 'skelly-main',
      name: 'Skelly',
      mood: 'happy' as MoodState,
      energy: 80,
      happiness: 75,
      focus: 50,
      meltLevel: 0,
      glowIntensity: 0.5,
      activity: 'idle' as ActivityState,
      position: { 
        x: window.innerWidth - 200, 
        y: window.innerHeight - 200 
      },
      transparency: 1,
      animationConfig: {
        fps: 30,
        quality: 'medium',
        enableShaders: true,
        reducedMotion: false
      },
      currentMessage: null,
      interventionCooldown: 0,
      
      // Actions
      updateMood: (mood) => set((state) => {
        state.mood = mood;
        
        // Mood-specific side effects
        switch (mood) {
          case 'melting':
            state.meltLevel = Math.min(state.meltLevel + 20, 100);
            state.glowIntensity = 0.2;
            break;
          case 'excited':
            state.glowIntensity = 0.8;
            break;
          case 'focused':
            state.glowIntensity = 0.3;
            break;
        }
      }),
      
      updateEnergy: (energy) => set((state) => {
        state.energy = Math.max(0, Math.min(100, energy));
        
        // Auto-adjust mood based on energy
        if (energy < 20 && state.mood !== 'tired') {
          state.mood = 'tired';
          state.meltLevel = Math.min(state.meltLevel + 10, 80);
        } else if (energy > 80 && state.mood === 'tired') {
          state.mood = 'happy';
          state.meltLevel = Math.max(state.meltLevel - 20, 0);
        }
      }),
      
      updateMeltLevel: (level) => set((state) => {
        state.meltLevel = Math.max(0, Math.min(100, level));
        if (level > 70 && state.mood !== 'melting') {
          state.mood = 'melting';
        }
      }),
      
      setActivity: (activity) => set((state) => {
        state.activity = activity;
      }),
      
      updatePosition: (position) => set((state) => {
        state.position = { ...state.position, ...position };
      }),
      
      savePosition: () => {
        const { position } = get();
        localStorage.setItem('skelly-position', JSON.stringify(position));
      },
      
      setAnimationConfig: (config) => set((state) => {
        state.animationConfig = { ...state.animationConfig, ...config };
      }),
      
      queueMessage: (message) => set((state) => {
        state.currentMessage = message;
        
        // Auto-clear message after duration
        setTimeout(() => {
          get().clearMessage();
        }, message.duration || 5000);
      }),
      
      clearMessage: () => set((state) => {
        state.currentMessage = null;
      }),
      
      canIntervene: () => {
        const { activity, interventionCooldown } = get();
        return interventionCooldown === 0 && activity !== 'resting';
      },
      
      getCurrentAnimation: () => {
        const { mood, activity, meltLevel } = get();
        
        // Determine animation based on state
        if (meltLevel > 70) return 'melting_heavy';
        if (activity === 'working' && mood === 'focused') return 'focused_breathing';
        if (mood === 'excited') return 'happy_bounce';
        if (mood === 'tired') return 'tired_sway';
        
        return 'idle_breathing';
      }
    }))
  )
);
```

### 3. Animation Engine

**src/animation/AnimationEngine.ts:**
```ts
import * as THREE from 'three';
import { AnimationState, Keyframe, TransitionConfig } from '@/types';
import { TransitionManager } from './TransitionManager';
import { BlendTree } from './BlendTree';
import { easings } from '@/utils/easing';

export class AnimationEngine {
  private clock: THREE.Clock;
  private mixer: THREE.AnimationMixer;
  private animations: Map<string, THREE.AnimationClip>;
  private currentAction: THREE.AnimationAction | null = null;
  private blendTree: BlendTree;
  private transitionManager: TransitionManager;
  
  constructor(private scene: THREE.Scene, private skeleton: THREE.Skeleton) {
    this.clock = new THREE.Clock();
    this.mixer = new THREE.AnimationMixer(skeleton.bones[0]);
    this.animations = new Map();
    this.blendTree = new BlendTree();
    this.transitionManager = new TransitionManager(this.mixer);
    
    this.loadAnimations();
  }
  
  private loadAnimations() {
    // Define animation clips
    const animations = [
      this.createIdleAnimation(),
      this.createHappyBounceAnimation(),
      this.createMeltingAnimation(),
      this.createFocusedAnimation()
    ];
    
    animations.forEach(clip => {
      this.animations.set(clip.name, clip);
    });
  }
  
  private createIdleAnimation(): THREE.AnimationClip {
    const times = [0, 1.5, 3];
    const values = [
      0, 1, 0,    // Start position
      0, 1.05, 0, // Mid position (slight rise)
      0, 1, 0     // End position
    ];
    
    const positionTrack = new THREE.VectorKeyframeTrack(
      '.position',
      times,
      values,
      THREE.InterpolateSmooth
    );
    
    return new THREE.AnimationClip('idle_breathing', 3, [positionTrack]);
  }
  
  private createHappyBounceAnimation(): THREE.AnimationClip {
    const times = [0, 0.3, 0.5, 0.7, 1];
    const values = [
      0, 0, 0,     // Start
      0, 0.3, 0,   // Jump up
      0, 0, 0,     // Land
      0, 0.1, 0,   // Small bounce
      0, 0, 0      // Settle
    ];
    
    const positionTrack = new THREE.VectorKeyframeTrack(
      '.position',
      times,
      values,
      THREE.InterpolateSmooth
    );
    
    // Add squash and stretch
    const scaleValues = [
      1, 1, 1,      // Normal
      0.9, 1.2, 0.9, // Stretch
      1.1, 0.9, 1.1, // Squash
      1, 1, 1,       // Normal
      1, 1, 1        // Normal
    ];
    
    const scaleTrack = new THREE.VectorKeyframeTrack(
      '.scale',
      times,
      scaleValues,
      THREE.InterpolateSmooth
    );
    
    return new THREE.AnimationClip('happy_bounce', 1, [positionTrack, scaleTrack]);
  }
  
  private createMeltingAnimation(): THREE.AnimationClip {
    // This would be handled by shaders, but we can add bone deformation
    const times = [0, 2];
    const scaleValues = [
      1, 1, 1,      // Normal
      1.2, 0.7, 1.2 // Melted (wider and shorter)
    ];
    
    const scaleTrack = new THREE.VectorKeyframeTrack(
      '.scale',
      times,
      scaleValues,
      THREE.InterpolateSmooth
    );
    
    return new THREE.AnimationClip('melting_effect', 2, [scaleTrack]);
  }
  
  private createFocusedAnimation(): THREE.AnimationClip {
    // Subtle breathing with glow
    const times = [0, 2, 4];
    const values = [
      1, 1, 1,
      1.02, 1.02, 1.02,
      1, 1, 1
    ];
    
    const scaleTrack = new THREE.VectorKeyframeTrack(
      '.scale',
      times,
      values,
      THREE.InterpolateSmooth
    );
    
    return new THREE.AnimationClip('focused_breathing', 4, [scaleTrack]);
  }
  
  play(animationName: string, options?: {
    loop?: boolean;
    duration?: number;
    transition?: boolean;
  }) {
    const clip = this.animations.get(animationName);
    if (!clip) {
      console.warn(`Animation ${animationName} not found`);
      return;
    }
    
    const action = this.mixer.clipAction(clip);
    
    // Configure action
    if (options?.loop !== false) {
      action.setLoop(THREE.LoopRepeat, Infinity);
    } else {
      action.setLoop(THREE.LoopOnce, 1);
      action.clampWhenFinished = true;
    }
    
    if (options?.duration) {
      action.setDuration(options.duration / 1000);
    }
    
    // Handle transition
    if (this.currentAction && options?.transition !== false) {
      this.transitionManager.crossfade(this.currentAction, action, 0.5);
    } else {
      action.play();
    }
    
    this.currentAction = action;
  }
  
  update(deltaTime: number) {
    this.mixer.update(deltaTime / 1000);
    this.transitionManager.update();
  }
  
  stop() {
    if (this.currentAction) {
      this.currentAction.stop();
      this.currentAction = null;
    }
  }
  
  setTimeScale(scale: number) {
    this.mixer.timeScale = scale;
  }
}
```

### 4. WebGL Renderer

**src/rendering/WebGLRenderer.ts:**
```ts
import * as THREE from 'three';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass';
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass';
import { ShaderManager } from './ShaderManager';
import { ParticleSystem } from './ParticleSystem';

export class WebGLRenderer {
  private renderer: THREE.WebGLRenderer;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private composer: EffectComposer;
  private shaderManager: ShaderManager;
  private particleSystem: ParticleSystem;
  
  constructor(canvas: HTMLCanvasElement, width: number, height: number) {
    // Initialize renderer
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      alpha: true,
      antialias: true,
      powerPreference: 'low-power'
    });
    
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    
    // Setup scene
    this.scene = new THREE.Scene();
    
    // Setup camera
    this.camera = new THREE.PerspectiveCamera(
      45,
      width / height,
      0.1,
      100
    );
    this.camera.position.z = 5;
    
    // Setup post-processing
    this.composer = new EffectComposer(this.renderer);
    const renderPass = new RenderPass(this.scene, this.camera);
    this.composer.addPass(renderPass);
    
    // Add glow effect
    const bloomPass = new UnrealBloomPass(
      new THREE.Vector2(width, height),
      0.5, // strength
      0.4, // radius
      0.85 // threshold
    );
    this.composer.addPass(bloomPass);
    
    // Initialize managers
    this.shaderManager = new ShaderManager();
    this.particleSystem = new ParticleSystem(this.scene);
    
    // Setup lighting
    this.setupLighting();
  }
  
  private setupLighting() {
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    this.scene.add(ambientLight);
    
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.4);
    directionalLight.position.set(0, 1, 1);
    this.scene.add(directionalLight);
  }
  
  createSkeletonMesh(): THREE.SkinnedMesh {
    // Create skeleton geometry
    const geometry = new THREE.BoxGeometry(1, 2, 0.5);
    
    // Create bones
    const bones: THREE.Bone[] = [];
    const rootBone = new THREE.Bone();
    bones.push(rootBone);
    
    // Create skeleton
    const skeleton = new THREE.Skeleton(bones);
    
    // Create material with custom shader
    const material = this.shaderManager.createSkeletonMaterial();
    
    // Create mesh
    const mesh = new THREE.SkinnedMesh(geometry, material);
    mesh.bind(skeleton);
    
    this.scene.add(mesh);
    this.scene.add(rootBone);
    
    return mesh;
  }
  
  render() {
    this.composer.render();
  }
  
  updateShaderUniforms(uniforms: Record<string, any>) {
    this.shaderManager.updateUniforms(uniforms);
  }
  
  emitParticles(type: string, options: any) {
    this.particleSystem.emit(type, options);
  }
  
  updateParticles(deltaTime: number) {
    this.particleSystem.update(deltaTime);
  }
  
  resize(width: number, height: number) {
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    
    this.renderer.setSize(width, height);
    this.composer.setSize(width, height);
  }
  
  dispose() {
    this.renderer.dispose();
    this.particleSystem.dispose();
    this.shaderManager.dispose();
  }
}
```

### 5. Event Bus Integration

**src/services/EventBusService.ts:**
```ts
import { useCompanionStore } from '@/state/companionStore';
import type { 
  AnimationCommandEvent, 
  StateClassificationEvent,
  InterventionRequestEvent,
  RewardEarnedEvent 
} from '@/types/events.types';

export class EventBusService {
  private eventHandlers: Map<string, Function[]> = new Map();
  private messageQueue = new MessageQueue();
  
  constructor() {
    this.setupEventHandlers();
  }
  
  private setupEventHandlers() {
    // Animation commands
    this.subscribe('AnimationCommand', (event: AnimationCommandEvent) => {
      this.handleAnimationCommand(event);
    });
    
    // State changes
    this.subscribe('StateClassification', (event: StateClassificationEvent) => {
      this.handleStateChange(event);
    });
    
    // Interventions
    this.subscribe('InterventionRequest', (event: InterventionRequestEvent) => {
      this.handleIntervention(event);
    });
    
    // Rewards
    this.subscribe('RewardEarned', (event: RewardEarnedEvent) => {
      this.handleReward(event);
    });
  }
  
  private handleAnimationCommand(event: AnimationCommandEvent) {
    const { command, animation, parameters, message } = event.payload;
    const store = useCompanionStore.getState();
    
    switch (command) {
      case 'play':
        if (animation) {
          // Animation will be handled by the animation engine
          window.dispatchEvent(new CustomEvent('playAnimation', { 
            detail: { animation, parameters } 
          }));
        }
        break;
        
      case 'setMood':
        if (parameters?.mood) {
          store.updateMood(parameters.mood);
        }
        break;
    }
    
    if (message) {
      store.queueMessage({
        text: message.text,
        duration: 5000,
        style: message.personality
      });
    }
  }
  
  private handleStateChange(event: StateClassificationEvent) {
    const { state, confidence } = event.payload;
    const store = useCompanionStore.getState();
    
    // Only update if confidence is high enough
    if (confidence < 0.7) return;
    
    switch (state) {
      case 'focused':
        store.updateMood('focused');
        store.setActivity('working');
        break;
        
      case 'distracted':
        store.updateMood('thinking');
        store.updateEnergy(store.energy - 5);
        break;
        
      case 'tired':
        store.updateMood('tired');
        store.updateMeltLevel(store.meltLevel + 10);
        break;
    }
  }
  
  private handleIntervention(event: InterventionRequestEvent) {
    const { interventionType, message, priority } = event.payload;
    const store = useCompanionStore.getState();
    
    if (!store.canIntervene()) return;
    
    // Queue message with appropriate priority
    this.messageQueue.add({
      text: message,
      duration: priority === 'high' ? 8000 : 5000,
      style: this.getInterventionStyle(interventionType)
    }, this.getPriorityValue(priority));
    
    // Play intervention animation
    const animationMap = {
      'break_reminder': 'gentle_wave',
      'focus_help': 'thinking_pose',
      'celebration': 'happy_dance',
      'encouragement': 'thumbs_up'
    };
    
    window.dispatchEvent(new CustomEvent('playAnimation', {
      detail: { animation: animationMap[interventionType] }
    }));
  }
  
  private handleReward(event: RewardEarnedEvent) {
    const { celebrationLevel, message, value } = event.payload;
    const store = useCompanionStore.getState();
    
    // Update happiness
    store.updateHappiness(store.happiness + value);
    
    // Show celebration
    store.updateMood('celebrating');
    
    // Emit particles
    window.dispatchEvent(new CustomEvent('emitParticles', {
      detail: {
        type: 'confetti',
        count: celebrationLevel === 'large' ? 100 : 50,
        duration: 3000
      }
    }));
    
    // Show message
    store.queueMessage({
      text: message,
      duration: 6000,
      style: 'celebration'
    });
  }
  
  subscribe(eventType: string, handler: Function) {
    if (!this.eventHandlers.has(eventType)) {
      this.eventHandlers.set(eventType, []);
    }
    this.eventHandlers.get(eventType)!.push(handler);
  }
  
  emit(eventType: string, data: any) {
    const handlers = this.eventHandlers.get(eventType) || [];
    handlers.forEach(handler => handler(data));
  }
}
```

## Development Workflow

### 1. Initial Setup

```bash
# Navigate to the module directory
cd modules/cute-figurine

# Install dependencies
npm install

# Run development server
npm run dev
```

### 2. Storybook Development

Create stories for isolated component development:

**src/components/SkellyCompanion/SkellyCompanion.stories.tsx:**
```tsx
import type { Meta, StoryObj } from '@storybook/react';
import { SkellyCompanion } from './SkellyCompanion';

const meta: Meta<typeof SkellyCompanion> = {
  title: 'Components/SkellyCompanion',
  component: SkellyCompanion,
  parameters: {
    layout: 'fullscreen',
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    initialState: {
      mood: 'happy',
      energy: 80,
    },
  },
};

export const Tired: Story = {
  args: {
    initialState: {
      mood: 'tired',
      energy: 20,
      meltLevel: 50,
    },
  },
};

export const Celebrating: Story = {
  args: {
    initialState: {
      mood: 'celebrating',
      energy: 90,
      happiness: 100,
    },
  },
};
```

### 3. Testing

**Unit tests:**
```bash
npm run test
```

**Visual regression tests:**
```bash
npm run test:visual
```

**Performance tests:**
```ts
// tests/performance/animation.perf.test.ts
import { test, expect } from '@playwright/test';

test('animation performance', async ({ page }) => {
  await page.goto('/test/animation-performance');
  
  // Start performance measurement
  const metrics = await page.evaluate(() => {
    return new Promise((resolve) => {
      const start = performance.now();
      let frames = 0;
      
      const measure = () => {
        frames++;
        if (performance.now() - start < 1000) {
          requestAnimationFrame(measure);
        } else {
          resolve({
            fps: frames,
            memory: performance.memory?.usedJSHeapSize || 0
          });
        }
      };
      
      requestAnimationFrame(measure);
    });
  });
  
  expect(metrics.fps).toBeGreaterThan(25);
  expect(metrics.memory).toBeLessThan(50 * 1024 * 1024); // 50MB
});
```

### 4. Integration

To integrate with the main Skelly-Jelly application:

```ts
// In the main app
import { SkellyCompanion } from '@skelly-jelly/cute-figurine';

function App() {
  return (
    <div>
      {/* Other components */}
      <SkellyCompanion
        onStateChange={(state) => {
          console.log('Companion state:', state);
        }}
      />
    </div>
  );
}
```

## Performance Optimization Checklist

- [ ] Profile initial render performance
- [ ] Implement texture atlasing for sprites
- [ ] Add LOD (Level of Detail) system
- [ ] Optimize shader complexity based on device
- [ ] Implement frame skipping for low-end devices
- [ ] Add performance monitoring dashboard
- [ ] Test on various devices and browsers
- [ ] Implement progressive enhancement
- [ ] Add memory leak detection
- [ ] Optimize bundle size with tree shaking

## Next Steps

1. **Phase 1 (MVP)**:
   - Basic skeleton with 5 animation states
   - Simple drag positioning
   - Text bubble messages
   - WebGL 2D rendering

2. **Phase 2**:
   - Advanced shader effects
   - Particle systems
   - More animation states
   - Settings panel

3. **Phase 3**:
   - 3D skeleton option
   - Custom skins/themes
   - Animation editor
   - Community animations

## Resources

- [Three.js Documentation](https://threejs.org/docs/)
- [React Three Fiber](https://docs.pmnd.rs/react-three-fiber)
- [Zustand Documentation](https://github.com/pmndrs/zustand)
- [WebGL Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)
- [Animation Principles](https://www.animationmentor.com/blog/12-principles-of-animation/)

## Troubleshooting

**Common Issues:**

1. **High CPU Usage**: 
   - Check animation loop efficiency
   - Reduce particle count
   - Disable shaders on low-end devices

2. **Memory Leaks**:
   - Ensure proper cleanup in useEffect
   - Dispose Three.js objects
   - Clear event listeners

3. **Animation Jank**:
   - Use requestAnimationFrame
   - Batch DOM updates
   - Optimize render calls

4. **Cross-browser Issues**:
   - Test WebGL support
   - Provide Canvas 2D fallback
   - Check for vendor prefixes