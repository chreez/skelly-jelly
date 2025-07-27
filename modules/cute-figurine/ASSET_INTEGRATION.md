# Asset Integration Technical Guide

## 🔧 Technical Requirements

### Supported Asset Formats

**3D Models**:
- **glTF 2.0** (`.glb`, `.gltf`) - **Recommended**
  - Built-in Three.js support
  - Embedded textures and animations
  - PBR material support
  - Efficient binary format
- **FBX** (`.fbx`) - Fallback option
  - Requires FBXLoader
  - Larger file sizes
  - Good for complex rigs

**Textures**:
- **PNG** - Transparency support for UI elements
- **JPEG** - Diffuse/color maps
- **WebP** - Modern format, smaller sizes

**Audio**:
- **MP3** - Wide browser support
- **OGG** - Open format, good compression
- **WAV** - Uncompressed, high quality

### Performance Constraints

**Model Specifications**:
```yaml
Polygon Count: <5,000 triangles (target: 2,000)
Texture Resolution: 512x512 (max: 1024x1024)
Animation Tracks: <20 simultaneous
File Size: <2MB total per model
Memory Usage: <50MB in GPU memory
```

**Animation Specifications**:
```yaml
Frame Rate: 30fps (smooth) or 15fps (performance)
Duration: 2-10 seconds per loop
Keyframes: Optimized, remove redundant frames
Compression: Use glTF animation compression
```

## 📁 Asset Organization

### Directory Structure
```
src/assets/
├── models/
│   ├── skelly/
│   │   ├── base.glb                 # Main skeleton model
│   │   ├── expressions/             # Facial expression variants
│   │   │   ├── happy.glb
│   │   │   ├── concerned.glb
│   │   │   └── excited.glb
│   │   └── melt-states/            # Melting level variants
│   │       ├── solid.glb           # 100% solid
│   │       ├── slight-melt.glb     # 75% melty
│   │       ├── half-melt.glb       # 50% melty
│   │       ├── very-melt.glb       # 25% melty
│   │       └── puddle.glb          # 0% puddle state
├── textures/
│   ├── skeleton/
│   │   ├── base-diffuse.png        # Base color
│   │   ├── base-normal.png         # Normal map (optional)
│   │   ├── base-roughness.png      # Material properties
│   │   └── mood-atlas.png          # Mood color variations
│   └── particles/
│       ├── sparkle.png             # Celebration particles
│       └── glow.png                # Highlight effects
├── animations/
│   ├── core/
│   │   ├── idle.json               # Breathing, subtle movement
│   │   ├── sleep.json              # Rest state animation
│   │   └── alert.json              # Attention/focus state
│   ├── transitions/
│   │   ├── melt-down.json          # Solid → melty transition
│   │   ├── melt-up.json            # Melty → solid transition
│   │   └── mood-change.json        # Mood transition animations
│   └── interactions/
│       ├── drag-start.json         # When user starts dragging
│       ├── drag-end.json           # When user releases
│       ├── click-response.json     # Response to clicks
│       └── celebration.json        # Achievement/reward animation
└── sounds/
    ├── ui/
    │   ├── notification-soft.mp3   # Gentle notification
    │   ├── click-response.mp3      # Click feedback
    │   └── achievement.mp3         # Success/celebration
    └── ambient/
        ├── focus-ambient.mp3       # Background focus sounds
        └── break-ambient.mp3       # Break time ambience
```

### Asset Naming Convention
```
Format: [category]-[state]-[variant]-[version].extension

Examples:
- skelly-solid-happy-v1.glb
- texture-bone-diffuse-512.png
- anim-idle-breathing-30fps.json
- sound-notification-gentle-22k.mp3
```

## 🔌 Integration Code Examples

### Loading 3D Models

```typescript
// src/utils/AssetLoader.ts
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader';

export class SkellyAssetLoader {
  private loader = new GLTFLoader();
  private models = new Map<string, THREE.Group>();
  private textures = new Map<string, THREE.Texture>();

  async loadSkellyModel(meltLevel: number, mood: string): Promise<THREE.Group> {
    const modelPath = this.getModelPath(meltLevel, mood);
    
    if (this.models.has(modelPath)) {
      return this.models.get(modelPath)!.clone();
    }

    const gltf = await this.loader.loadAsync(modelPath);
    const model = gltf.scene;
    
    // Cache for reuse
    this.models.set(modelPath, model);
    
    return model.clone();
  }

  private getModelPath(meltLevel: number, mood: string): string {
    const meltState = this.getMeltStateName(meltLevel);
    return `/assets/models/skelly/${meltState}-${mood}.glb`;
  }

  private getMeltStateName(level: number): string {
    if (level >= 0.8) return 'solid';
    if (level >= 0.6) return 'slight-melt';
    if (level >= 0.4) return 'half-melt';
    if (level >= 0.2) return 'very-melt';
    return 'puddle';
  }
}
```

