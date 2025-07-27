import * as THREE from 'three';

export interface TransitionConfig {
  from: string;
  to: string;
  duration: number;
  easing: (t: number) => number;
  conditions?: (() => boolean)[];
}

export interface ActiveTransition {
  fromAction: THREE.AnimationAction;
  toAction: THREE.AnimationAction;
  duration: number;
  startTime: number;
  easing: (t: number) => number;
  resolve: () => void;
}

export class TransitionManager {
  private activeTransitions: ActiveTransition[] = [];
  private transitionConfigs: Map<string, TransitionConfig[]> = new Map();
  private mixer: THREE.AnimationMixer;

  constructor(mixer: THREE.AnimationMixer) {
    this.mixer = mixer;
    this.setupDefaultTransitions();
  }

  private setupDefaultTransitions(): void {
    // Define common transition patterns
    const defaultTransitions: TransitionConfig[] = [
      // Mood transitions
      {
        from: 'idle_breathing',
        to: 'happy_bounce',
        duration: 0.3,
        easing: this.easeOutCubic,
      },
      {
        from: 'happy_bounce',
        to: 'idle_breathing',
        duration: 0.5,
        easing: this.easeInOutCubic,
      },
      {
        from: 'idle_breathing',
        to: 'focused_breathing',
        duration: 0.8,
        easing: this.easeInOutQuad,
      },
      {
        from: 'focused_breathing',
        to: 'idle_breathing',
        duration: 0.6,
        easing: this.easeOutQuad,
      },
      {
        from: 'idle_breathing',
        to: 'tired_sway',
        duration: 1.0,
        easing: this.easeInQuad,
      },
      {
        from: 'tired_sway',
        to: 'idle_breathing',
        duration: 0.8,
        easing: this.easeOutQuad,
      },
      // Melting transitions
      {
        from: 'idle_breathing',
        to: 'melting_medium',
        duration: 1.2,
        easing: this.easeInCubic,
      },
      {
        from: 'melting_medium',
        to: 'melting_heavy',
        duration: 0.8,
        easing: this.easeInQuad,
      },
      {
        from: 'melting_heavy',
        to: 'melting_medium',
        duration: 1.0,
        easing: this.easeOutQuad,
      },
      {
        from: 'melting_medium',
        to: 'idle_breathing',
        duration: 1.5,
        easing: this.easeOutCubic,
      },
      // Celebration transitions
      {
        from: 'idle_breathing',
        to: 'celebration_dance',
        duration: 0.2,
        easing: this.easeOutQuart,
      },
      {
        from: 'celebration_dance',
        to: 'happy_bounce',
        duration: 0.3,
        easing: this.easeInOutCubic,
      },
      // Interaction transitions
      {
        from: 'idle_breathing',
        to: 'interaction_wave',
        duration: 0.2,
        easing: this.easeOutQuad,
      },
      {
        from: 'interaction_wave',
        to: 'idle_breathing',
        duration: 0.3,
        easing: this.easeInOutQuad,
      },
    ];

    // Group transitions by source animation
    defaultTransitions.forEach((transition) => {
      const fromTransitions = this.transitionConfigs.get(transition.from) || [];
      fromTransitions.push(transition);
      this.transitionConfigs.set(transition.from, fromTransitions);
    });
  }

  public crossfade(
    fromAction: THREE.AnimationAction,
    toAction: THREE.AnimationAction,
    duration: number = 0.5,
    easing?: (t: number) => number
  ): Promise<void> {
    return new Promise((resolve) => {
      // Find appropriate transition config
      const fromName = fromAction.getClip().name;
      const toName = toAction.getClip().name;
      const config = this.findTransitionConfig(fromName, toName);

      const actualDuration = config?.duration || duration;
      const actualEasing = config?.easing || easing || this.easeInOutCubic;

      // Setup the transition
      toAction.reset();
      toAction.weight = 0;
      toAction.play();

      const transition: ActiveTransition = {
        fromAction,
        toAction,
        duration: actualDuration,
        startTime: this.mixer.time,
        easing: actualEasing,
        resolve,
      };

      this.activeTransitions.push(transition);
    });
  }

