# Cute Figurine State Management & Animation System

## Overview

This document details the state management architecture and animation system for the Skelly-Jelly cute figurine component. The system is designed to be reactive, performant, and maintainable while providing smooth, delightful animations that respond to user ADHD states.

## State Management Architecture

### Core State Model

```typescript
// Primary companion state
interface CompanionState {
  // Identity
  id: string;
  name: string;
  
  // Emotional state
  mood: MoodState;
  energy: number; // 0-100
  happiness: number; // 0-100
  focus: number; // 0-100
  
  // Physical state
  meltLevel: number; // 0-100
  glowIntensity: number; // 0-1
  particleCount: number; // 0-maxParticles
  
  // Activity state
  activity: ActivityState;
  lastInteraction: Date;
  interventionCooldown: number; // seconds
  
  // Position and appearance
  position: Position;
  scale: number;
  rotation: number;
  transparency: number; // 0-1
}

// Mood states with transitions
enum MoodState {
  HAPPY = 'happy',
  FOCUSED = 'focused',
  TIRED = 'tired',
  EXCITED = 'excited',
  MELTING = 'melting',
  THINKING = 'thinking',
  CELEBRATING = 'celebrating'
}

// Activity states
enum ActivityState {
  IDLE = 'idle',
  WORKING = 'working',
  INTERACTING = 'interacting',
  RESTING = 'resting',
  INTERVENING = 'intervening'
}
```

### State Store Implementation

Using Zustand for lightweight, performant state management:

```typescript
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';

interface CompanionStore extends CompanionState {
  // Actions
  updateMood: (mood: MoodState, transition?: boolean) => void;
  updateEnergy: (energy: number) => void;
  updateMeltLevel: (level: number) => void;
  setActivity: (activity: ActivityState) => void;
  recordInteraction: () => void;
  
  // Position actions
  setPosition: (position: Partial<Position>) => void;
  savePosition: () => void;
  
  // Animation actions
  triggerAnimation: (animation: string) => void;
  queueMessage: (message: Message) => void;
  
  // Computed values
  isInCooldown: () => boolean;
  canIntervene: () => boolean;
  getCurrentAnimation: () => AnimationState;
}

const useCompanionStore = create<CompanionStore>()(
  subscribeWithSelector(
    immer((set, get) => ({
      // Initial state
      id: 'skelly-main',
      name: 'Skelly',
      mood: MoodState.HAPPY,
      energy: 80,
      happiness: 75,
      focus: 50,
      meltLevel: 0,
      glowIntensity: 0.5,
      particleCount: 0,
      activity: ActivityState.IDLE,
      lastInteraction: new Date(),
      interventionCooldown: 0,
      position: { x: window.innerWidth - 150, y: window.innerHeight - 150 },
      scale: 1,
      rotation: 0,
      transparency: 1,
      
      // Actions
      updateMood: (mood, transition = true) => set((state) => {
        state.mood = mood;
        // Trigger mood-specific side effects
        switch (mood) {
          case MoodState.MELTING:
            state.meltLevel = Math.min(state.meltLevel + 20, 100);
            break;
          case MoodState.EXCITED:
            state.particleCount = 50;
            state.glowIntensity = 0.8;
            break;
          case MoodState.FOCUSED:
            state.glowIntensity = 0.3;
            state.particleCount = 0;
            break;
        }
      }),
      
      updateEnergy: (energy) => set((state) => {
        state.energy = Math.max(0, Math.min(100, energy));
        // Auto-adjust mood based on energy
        if (energy < 20) {
          state.mood = MoodState.TIRED;
          state.meltLevel = Math.min(state.meltLevel + 10, 80);
        } else if (energy > 80 && state.mood === MoodState.TIRED) {
          state.mood = MoodState.HAPPY;
          state.meltLevel = Math.max(state.meltLevel - 20, 0);
        }
      }),
      
      updateMeltLevel: (level) => set((state) => {
        state.meltLevel = Math.max(0, Math.min(100, level));
        if (level > 70) {
          state.mood = MoodState.MELTING;
        }
      }),
      
      setActivity: (activity) => set((state) => {
        state.activity = activity;
        // Activity-specific adjustments
        if (activity === ActivityState.WORKING) {
          state.focus = Math.min(state.focus + 10, 100);
        }
      }),
      
      recordInteraction: () => set((state) => {
        state.lastInteraction = new Date();
        state.happiness = Math.min(state.happiness + 5, 100);
        state.interventionCooldown = 900; // 15 minutes
      }),
      
      setPosition: (position) => set((state) => {
        state.position = { ...state.position, ...position };
      }),
      
      savePosition: () => {
        const { position } = get();
        localStorage.setItem('skelly-position', JSON.stringify(position));
      },
      
      // Computed
      isInCooldown: () => {
        const { interventionCooldown } = get();
        return interventionCooldown > 0;
      },
      
      canIntervene: () => {
        const { activity, interventionCooldown, mood } = get();
        return (
          interventionCooldown === 0 &&
          activity !== ActivityState.RESTING &&
          mood !== MoodState.FOCUSED
        );
      },
      
      getCurrentAnimation: () => {
        const { mood, activity, meltLevel } = get();
        return animationResolver.resolve(mood, activity, meltLevel);
      }
    }))
  )
);
```

