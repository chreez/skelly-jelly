# Skelly Companion - Required Assets & Creative Direction

## 🎨 Current State
The cute-figurine module is currently displaying a **placeholder gray box** instead of the actual skeleton companion. The technical infrastructure is complete (animation engine, Three.js integration, state management), but visual assets are needed.

## 🦴 Visual Asset Requirements

### 1. Skeleton Character Design

**Core Character Concept**:
- **"Melty Skeleton"** - A cute, friendly skeleton that can "melt" based on stress/focus states
- **Personality**: Supportive, non-judgmental, endearing companion for ADHD users
- **Aesthetic**: Cute/kawaii style, NOT scary or medical

**Required Character States**:
```
├── Base Forms
│   ├── Solid Skeleton (100% focus state)
│   ├── Slightly Melty (75% focus)  
│   ├── Half Melted (50% focus)
│   ├── Very Melty (25% focus)
│   └── Puddle Form (0% focus/break time)
├── Mood Variants (per melt level)
│   ├── Happy/Content
│   ├── Excited/Energized
│   ├── Concerned/Worried
│   ├── Sleepy/Resting
│   └── Celebrating/Proud
└── Interaction States
    ├── Being Dragged
    ├── Focused/Listening
    ├── Waving/Greeting
    └── Thinking/Processing
```

### 2. Asset Format Requirements

**3D Model Assets**:
- **Format**: `.glb` or `.gltf` (Three.js compatible)
- **Skeleton**: Rigged armature for animation
- **Polycount**: <5000 triangles (performance target)
- **Textures**: 512x512 or 1024x1024 max
- **Animations**: Embedded in model file

**2D Fallback Assets** (if 3D too complex):
- **Format**: `.png` with transparency
- **Resolution**: 200x200 base size (scalable)
- **Spritesheet**: Multiple states in single file
- **Animation Frames**: 8-12 frames per animation loop

**Audio Assets**:
- **Format**: `.mp3` or `.wav`
- **Quality**: 22kHz, mono, <50KB per file
- **Sounds**: Gentle, non-intrusive notification sounds

### 3. Creative Direction Needed

**Character Personality**:
- [ ] **Visual Style**: Cute/kawaii vs. realistic vs. minimalist?
- [ ] **Color Palette**: Bone white, warm tones, accent colors?
- [ ] **Facial Features**: Eyes, expression capabilities?
- [ ] **Proportions**: Chibi-style vs. anatomically proportioned?

**Melting Behavior**:
- [ ] **Melt Physics**: How should the skeleton "melt"? (dripping, dissolving, squishing?)
- [ ] **Transition Speed**: Quick snaps vs. smooth morphing?
- [ ] **Melt Direction**: Top-down, bottom-up, inside-out?
- [ ] **Puddle State**: Completely flat or retain some skeleton features?

**Mood Expression**:
- [ ] **Happy**: Glowing, bouncing, sparkles?
- [ ] **Concerned**: Tilted head, dimmed colors, slower movements?
- [ ] **Excited**: Faster animations, brighter colors, particle effects?
- [ ] **Sleepy**: Slower movements, yawning, droopy posture?

## 📋 Asset Checklist for Chris

### Immediate Needs (MVP)
- [ ] **Base skeleton model** (solid state)
- [ ] **2-3 melt levels** (50%, 25%, puddle)
- [ ] **Happy mood texture/variant**
- [ ] **Basic idle animation** (breathing, slight movement)

### Phase 2 Assets
- [ ] **All 5 melt levels** with smooth transitions
- [ ] **3-4 mood variants** per melt level
- [ ] **Interaction animations** (drag, click response)
- [ ] **Celebration particles/effects**
- [ ] **Gentle notification sounds**

### Phase 3 Polish
- [ ] **Custom shaders** for melt effects
- [ ] **Facial animations** (blinking, expressions)
- [ ] **Seasonal variants** (hat, accessories)
- [ ] **Customization options** (colors, accessories)

## 🛠️ Technical Integration Points

### Asset Locations
```
src/assets/
├── models/
│   ├── skelly-base.glb          # Main character model
│   ├── skelly-variants.glb      # Mood/state variants
│   └── particles.glb            # Celebration effects
├── textures/
│   ├── skeleton-diffuse.png     # Base color texture
│   ├── skeleton-normal.png      # Normal map (optional)
│   └── mood-atlas.png           # Mood expression atlas
├── animations/
│   ├── idle.json               # Base idle animation
│   ├── melt-transitions.json   # Melting animations
│   └── interactions.json       # Click/drag responses
└── sounds/
    ├── notification-gentle.mp3  # Soft notification
    ├── celebration.mp3          # Achievement sound
    └── ambient.mp3              # Background ambience
```

### Code Integration
The assets will be loaded into:
- **AnimationEngine.ts** - Handles 3D model and animations
- **AnimationCanvas** - Renders the visual companion
- **CompanionStore** - Manages state-based asset switching

## 🎯 Creative Decisions Needed

**High Priority Questions**:
1. **Art Style**: Should Skelly be cute/cartoon or more realistic?
2. **Melt Mechanism**: How should the melting visually work?
3. **Color Scheme**: Base colors and mood-based variations?
4. **Size/Proportions**: Head-to-body ratio, overall cuteness level?

**Medium Priority**:
5. **Facial Features**: Eyes style, mouth, expression range?
6. **Accessories**: Hat, bowtie, glasses for personality?
7. **Particles**: Sparkles, stars, or other celebration effects?
8. **Sounds**: Musical tones, nature sounds, or synthetic?

## 📞 Next Steps for Chris

1. **Review this document** and provide creative direction answers
2. **Choose art direction** (style, colors, proportions)
3. **Provide assets** or connect with an artist/designer
4. **Start with MVP assets** (1 base model + 2 melt states)
5. **Iterate** based on how it feels in the application

The technical foundation is ready - we just need the creative vision and visual assets to bring Skelly to life! 🦴✨