  public update(): void {
    const currentTime = this.mixer.time;

    // Update all active transitions
    for (let i = this.activeTransitions.length - 1; i >= 0; i--) {
      const transition = this.activeTransitions[i];
      const elapsed = currentTime - transition.startTime;
      const progress = Math.min(elapsed / transition.duration, 1);

      if (progress >= 1) {
        // Transition complete
        this.completeTransition(transition);
        this.activeTransitions.splice(i, 1);
      } else {
        // Update weights
        const easedProgress = transition.easing(progress);
        transition.fromAction.weight = 1 - easedProgress;
        transition.toAction.weight = easedProgress;
      }
    }
  }

  private completeTransition(transition: ActiveTransition): void {
    // Final weight adjustment
    transition.fromAction.weight = 0;
    transition.toAction.weight = 1;

    // Stop the from action
    transition.fromAction.stop();

    // Resolve the promise
    transition.resolve();
  }

  private findTransitionConfig(fromName: string, toName: string): TransitionConfig | null {
    const fromTransitions = this.transitionConfigs.get(fromName);
    if (!fromTransitions) return null;

    return fromTransitions.find((config) => config.to === toName) || null;
  }

  public addTransitionConfig(config: TransitionConfig): void {
    const fromTransitions = this.transitionConfigs.get(config.from) || [];

    // Remove existing transition with same from/to
    const existingIndex = fromTransitions.findIndex((t) => t.to === config.to);
    if (existingIndex !== -1) {
      fromTransitions.splice(existingIndex, 1);
    }

    fromTransitions.push(config);
    this.transitionConfigs.set(config.from, fromTransitions);
  }

  public removeTransitionConfig(from: string, to: string): boolean {
    const fromTransitions = this.transitionConfigs.get(from);
    if (!fromTransitions) return false;

    const index = fromTransitions.findIndex((config) => config.to === to);
    if (index !== -1) {
      fromTransitions.splice(index, 1);
      return true;
    }

    return false;
  }

  public canTransition(from: string, to: string): boolean {
    const config = this.findTransitionConfig(from, to);
    if (!config) return true; // Allow any transition if no config exists

    // Check conditions if they exist
    if (config.conditions) {
      return config.conditions.every((condition) => condition());
    }

    return true;
  }

  public getTransitionDuration(from: string, to: string): number {
    const config = this.findTransitionConfig(from, to);
    return config?.duration || 0.5; // Default duration
  }

  public interruptTransition(newToAction: THREE.AnimationAction): void {
    // Find any active transition to interrupt
    const activeTransition = this.activeTransitions.find((t) => t.toAction.isRunning());

    if (activeTransition) {
      // Start new transition from current state
      const currentWeight = activeTransition.toAction.weight;
      this.crossfade(activeTransition.toAction, newToAction, 0.3);

      // Clean up the interrupted transition
      const index = this.activeTransitions.indexOf(activeTransition);
      if (index !== -1) {
        this.activeTransitions.splice(index, 1);
      }
    }
  }

  public isTransitioning(): boolean {
    return this.activeTransitions.length > 0;
  }

  public getActiveTransitions(): readonly ActiveTransition[] {
    return this.activeTransitions;
  }

  // Easing functions
  private easeLinear(t: number): number {
    return t;
  }

  private easeInQuad(t: number): number {
    return t * t;
  }

  private easeOutQuad(t: number): number {
    return t * (2 - t);
  }

  private easeInOutQuad(t: number): number {
    return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
  }

  private easeInCubic(t: number): number {
    return t * t * t;
  }

  private easeOutCubic(t: number): number {
    return 1 + --t * t * t;
  }

  private easeInOutCubic(t: number): number {
    return t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1;
  }

  private easeInQuart(t: number): number {
    return t * t * t * t;
  }

  private easeOutQuart(t: number): number {
    return 1 - --t * t * t * t;
  }

  private easeInOutQuart(t: number): number {
    return t < 0.5 ? 8 * t * t * t * t : 1 - 8 * --t * t * t * t;
  }

  public dispose(): void {
    // Complete all active transitions immediately
    this.activeTransitions.forEach((transition) => {
      this.completeTransition(transition);
    });

    this.activeTransitions = [];
    this.transitionConfigs.clear();
  }
}