### State Synchronization

```typescript
// Sync with event bus
class StateEventSync {
  private unsubscribers: (() => void)[] = [];
  
  initialize() {
    // Listen for external state changes
    eventBus.subscribe('StateChange', (event: StateChangeEvent) => {
      const store = useCompanionStore.getState();
      
      switch (event.type) {
        case 'ADHD_STATE_DETECTED':
          this.handleADHDState(event.payload);
          break;
        case 'INTERVENTION_REQUEST':
          this.handleIntervention(event.payload);
          break;
        case 'REWARD_EARNED':
          this.handleReward(event.payload);
          break;
      }
    });
    
    // Subscribe to store changes
    const unsubscribe = useCompanionStore.subscribe(
      (state) => state.mood,
      (mood) => {
        eventBus.emit('CompanionMoodChange', { mood });
      }
    );
    
    this.unsubscribers.push(unsubscribe);
  }
  
  private handleADHDState(state: ADHDState) {
    const store = useCompanionStore.getState();
    
    if (state.type === 'HYPERFOCUS') {
      store.updateMood(MoodState.FOCUSED);
      store.updateEnergy(90);
    } else if (state.type === 'DISTRACTED') {
      store.updateMood(MoodState.THINKING);
      store.updateEnergy(store.energy - 5);
    }
  }
  
  private handleIntervention(request: InterventionRequest) {
    const store = useCompanionStore.getState();
    
    if (store.canIntervene()) {
      store.setActivity(ActivityState.INTERVENING);
      store.queueMessage({
        text: request.message,
        duration: 5000,
        type: 'intervention'
      });
    }
  }
  
  private handleReward(reward: Reward) {
    const store = useCompanionStore.getState();
    store.updateMood(MoodState.CELEBRATING);
    store.updateHappiness(store.happiness + reward.value);
  }
  
  destroy() {
    this.unsubscribers.forEach(unsub => unsub());
  }
}
```

## Animation System

### Animation Architecture

```typescript
// Core animation types
interface AnimationState {
  id: string;
  name: string;
  type: AnimationType;
  duration: number;
  loop: boolean;
  priority: number;
  
  // Animation data
  keyframes: Keyframe[];
  transitions: StateTransition[];
  blendWeight: number;
  
  // Performance settings
  fps: number;
  quality: AnimationQuality;
  
  // Shader parameters
  shaderUniforms?: ShaderUniforms;
}

enum AnimationType {
  SKELETAL = 'skeletal',
  MORPH = 'morph',
  SHADER = 'shader',
  PARTICLE = 'particle',
  COMPOSITE = 'composite'
}

interface Keyframe {
  time: number; // 0-1 normalized
  value: any; // Type depends on animation type
  easing: EasingFunction;
  interpolation: 'linear' | 'cubic' | 'step';
}

interface StateTransition {
  from: string;
  to: string;
  duration: number;
  condition?: () => boolean;
  blendCurve: BlendCurve;
}
```

### Animation Engine

