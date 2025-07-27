import * as THREE from 'three';
import { AnimationState, Keyframe, TransitionConfig } from '../types/animation.types';
import { TransitionManager } from './TransitionManager';
import { BlendTree } from './BlendTree';
import { easings } from '../utils/easing';

export interface AnimationEngineConfig {
  fps: number;
  quality: 'low' | 'medium' | 'high';
  enableShaders: boolean;
  reducedMotion: boolean;
  enablePerformanceOptimization: boolean;
}

export interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  memoryUsage: number;
  activeTracks: number;
  skippedFrames: number;
}

export class AnimationEngine {
  private clock: THREE.Clock;
  private mixer: THREE.AnimationMixer;
  private animations: Map<string, THREE.AnimationClip>;
  private currentAction: THREE.AnimationAction | null = null;
  private blendTree: BlendTree;
  private transitionManager: TransitionManager;
  private config: AnimationEngineConfig;

  // Performance monitoring
  private frameCount = 0;
  private lastFrameTime = 0;
  private performanceMetrics: PerformanceMetrics;
  private targetFrameTime: number;
  private lastUpdate = 0;

  // Animation state
  private isPlaying = false;
  private currentStateName = 'idle';
  private timeScale = 1;

  // On-demand rendering
  private needsUpdate = false;
  private onInvalidate?: () => void;

  constructor(
    private scene: THREE.Scene,
    private skeleton: THREE.Skeleton,
    config: Partial<AnimationEngineConfig> = {},
    onInvalidate?: () => void
  ) {
    this.config = {
      fps: 30,
      quality: 'medium',
      enableShaders: true,
      reducedMotion: false,
      enablePerformanceOptimization: true,
      ...config,
    };

    this.onInvalidate = onInvalidate;
    this.targetFrameTime = 1000 / this.config.fps;

    this.clock = new THREE.Clock();
    this.mixer = new THREE.AnimationMixer(skeleton.bones[0]);
    this.animations = new Map();
    this.blendTree = new BlendTree();
    this.transitionManager = new TransitionManager(this.mixer);

    this.performanceMetrics = {
      fps: this.config.fps,
      frameTime: this.targetFrameTime,
      memoryUsage: 0,
      activeTracks: 0,
      skippedFrames: 0,
    };

    this.loadAnimations();
    this.setupEventListeners();
  }

  private setupEventListeners() {
    // Listen for animation state changes
    this.mixer.addEventListener('finished', (event) => {
      this.handleAnimationFinished(event);
    });

    this.mixer.addEventListener('loop', (event) => {
      this.handleAnimationLoop(event);
    });
  }

  private loadAnimations() {
    // Create all predefined animations
    const animationConfigs = [
      { name: 'idle_breathing', duration: 3, loop: true },
      { name: 'happy_bounce', duration: 1, loop: false },
      { name: 'melting_heavy', duration: 2, loop: true },
      { name: 'melting_medium', duration: 1.5, loop: true },
      { name: 'focused_breathing', duration: 4, loop: true },
      { name: 'tired_sway', duration: 2.5, loop: true },
      { name: 'celebration_dance', duration: 2, loop: false },
      { name: 'interaction_wave', duration: 1.2, loop: false },
    ];

    animationConfigs.forEach((config) => {
      const clip = this.createAnimationClip(config.name, config.duration, config.loop);
      if (clip) {
        this.animations.set(config.name, clip);
      }
    });
  }

  private createAnimationClip(
    name: string,
    duration: number,
    loop: boolean
  ): THREE.AnimationClip | null {
    const tracks: THREE.KeyframeTrack[] = [];

    switch (name) {
      case 'idle_breathing':
        tracks.push(...this.createIdleBreathingTracks(duration));
        break;
      case 'happy_bounce':
        tracks.push(...this.createHappyBounceTracks(duration));
        break;
      case 'melting_heavy':
        tracks.push(...this.createMeltingTracks(duration, 0.8));
        break;
      case 'melting_medium':
        tracks.push(...this.createMeltingTracks(duration, 0.5));
        break;
      case 'focused_breathing':
        tracks.push(...this.createFocusedBreathingTracks(duration));
        break;
      case 'tired_sway':
        tracks.push(...this.createTiredSwayTracks(duration));
        break;
      case 'celebration_dance':
        tracks.push(...this.createCelebrationTracks(duration));
        break;
      case 'interaction_wave':
        tracks.push(...this.createInteractionTracks(duration));
        break;
      default:
        console.warn(`Unknown animation: ${name}`);
        return null;
    }

    if (tracks.length === 0) return null;

    return new THREE.AnimationClip(name, duration, tracks);
  }

