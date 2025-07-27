# Animation & Interaction Specifications
## Skelly Companion Behavioral Framework

## 🎭 Animation Categories

### Core Behavioral Animations

**Idle States** (Always Active):
```yaml
idle_breathing:
  description: "Gentle rise and fall breathing animation"
  duration: 3 seconds
  loop: true
  triggers: Default state, no user activity
  visual_cues: Subtle scale changes (1.0 → 1.02 → 1.0)
  
focused_breathing: 
  description: "Slower, more controlled breathing during focus"
  duration: 4 seconds  
  loop: true
  triggers: Flow state, hyperfocus detection
  visual_cues: Minimal movement, centered posture
```

**Emotional Response Animations**:
```yaml
happy_bounce:
  description: "Joyful bouncing with squash-and-stretch"
  duration: 1 second
  loop: false
  triggers: Task completion, achievement unlocked
  visual_cues: Upward movement with scale deformation
  
tired_sway:
  description: "Gentle swaying indicating fatigue"
  duration: 2.5 seconds
  loop: true  
  triggers: Extended work periods, low energy detection
  visual_cues: Side-to-side movement, slight droop
  
celebration_dance:
  description: "Energetic celebration sequence"
  duration: 2 seconds
  loop: false
  triggers: Major milestones, streak achievements  
  visual_cues: Multi-axis movement, scaling effects
```

**State Transition Animations**:
```yaml
melting_heavy:
  description: "Significant melting for high distraction"
  duration: 2 seconds
  loop: true
  triggers: Perseveration state, high distraction
  visual_cues: Vertical compression, horizontal expansion
  
melting_medium:
  description: "Moderate melting for mild distraction"  
  duration: 1.5 seconds
  loop: true
  triggers: Productive switching, mild distraction
  visual_cues: Subtle shape deformation
  
interaction_wave:
  description: "Friendly acknowledgment of user interaction"
  duration: 1.2 seconds
  loop: false
  triggers: User clicks, drag interactions
  visual_cues: Rotation and scale changes
```

## 🎯 ADHD State Mapping

### State-to-Animation Correspondence

| ADHD State | Primary Animation | Secondary Animation | Transition Effect |
|------------|------------------|-------------------|------------------|
| **Flow** | `focused_breathing` | `happy_bounce` (occasional) | Smooth fade-in |
| **Hyperfocus** | `focused_breathing` | Static pose variants | Minimal movement |
| **Productive Switching** | `idle_breathing` | `happy_bounce` | Quick transitions |
| **Distracted** | `tired_sway` | `melting_medium` | Gradual morphing |
| **Perseveration** | `melting_heavy` | `tired_sway` | Slow fade |
| **Idle/Break** | `idle_breathing` | `celebration_dance` | Reset to baseline |

### Melt Level Visual Progression

**Melt Level 1.0 (Solid - Peak Focus)**:
- Animation: `focused_breathing`
- Visual: Crisp edges, upright posture
- Colors: Bright, clear bone white
- Movement: Minimal, controlled

**Melt Level 0.75 (Slight Melt - Good Focus)**:
- Animation: `idle_breathing` 
- Visual: Slightly softer edges
- Colors: Warm bone tones
- Movement: Gentle breathing

**Melt Level 0.5 (Half Melt - Moderate Distraction)**:
- Animation: `melting_medium`
- Visual: Rounded forms, slight droop
- Colors: Subdued tones
- Movement: Swaying, deformation

**Melt Level 0.25 (Very Melty - High Distraction)**:
- Animation: `melting_heavy`
- Visual: Significant deformation
- Colors: Muted, gray tones
- Movement: Heavy swaying, morphing

**Melt Level 0.0 (Puddle - Break Time)**:
- Animation: `idle_breathing` (adapted for puddle)
- Visual: Flat, spread out form
- Colors: Translucent, flowing
- Movement: Gentle ripples

## 🎮 Interaction Specifications

### User Input Responses

**Click Interactions**:
```typescript
onClick: {
  animation: 'interaction_wave',
  duration: 1200,
  acknowledgment: true,
  sound_effect: 'click-response.mp3'
}
```

**Drag Interactions**:
```typescript
onDragStart: {
  animation: 'drag-start',
  visual_feedback: 'highlight_glow',
  physics: 'follow_cursor'
}

onDragEnd: {
  animation: 'drag-end', 
  return_animation: 'bounce_settle',
  position_memory: true
}
```

**Hover States**:
```typescript
onHover: {
  subtle_glow: true,
  scale_factor: 1.05,
  animation_speed: 1.1,
  transition_time: 200
}
```

### Contextual Behaviors

**Time-Based Triggers**:
- **Morning**: Energetic greeting animation
- **Afternoon**: Steady, supportive presence  
- **Evening**: Gentle, winding-down movements
- **Long Sessions**: Periodic stretch suggestions

**Activity-Based Triggers**:
- **Typing Detected**: Focused attention pose
- **Mouse Activity**: Responsive following
- **Inactivity**: Gentle attention prompts
- **App Switching**: Transition acknowledgment

## 🎨 Visual Effects Specifications

### Particle Systems

