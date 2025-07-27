# Visual Implementation Guide
## From Placeholder to Skeleton Companion

## 🎯 Current State Analysis

**Placeholder Implementation**: The AnimationCanvas component currently shows a gray box (`#f0f0f0`) with text instead of the skeleton companion.

**Technical Infrastructure Ready**:
- ✅ AnimationEngine with 8 predefined animations
- ✅ Three.js scene management and skeleton support
- ✅ Performance optimization with frame skipping
- ✅ State management via CompanionStore
- ✅ Event-driven animation triggers

**Missing Visual Layer**: Only the actual 3D model assets and Three.js rendering integration.

## 🔄 Implementation Phases

### Phase 1: Basic 3D Rendering (Replace Placeholder)
**Goal**: Replace gray box with actual 3D skeleton model

**Required Changes**:
```typescript
// src/components/AnimationCanvas/index.tsx
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader';
import { useRef, useEffect } from 'react';

// Replace placeholder div with Three.js canvas
const canvasRef = useRef<HTMLCanvasElement>(null);
const sceneRef = useRef<THREE.Scene>();
const rendererRef = useRef<THREE.WebGLRenderer>();
```

**Assets Needed**:
- `src/assets/models/skelly-base.glb` - Basic skeleton model
- Basic material textures

### Phase 2: Animation Integration
**Goal**: Connect AnimationEngine to visual rendering

**Integration Points**:
- Load animations from AnimationEngine into Three.js mixer
- Map ADHD states to animation names
- Handle smooth transitions between states

**Code Changes**:
```typescript
// Connect to existing AnimationEngine
import { AnimationEngine } from '../../animation/AnimationEngine';

const animationEngine = new AnimationEngine(scene, skeleton, config);
animationEngine.play('idle_breathing'); // Default state
```

### Phase 3: Melt State Morphing
**Goal**: Visual representation of melt levels

**Technical Approach**:
- Morph target animations for melting
- Material property changes (transparency, vertex displacement)
- Particle effects for transitions

### Phase 4: Mood Visual Variants
**Goal**: Color and expression changes based on ADHD state

**Implementation**:
- Material swapping for mood colors
- Facial expression morphs
- Ambient lighting changes

## 🛠️ Specific Code Changes Required

### 1. Replace AnimationCanvas Placeholder

**Current Code** (lines 20-35):
```typescript
<div style={{
  backgroundColor: '#f0f0f0',
  border: '1px solid #ccc',
  // ... placeholder styling
}}>
  Animation Canvas ({width}x{height})
</div>
```

**New Implementation**:
```typescript
<canvas
  ref={canvasRef}
  width={width}
  height={height}
  style={{
    display: visible ? 'block' : 'none',
    width: '100%',
    height: '100%'
  }}
/>
```

### 2. Three.js Scene Initialization

**Add to useEffect**:
```typescript
useEffect(() => {
  if (!canvasRef.current) return;

  // Initialize Three.js
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer({ 
    canvas: canvasRef.current,
    antialias: true,
    alpha: true 
  });

  renderer.setSize(width, height);
  renderer.setClearColor(0x000000, 0); // Transparent background

  // Load skeleton model
  const loader = new GLTFLoader();
  loader.load('/assets/models/skelly-base.glb', (gltf) => {
    const skeleton = gltf.scene;
    scene.add(skeleton);
    
    // Initialize AnimationEngine
    const animEngine = new AnimationEngine(scene, skeleton.skeleton);
    animEngine.play('idle_breathing');
  });

  // Lighting
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.4);
  scene.add(ambientLight, directionalLight);

  // Animation loop
  const animate = () => {
    requestAnimationFrame(animate);
    animEngine.update(clock.getDelta());
    renderer.render(scene, camera);
  };
  animate();

  return () => {
    renderer.dispose();
  };
}, [width, height]);
```

### 3. State-to-Animation Mapping

**Connect ADHD states to animations**:
```typescript
const stateAnimationMap = {
  'flow': 'focused_breathing',
  'hyperfocus': 'focused_breathing', 
  'distracted': 'tired_sway',
  'productive_switching': 'happy_bounce',
  'perseveration': 'melting_heavy',
  'idle': 'idle_breathing'
};

// In component effect
useEffect(() => {
  const currentState = companionStore.adhdState;
  const animationName = stateAnimationMap[currentState];
  animationEngine?.play(animationName);
}, [companionStore.adhdState]);
```

## 📋 Asset Integration Checklist

### Development Phase Assets
- [ ] **skelly-base.glb** - Basic skeleton model with rigging
- [ ] **basic-texture.png** - Simple diffuse texture
- [ ] **Test with placeholder geometry first** (box → sphere → skeleton)

### Production Phase Assets  
- [ ] **Complete melt state models** (5 levels: solid → puddle)
- [ ] **Mood texture variants** (happy, concerned, excited, sleepy)
- [ ] **Animation clips** embedded in glTF files
- [ ] **Particle effect textures** for celebrations

### Performance Validation
- [ ] **Model loads in <500ms**
- [ ] **Maintains 30fps target**
- [ ] **Memory usage <50MB**
- [ ] **CPU overhead <2%**

## 🚀 Development Workflow

### Step 1: Create Basic Three.js Setup
1. Install Three.js dependencies
2. Replace placeholder div with canvas
3. Initialize basic scene with cube (test geometry)
4. Verify rendering pipeline works

### Step 2: Load First Asset
1. Create basic skeleton model in Blender
2. Export as .glb format
3. Load into Three.js scene
4. Replace cube with skeleton model

### Step 3: Connect Animation System
1. Link AnimationEngine to Three.js mixer
2. Test basic idle animation
3. Verify performance targets

### Step 4: State Integration
1. Connect to CompanionStore state changes
2. Map ADHD states to animations
3. Test state transitions

### Step 5: Polish and Optimize
1. Add melt state morphing
2. Implement mood variations
3. Performance optimization

## 🔧 Troubleshooting Guide

### Common Issues

**Model Not Loading**:
- Verify asset path is correct
- Check browser console for errors
- Ensure glTF file is valid

**Performance Issues**:
- Reduce polygon count (<5000 triangles)
- Optimize texture sizes (512x512 max)
- Enable performance monitoring in AnimationEngine

**Animation Not Playing**:
- Verify skeleton is properly rigged
- Check AnimationEngine state management
- Ensure animations are embedded in glTF

**State Changes Not Reflected**:
- Verify CompanionStore subscription
- Check state-to-animation mapping
- Debug animation transition logic

## 📈 Success Metrics

**Technical Success**:
- [ ] Skeleton model renders correctly
- [ ] Animations play smoothly at 30fps
- [ ] State changes trigger appropriate animations
- [ ] Performance targets maintained

**User Experience Success**:
- [ ] Skeleton appears friendly and non-distracting
- [ ] Melt states clearly represent focus levels
- [ ] Mood changes are noticeable but subtle
- [ ] Overall experience feels supportive

## 🎯 Next Steps for Implementation

1. **Create basic Three.js setup** in AnimationCanvas
2. **Test with simple geometry** (cube/sphere) first
3. **Load first skeleton asset** when available
4. **Connect to animation system** progressively
5. **Add state integration** once visual rendering works
6. **Iterate based on feel** and user feedback

The technical foundation is solid - the main work is replacing the placeholder rendering with actual Three.js scene management and loading the visual assets Chris will provide! 🦴✨