```typescript
class AnimationEngine {
  private states: Map<string, AnimationState> = new Map();
  private activeAnimations: AnimationInstance[] = [];
  private blendTree: BlendTree;
  private clock: Clock;
  private performanceMonitor: PerformanceMonitor;
  
  constructor(private renderer: WebGLRenderer) {
    this.clock = new Clock();
    this.blendTree = new BlendTree();
    this.performanceMonitor = new PerformanceMonitor();
    
    this.loadAnimationStates();
    this.setupBlendTree();
  }
  
  // Main update loop
  update(deltaTime: number) {
    // Update all active animations
    for (const animation of this.activeAnimations) {
      this.updateAnimation(animation, deltaTime);
    }
    
    // Blend animations
    const blendedPose = this.blendTree.evaluate(this.activeAnimations);
    
    // Apply to skeleton
    this.applyPose(blendedPose);
    
    // Update particles and shaders
    this.updateEffects(deltaTime);
    
    // Monitor performance
    this.performanceMonitor.update();
    
    // Adaptive quality
    if (this.performanceMonitor.shouldReduceQuality()) {
      this.reduceAnimationQuality();
    }
  }
  
  // Play animation with transition
  async play(stateName: string, transition = true): Promise<void> {
    const state = this.states.get(stateName);
    if (!state) throw new Error(`Animation state ${stateName} not found`);
    
    if (transition && this.activeAnimations.length > 0) {
      await this.transitionTo(state);
    } else {
      this.startAnimation(state);
    }
  }
  
  // Smooth transition between states
  private async transitionTo(targetState: AnimationState): Promise<void> {
    const currentAnimation = this.activeAnimations[0];
    const transition = this.findTransition(currentAnimation.state, targetState);
    
    if (!transition) {
      // Default transition
      await this.crossfade(currentAnimation, targetState, 500);
    } else {
      // Custom transition
      await this.executeTransition(transition, currentAnimation, targetState);
    }
  }
  
  // Crossfade between animations
  private async crossfade(
    from: AnimationInstance,
    to: AnimationState,
    duration: number
  ): Promise<void> {
    const toInstance = this.createInstance(to);
    this.activeAnimations.push(toInstance);
    
    const startTime = performance.now();
    
    return new Promise((resolve) => {
      const fade = () => {
        const elapsed = performance.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        from.blendWeight = 1 - progress;
        toInstance.blendWeight = progress;
        
        if (progress >= 1) {
          this.removeAnimation(from);
          resolve();
        } else {
          requestAnimationFrame(fade);
        }
      };
      
      requestAnimationFrame(fade);
    });
  }
}
```

### Animation State Definitions

```typescript
// Animation state registry
const animationStates: AnimationStateDefinition[] = [
  // Idle animations
  {
    id: 'idle_breathing',
    type: AnimationType.SKELETAL,
    duration: 3000,
    loop: true,
    keyframes: [
      { time: 0, value: { scale: 1.0 }, easing: 'easeInOutSine' },
      { time: 0.5, value: { scale: 1.05 }, easing: 'easeInOutSine' },
      { time: 1, value: { scale: 1.0 }, easing: 'easeInOutSine' }
    ]
  },
  
  // Mood animations
  {
    id: 'happy_bounce',
    type: AnimationType.COMPOSITE,
    duration: 1000,
    loop: false,
    keyframes: [
      { time: 0, value: { y: 0, squash: 1 }, easing: 'easeOutQuad' },
      { time: 0.3, value: { y: -20, squash: 0.9 }, easing: 'easeInQuad' },
      { time: 0.5, value: { y: 0, squash: 1.1 }, easing: 'easeOutBounce' },
      { time: 1, value: { y: 0, squash: 1 }, easing: 'easeOutQuad' }
    ]
  },
  
  // Melting animation
  {
    id: 'melting_effect',
    type: AnimationType.SHADER,
    duration: 2000,
    loop: false,
    shaderUniforms: {
      meltAmount: { value: 0, target: 1 },
      viscosity: { value: 0.8 },
      gravity: { value: 9.8 }
    }
  },
  
  // Particle effects
  {
    id: 'celebration_particles',
    type: AnimationType.PARTICLE,
    duration: 3000,
    loop: false,
    particleConfig: {
      count: 100,
      velocity: { min: 50, max: 200 },
      lifetime: { min: 1000, max: 2000 },
      colors: ['#FFD700', '#FF69B4', '#00CED1', '#32CD32'],
      gravity: -100,
      spread: 45
    }
  }
];
```

### Blend Tree System

```typescript
class BlendTree {
  private root: BlendNode;
  
  constructor() {
    this.root = this.buildDefaultTree();
  }
  
  private buildDefaultTree(): BlendNode {
    // Create blend tree structure
    return new BlendNode({
      type: 'blend2d',
      parameters: ['energy', 'happiness'],
      children: [
        // Low energy, low happiness
        new AnimationNode('tired_slump'),
        // Low energy, high happiness
        new AnimationNode('relaxed_smile'),
        // High energy, low happiness
        new AnimationNode('anxious_fidget'),
        // High energy, high happiness
        new AnimationNode('excited_bounce')
      ],
      blendSpace: [
        { x: 0, y: 0, weight: [1, 0, 0, 0] },
        { x: 0, y: 1, weight: [0, 1, 0, 0] },
        { x: 1, y: 0, weight: [0, 0, 1, 0] },
        { x: 1, y: 1, weight: [0, 0, 0, 1] }
      ]
    });
  }
  
  evaluate(activeAnimations: AnimationInstance[]): Pose {
    const companionState = useCompanionStore.getState();
    
    // Set blend parameters
    const params = {
      energy: companionState.energy / 100,
      happiness: companionState.happiness / 100,
      meltLevel: companionState.meltLevel / 100,
      focus: companionState.focus / 100
    };
    
    // Evaluate tree
    return this.root.evaluate(params, activeAnimations);
  }
}
```