**Celebration Effects**:
```yaml
sparkle_burst:
  particle_count: 15-25
  duration: 2000ms
  colors: ['#FFD700', '#FFA500', '#FF69B4']
  spawn_pattern: radial_burst
  physics: gravity + fade
  
success_glow:
  effect_type: ambient_glow
  color: '#00FF88'
  intensity: 0.3
  duration: 3000ms
  fade_pattern: pulse_out
```

**State Transition Effects**:
```yaml
melt_transition:
  shader_effect: vertex_displacement
  duration: 1500ms
  interpolation: ease_in_out
  particle_trail: drip_effect
  
solidify_transition:
  shader_effect: reverse_displacement
  duration: 1000ms
  interpolation: bounce_out
  particle_effect: sparkle_reform
```

### Material Properties

**Base Materials**:
```yaml
bone_material:
  base_color: '#F5F5DC' # Bone white
  roughness: 0.4
  metallic: 0.0
  transparency: false
  
melt_material:
  base_color: '#E6E6FA' # Lavender tint
  roughness: 0.2  
  metallic: 0.0
  transparency: 0.1-0.8 # Variable based on melt level
  
mood_variants:
  happy: '#FFE4B5'    # Moccasin
  excited: '#FFB6C1'  # Light pink  
  concerned: '#D3D3D3' # Light gray
  sleepy: '#E0E0E0'   # Gainsboro
```

### Lighting Interactions

**Ambient Lighting**:
- **Focus States**: Bright, clear lighting
- **Distracted States**: Softer, diffused light
- **Break Time**: Warm, relaxing glow
- **Celebration**: Dynamic, colorful lighting

**Shadow Effects**:
- **Solid State**: Crisp, defined shadows
- **Melting States**: Soft, blurred shadows
- **Puddle State**: Minimal shadow footprint

## ⚡ Performance Specifications

### Animation Performance Targets

**Frame Rate Requirements**:
- **Target**: 30 FPS consistent
- **Minimum**: 24 FPS acceptable
- **Degradation**: Auto-reduce complexity below 20 FPS

**Resource Usage Limits**:
```yaml
cpu_usage: <2% average
memory_footprint: <50MB total
gpu_memory: <20MB texture data
animation_tracks: <10 simultaneous
```

**Quality Degradation Levels**:
```yaml
high_quality:
  animation_smoothness: 30fps
  particle_effects: enabled
  shader_complexity: full
  texture_resolution: 1024x1024
  
medium_quality:
  animation_smoothness: 24fps
  particle_effects: reduced
  shader_complexity: simplified  
  texture_resolution: 512x512
  
low_quality:
  animation_smoothness: 15fps
  particle_effects: disabled
  shader_complexity: basic
  texture_resolution: 256x256
```

### Optimization Strategies

**Animation Optimization**:
- Keyframe reduction for distant viewing
- Level-of-detail (LOD) based on window size
- Animation blending pool management
- Unused animation track cleanup

**Rendering Optimization**:
- Occlusion culling for hidden parts
- Texture atlas for multiple materials
- Instanced rendering for particles
- Frustum culling optimization

## 🎵 Audio Integration

### Sound Design Specifications

**Notification Sounds**:
```yaml
gentle_notification:
  file: notification-soft.mp3
  volume: 0.3 (30%)
  duration: 800ms
  frequency_range: 200-2000Hz
  mood: calm, non-intrusive
  
achievement_sound:
  file: achievement.mp3  
  volume: 0.4 (40%)
  duration: 1200ms
  frequency_range: 400-4000Hz
  mood: celebratory, uplifting
  
click_response:
  file: click-response.mp3
  volume: 0.2 (20%) 
  duration: 300ms
  frequency_range: 800-1600Hz
  mood: acknowledging, friendly
```

**Ambient Audio**:
```yaml
focus_ambient:
  file: focus-ambient.mp3
  volume: 0.1 (10%)
  loop: true
  duration: 5-10 minutes
  mood: concentration-supporting
  
break_ambient:
  file: break-ambient.mp3
  volume: 0.15 (15%)
  loop: true  
  duration: 3-8 minutes
  mood: relaxing, restorative
```

### Audio-Visual Synchronization

**Synchronized Events**:
- Animation peaks align with audio emphasis
- Particle effects triggered by audio cues
- Breathing animations sync with ambient rhythm
- Transition sounds match visual morphing

## 🔄 Implementation Priority

### Phase 1: Core Animations (MVP)
1. `idle_breathing` - Essential baseline
2. `focused_breathing` - Focus state representation
3. `tired_sway` - Distraction indicator
4. Basic melt level morphing

### Phase 2: Interaction Layer
1. `interaction_wave` - User acknowledgment
2. Click and hover responses  
3. Basic particle celebration effects
4. Audio feedback integration

### Phase 3: Full Behavioral System
1. `celebration_dance` - Achievement celebration
2. Complete melt transition system
3. Mood-based material variations
4. Contextual behavioral triggers

### Phase 4: Polish & Optimization
1. Advanced particle systems
2. Dynamic lighting effects
3. Performance optimization
4. Accessibility adaptations

## 📊 Success Metrics

**Animation Quality**:
- Smooth transitions between states
- Believable character personality
- Performance targets maintained
- User engagement without distraction

**User Experience**:
- Clear state communication
- Helpful without intrusive
- Emotionally supportive presence
- Customization responsiveness

The animation system should make Skelly feel like a genuine companion that understands and responds appropriately to the user's ADHD patterns! 🦴✨