  private createIdleBreathingTracks(duration: number): THREE.KeyframeTrack[] {
    const times = [0, duration * 0.5, duration];
    const scaleValues = [
      1,
      1,
      1, // Start
      1.02,
      1.02,
      1.02, // Inhale
      1,
      1,
      1, // Exhale
    ];

    // Add slight vertical movement
    const positionValues = [0, 0, 0, 0, 0.01, 0, 0, 0, 0];

    return [
      new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth),
      new THREE.VectorKeyframeTrack('.position', times, positionValues, THREE.InterpolateSmooth),
    ];
  }

  private createHappyBounceTracks(duration: number): THREE.KeyframeTrack[] {
    const times = [0, 0.2, 0.4, 0.6, 0.8, 1.0];

    // Bounce animation with squash and stretch
    const positionValues = [
      0,
      0,
      0, // Start
      0,
      0.3,
      0, // Up
      0,
      0,
      0, // Down
      0,
      0.15,
      0, // Small bounce
      0,
      0,
      0, // Down
      0,
      0,
      0, // Rest
    ];

    const scaleValues = [
      1,
      1,
      1, // Normal
      0.9,
      1.3,
      0.9, // Stretch up
      1.2,
      0.8,
      1.2, // Squash down
      1,
      1,
      1, // Normal
      1.05,
      0.95,
      1.05, // Slight squash
      1,
      1,
      1, // Normal
    ];

    return [
      new THREE.VectorKeyframeTrack('.position', times, positionValues, THREE.InterpolateSmooth),
      new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth),
    ];
  }

  private createMeltingTracks(duration: number, intensity: number): THREE.KeyframeTrack[] {
    const times = [0, duration];

    // Melting effect - scale down vertically, expand horizontally
    const scaleValues = [1, 1, 1, 1 + intensity * 0.3, 1 - intensity * 0.4, 1 + intensity * 0.2];

    // Slight sinking
    const positionValues = [0, 0, 0, 0, -intensity * 0.1, 0];

    return [
      new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth),
      new THREE.VectorKeyframeTrack('.position', times, positionValues, THREE.InterpolateSmooth),
    ];
  }

  private createFocusedBreathingTracks(duration: number): THREE.KeyframeTrack[] {
    const times = [0, duration * 0.3, duration * 0.7, duration];

    // Slower, more controlled breathing
    const scaleValues = [1, 1, 1, 1.01, 1.01, 1.01, 1.015, 1.015, 1.015, 1, 1, 1];

    return [new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth)];
  }

  private createTiredSwayTracks(duration: number): THREE.KeyframeTrack[] {
    const times = [0, duration * 0.5, duration];

    // Gentle swaying motion
    const positionValues = [0, 0, 0, 0.02, -0.01, 0, 0, 0, 0];

    // Slight droop
    const scaleValues = [1, 1, 1, 1, 0.98, 1, 1, 1, 1];

    return [
      new THREE.VectorKeyframeTrack('.position', times, positionValues, THREE.InterpolateSmooth),
      new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth),
    ];
  }

  private createCelebrationTracks(duration: number): THREE.KeyframeTrack[] {
    const times = [0, 0.2, 0.4, 0.6, 0.8, 1.0];

    // Energetic celebration dance
    const positionValues = [
      0, 0, 0, -0.1, 0.2, 0, 0.1, 0.3, 0, -0.05, 0.25, 0, 0.05, 0.15, 0, 0, 0, 0,
    ];

    const scaleValues = [
      1, 1, 1, 1.1, 1.1, 1.1, 0.9, 1.2, 0.9, 1.05, 1.05, 1.05, 1.1, 0.9, 1.1, 1, 1, 1,
    ];

    return [
      new THREE.VectorKeyframeTrack('.position', times, positionValues, THREE.InterpolateSmooth),
      new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth),
    ];
  }

  private createInteractionTracks(duration: number): THREE.KeyframeTrack[] {
    const times = [0, 0.3, 0.6, 1.0];

    // Friendly waving motion
    const rotationValues = [0, 0, 0, 0, 0, 0.1, 0, 0, -0.1, 0, 0, 0];

    const scaleValues = [1, 1, 1, 1.05, 1.05, 1.05, 1.02, 1.02, 1.02, 1, 1, 1];

    return [
      new THREE.VectorKeyframeTrack('.rotation', times, rotationValues, THREE.InterpolateSmooth),
      new THREE.VectorKeyframeTrack('.scale', times, scaleValues, THREE.InterpolateSmooth),
    ];
  }

  public play(
    animationName: string,
    options: {
      loop?: boolean;
      duration?: number;
      transition?: boolean;
      weight?: number;
      timeScale?: number;
    } = {}
  ): void {
    const clip = this.animations.get(animationName);
    if (!clip) {
      console.warn(`Animation ${animationName} not found`);
      return;
    }

    // Performance optimization: skip if reduced motion
    if (this.config.reducedMotion && animationName !== 'idle_breathing') {
      return;
    }

    const action = this.mixer.clipAction(clip);

    // Configure action
    action.reset();
    action.setLoop(options.loop !== false ? THREE.LoopRepeat : THREE.LoopOnce, Infinity);
    action.clampWhenFinished = !options.loop;
    action.weight = options.weight ?? 1;

    if (options.duration) {
      action.setDuration(options.duration / 1000);
    }

    if (options.timeScale) {
      action.timeScale = options.timeScale;
    }

    // Handle transition
    if (this.currentAction && options.transition !== false) {
      this.transitionManager.crossfade(this.currentAction, action, 0.3);
    } else {
      // Stop current action
      if (this.currentAction) {
        this.currentAction.stop();
      }
      action.play();
    }

    this.currentAction = action;
    this.currentStateName = animationName;
    this.isPlaying = true;
    this.invalidate();
  }

  public transition(toAnimation: string, duration: number = 0.5): Promise<void> {
    return new Promise((resolve) => {
      if (!this.currentAction) {
        this.play(toAnimation);
        resolve();
        return;
      }

      const toClip = this.animations.get(toAnimation);
      if (!toClip) {
        console.warn(`Animation ${toAnimation} not found`);
        resolve();
        return;
      }

      const toAction = this.mixer.clipAction(toClip);
      toAction.reset();
      toAction.setLoop(THREE.LoopRepeat, Infinity);

      this.transitionManager.crossfade(this.currentAction, toAction, duration).then(() => {
        this.currentAction = toAction;
        this.currentStateName = toAnimation;
        resolve();
      });
    });
  }

  public update(deltaTime: number): boolean {
    // Performance optimization: frame skipping
    if (this.config.enablePerformanceOptimization) {
      const now = performance.now();
      if (now - this.lastUpdate < this.targetFrameTime) {
        return false; // Skip this frame
      }
      this.lastUpdate = now;
    }

    const startTime = performance.now();

    // Update mixer
    if (this.isPlaying && this.mixer) {
      this.mixer.update(deltaTime * this.timeScale);
      this.transitionManager.update();
    }

    // Update performance metrics
    this.updatePerformanceMetrics(startTime);

    // Reset invalidation flag
    const hadUpdate = this.needsUpdate;
    this.needsUpdate = false;

    return hadUpdate;
  }

  private updatePerformanceMetrics(startTime: number): void {
    this.frameCount++;
    const frameTime = performance.now() - startTime;

    // Update metrics every 60 frames
    if (this.frameCount % 60 === 0) {
      this.performanceMetrics.frameTime = frameTime;
      this.performanceMetrics.fps = 1000 / (frameTime || 1);
      this.performanceMetrics.activeTracks = this.mixer.getRoot().children.length;

      // Memory estimation (rough)
      if (typeof performance !== 'undefined' && (performance as any).memory) {
        this.performanceMetrics.memoryUsage = (performance as any).memory.usedJSHeapSize;
      }

      // Auto-adjust quality based on performance
      if (this.config.enablePerformanceOptimization) {
        this.autoAdjustQuality();
      }
    }
  }

  private autoAdjustQuality(): void {
    const avgFrameTime = this.performanceMetrics.frameTime;
    const targetFrameTime = this.targetFrameTime;

    // If frame time is too high, reduce quality
    if (avgFrameTime > targetFrameTime * 1.5) {
      if (this.config.quality === 'high') {
        this.config.quality = 'medium';
      } else if (this.config.quality === 'medium') {
        this.config.quality = 'low';
      }
      this.performanceMetrics.skippedFrames++;
    }
    // If performance is good, try to increase quality
    else if (avgFrameTime < targetFrameTime * 0.7) {
      if (this.config.quality === 'low') {
        this.config.quality = 'medium';
      } else if (this.config.quality === 'medium') {
        this.config.quality = 'high';
      }
    }
  }

  private handleAnimationFinished(event: any): void {
    if (event.action === this.currentAction) {
      this.isPlaying = false;
      // Automatically transition to idle if no loop
      if (event.action.loop === THREE.LoopOnce) {
        this.play('idle_breathing', { transition: true });
      }
    }
  }

  private handleAnimationLoop(event: any): void {
    // Animation looped, can trigger events here if needed
  }

  public stop(): void {
    if (this.currentAction) {
      this.currentAction.stop();
      this.currentAction = null;
    }
    this.isPlaying = false;
  }

  public pause(): void {
    if (this.currentAction) {
      this.currentAction.paused = true;
    }
    this.isPlaying = false;
  }

  public resume(): void {
    if (this.currentAction) {
      this.currentAction.paused = false;
    }
    this.isPlaying = true;
    this.invalidate();
  }

  public setTimeScale(scale: number): void {
    this.timeScale = scale;
    if (this.currentAction) {
      this.currentAction.timeScale = scale;
    }
  }

  public getCurrentAnimation(): string {
    return this.currentStateName;
  }

  public isAnimationPlaying(): boolean {
    return this.isPlaying;
  }

  public getPerformanceMetrics(): PerformanceMetrics {
    return { ...this.performanceMetrics };
  }

  public updateConfig(config: Partial<AnimationEngineConfig>): void {
    this.config = { ...this.config, ...config };
    this.targetFrameTime = 1000 / this.config.fps;

    // Apply reduced motion immediately
    if (this.config.reducedMotion && this.currentStateName !== 'idle_breathing') {
      this.play('idle_breathing', { transition: true });
    }
  }

  private invalidate(): void {
    this.needsUpdate = true;
    this.onInvalidate?.();
  }

  public dispose(): void {
    this.mixer.uncacheRoot(this.mixer.getRoot());
    this.animations.clear();
    this.transitionManager.dispose();
    this.blendTree.dispose();
  }
}