### Shader Effects System

```typescript
// Melt shader
const meltShader = {
  uniforms: {
    time: { value: 0 },
    meltAmount: { value: 0 },
    meltSpeed: { value: 1 },
    noiseTexture: { value: null }
  },
  
  vertexShader: `
    uniform float time;
    uniform float meltAmount;
    uniform sampler2D noiseTexture;
    
    varying vec2 vUv;
    varying float vMelt;
    
    void main() {
      vUv = uv;
      vec3 pos = position;
      
      // Sample noise for organic melting
      float noise = texture2D(noiseTexture, uv * 2.0 + time * 0.1).r;
      
      // Apply melting deformation
      float melt = meltAmount * (1.0 - uv.y) * (0.5 + noise * 0.5);
      pos.y -= melt * 10.0;
      pos.x += sin(uv.y * 6.28 + time) * melt * 2.0;
      
      vMelt = melt;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
    }
  `,
  
  fragmentShader: `
    uniform float meltAmount;
    varying vec2 vUv;
    varying float vMelt;
    
    void main() {
      vec4 color = texture2D(map, vUv);
      
      // Add glossy effect to melted parts
      float gloss = smoothstep(0.0, 0.5, vMelt);
      color.rgb += gloss * 0.2;
      
      // Transparency at edges
      color.a *= 1.0 - smoothstep(0.7, 1.0, vMelt);
      
      gl_FragColor = color;
    }
  `
};

// Glow shader
const glowShader = {
  uniforms: {
    glowIntensity: { value: 0.5 },
    glowColor: { value: new THREE.Color(0x00ff88) },
    time: { value: 0 }
  },
  
  fragmentShader: `
    uniform float glowIntensity;
    uniform vec3 glowColor;
    uniform float time;
    
    void main() {
      vec4 color = texture2D(map, vUv);
      
      // Pulsing glow
      float pulse = sin(time * 2.0) * 0.5 + 0.5;
      float glow = glowIntensity * (0.7 + pulse * 0.3);
      
      // Apply glow
      color.rgb = mix(color.rgb, glowColor, glow * 0.5);
      color.rgb += glowColor * glow * 0.3;
      
      gl_FragColor = color;
    }
  `
};
```

### Performance Optimization

```typescript
class PerformanceMonitor {
  private samples: PerformanceSample[] = [];
  private targetFPS = 30;
  private maxCPU = 2; // percentage
  
  update() {
    const sample = {
      timestamp: performance.now(),
      fps: this.calculateFPS(),
      cpu: this.getCPUUsage(),
      memory: performance.memory?.usedJSHeapSize || 0
    };
    
    this.samples.push(sample);
    if (this.samples.length > 60) {
      this.samples.shift();
    }
  }
  
  shouldReduceQuality(): boolean {
    const recentSamples = this.samples.slice(-10);
    const avgFPS = average(recentSamples.map(s => s.fps));
    const avgCPU = average(recentSamples.map(s => s.cpu));
    
    return avgFPS < this.targetFPS * 0.8 || avgCPU > this.maxCPU;
  }
  
  getQualityLevel(): AnimationQuality {
    const avgFPS = this.getAverageFPS();
    
    if (avgFPS > this.targetFPS * 0.9) return AnimationQuality.HIGH;
    if (avgFPS > this.targetFPS * 0.7) return AnimationQuality.MEDIUM;
    return AnimationQuality.LOW;
  }
}

// Adaptive quality settings
interface QualitySettings {
  [AnimationQuality.HIGH]: {
    particleCount: 100,
    shaderComplexity: 'complex',
    skeletonBones: 30,
    textureResolution: 1024
  },
  [AnimationQuality.MEDIUM]: {
    particleCount: 50,
    shaderComplexity: 'simple',
    skeletonBones: 20,
    textureResolution: 512
  },
  [AnimationQuality.LOW]: {
    particleCount: 10,
    shaderComplexity: 'none',
    skeletonBones: 10,
    textureResolution: 256
  }
}
```

## State-Animation Integration

### Animation Resolver