### Animation Integration

```typescript
// src/animation/SkellyAnimationController.ts
import { AnimationEngine } from './AnimationEngine';

export class SkellyAnimationController {
  constructor(private animationEngine: AnimationEngine) {}

  async playMeltTransition(fromLevel: number, toLevel: number): Promise<void> {
    const animationName = this.getTransitionAnimation(fromLevel, toLevel);
    await this.animationEngine.playAnimation(animationName, {
      duration: 2000, // 2 seconds
      easing: 'ease-in-out',
      loop: false
    });
  }

  async setIdleState(meltLevel: number, mood: string): Promise<void> {
    const idleAnimation = `idle-${this.getMeltStateName(meltLevel)}`;
    await this.animationEngine.playAnimation(idleAnimation, {
      loop: true,
      fadeIn: 500
    });
  }

  private getTransitionAnimation(from: number, to: number): string {
    if (to > from) return 'melt-up';
    if (to < from) return 'melt-down';
    return 'mood-change';
  }
}
```

### Texture Swapping

```typescript
// src/materials/SkellyMaterialController.ts
export class SkellyMaterialController {
  private materials = new Map<string, THREE.Material>();

  async updateMoodMaterial(model: THREE.Group, mood: string): Promise<void> {
    const material = await this.getMoodMaterial(mood);
    
    model.traverse((child) => {
      if (child instanceof THREE.Mesh) {
        child.material = material;
      }
    });
  }

  private async getMoodMaterial(mood: string): Promise<THREE.Material> {
    const cacheKey = `mood-${mood}`;
    
    if (this.materials.has(cacheKey)) {
      return this.materials.get(cacheKey)!;
    }

    const texture = await this.loadTexture(`/assets/textures/skeleton/mood-${mood}.png`);
    const material = new THREE.MeshStandardMaterial({
      map: texture,
      transparent: true
    });

    this.materials.set(cacheKey, material);
    return material;
  }
}
```

## 🎯 Asset Pipeline Workflow

### 1. Asset Creation
```bash
# Recommended tools:
- Blender (free, glTF export)
- Maya (FBX export + glTF plugin)
- 3ds Max (FBX export + glTF plugin)
- Substance Painter (texture creation)
```

### 2. Asset Optimization
```bash
# glTF optimization
npm install -g gltf-pipeline
gltf-pipeline -i model.gltf -o optimized.glb --draco.compressionLevel 7

# Texture optimization
npm install -g imagemin-cli
imagemin textures/*.png --out-dir=optimized/ --plugin=pngquant
```

### 3. Asset Validation
```typescript
// Test script to validate assets
export async function validateAssets(): Promise<void> {
  const requiredAssets = [
    'models/skelly/solid-happy.glb',
    'models/skelly/puddle-happy.glb',
    'textures/skeleton/base-diffuse.png'
  ];

  for (const asset of requiredAssets) {
    try {
      await fetch(`/assets/${asset}`);
      console.log(`✅ ${asset} - OK`);
    } catch (error) {
      console.error(`❌ ${asset} - Missing`);
    }
  }
}
```

## 📋 Asset Quality Checklist

**Before Integration**:
- [ ] Model polycount under 5,000 triangles
- [ ] Textures power-of-2 dimensions (512x512, 1024x1024)
- [ ] Animations under 10 seconds duration
- [ ] File sizes under 2MB each
- [ ] glTF format validation passes
- [ ] All required melt states included
- [ ] Mood variants for each melt level

**After Integration**:
- [ ] Models load without errors
- [ ] Animations play smoothly at 30fps
- [ ] Memory usage under 50MB
- [ ] No texture artifacts or UV issues
- [ ] Responsive to user interactions
- [ ] Performance targets maintained (<2% CPU)

## 🚀 Deployment Considerations

**Asset Loading Strategy**:
- Load base model immediately
- Lazy load mood variants
- Preload common transitions
- Use asset caching for performance

**Progressive Enhancement**:
- Start with basic solid skeleton
- Add melt states incrementally
- Mood variants as Phase 2
- Advanced animations as Phase 3

**Fallback Strategy**:
- 2D sprite fallback if 3D fails
- Static image if animations fail
- Placeholder box as last resort