```typescript
class AnimationResolver {
  private stateAnimationMap = new Map<string, string[]>();
  
  constructor() {
    this.initializeStateMap();
  }
  
  private initializeStateMap() {
    // Map states to animations
    this.stateAnimationMap.set('happy-idle', [
      'idle_breathing',
      'idle_sway',
      'occasional_blink'
    ]);
    
    this.stateAnimationMap.set('focused-working', [
      'subtle_breathing',
      'minimal_movement',
      'steady_glow'
    ]);
    
    this.stateAnimationMap.set('tired-melting', [
      'slow_breathing',
      'melting_effect',
      'droopy_eyes'
    ]);
    
    this.stateAnimationMap.set('excited-celebrating', [
      'happy_bounce',
      'celebration_particles',
      'rainbow_glow'
    ]);
  }
  
  resolve(mood: MoodState, activity: ActivityState, meltLevel: number): string[] {
    const stateKey = `${mood}-${activity}`;
    let animations = this.stateAnimationMap.get(stateKey) || ['idle_breathing'];
    
    // Add melt effect if needed
    if (meltLevel > 30) {
      animations.push('melting_effect');
    }
    
    // Add glow based on energy
    const { energy } = useCompanionStore.getState();
    if (energy > 70) {
      animations.push('energy_glow');
    }
    
    return animations;
  }
}
```

### React Integration

```typescript
// Custom hook for animation state
function useCompanionAnimation() {
  const { mood, activity, meltLevel, energy } = useCompanionStore();
  const [animationEngine] = useState(() => new AnimationEngine());
  
  useEffect(() => {
    // Update animations based on state
    const animations = animationResolver.resolve(mood, activity, meltLevel);
    
    animations.forEach(animation => {
      animationEngine.play(animation);
    });
  }, [mood, activity, meltLevel]);
  
  useAnimationFrame((deltaTime) => {
    animationEngine.update(deltaTime);
  });
  
  return {
    engine: animationEngine,
    currentAnimations: animationEngine.getActiveAnimations()
  };
}

// Animation component
function SkellyAnimation() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { engine } = useCompanionAnimation();
  
  useEffect(() => {
    if (!canvasRef.current) return;
    
    // Initialize renderer
    const renderer = new WebGLRenderer({
      canvas: canvasRef.current,
      alpha: true,
      antialias: true,
      powerPreference: 'low-power'
    });
    
    engine.setRenderer(renderer);
    
    return () => {
      renderer.dispose();
    };
  }, []);
  
  return (
    <canvas
      ref={canvasRef}
      className="companion-canvas"
      width={200}
      height={200}
    />
  );
}
```

## Testing Strategy

### State Management Tests

```typescript
describe('CompanionStore', () => {
  it('should update mood with side effects', () => {
    const { updateMood } = useCompanionStore.getState();
    
    updateMood(MoodState.MELTING);
    
    const state = useCompanionStore.getState();
    expect(state.mood).toBe(MoodState.MELTING);
    expect(state.meltLevel).toBeGreaterThan(0);
  });
  
  it('should manage intervention cooldown', () => {
    const { recordInteraction, canIntervene } = useCompanionStore.getState();
    
    recordInteraction();
    expect(canIntervene()).toBe(false);
    
    // Fast-forward time
    jest.advanceTimersByTime(900000); // 15 minutes
    expect(canIntervene()).toBe(true);
  });
});
```

### Animation Tests

```typescript
describe('AnimationEngine', () => {
  it('should blend animations smoothly', async () => {
    const engine = new AnimationEngine();
    
    await engine.play('idle_breathing');
    const promise = engine.play('happy_bounce', true);
    
    // Check mid-transition
    await delay(250);
    const activeAnimations = engine.getActiveAnimations();
    expect(activeAnimations).toHaveLength(2);
    expect(activeAnimations[0].blendWeight).toBeLessThan(1);
    expect(activeAnimations[1].blendWeight).toBeGreaterThan(0);
    
    await promise;
    expect(engine.getActiveAnimations()).toHaveLength(1);
  });
  
  it('should respect performance constraints', () => {
    const engine = new AnimationEngine();
    const monitor = engine.getPerformanceMonitor();
    
    // Simulate low performance
    monitor.reportFPS(15);
    monitor.reportCPU(3);
    
    expect(engine.getQualityLevel()).toBe(AnimationQuality.LOW);
  });
});
```

## Conclusion

This state management and animation system provides a robust foundation for the Skelly-Jelly companion. The reactive state management ensures smooth synchronization with the rest of the application, while the sophisticated animation system delivers delightful, performant animations that respond to user states and maintain the 2% CPU usage target.