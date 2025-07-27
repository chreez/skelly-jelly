export enum AnimationType {
  SKELETAL = 'skeletal',
  MORPH = 'morph',
  SHADER = 'shader',
  PARTICLE = 'particle',
  COMPOSITE = 'composite',
}

export enum AnimationQuality {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
}

export interface AnimationConfig {
  fps: number;
  quality: AnimationQuality;
  enableShaders: boolean;
  reducedMotion: boolean;
}

export interface AnimationState {
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

export interface Keyframe {
  time: number; // 0-1 normalized
  value: any;
  easing: EasingFunction;
  interpolation: 'linear' | 'cubic' | 'step';
}

export interface StateTransition {
  from: string;
  to: string;
  duration: number;
  condition?: () => boolean;
  blendCurve?: BlendCurve;
  priority?: number;
  interruptible?: boolean;
}

export interface TransitionConfig {
  to: string;
  duration: number;
  easing: EasingFunction;
  conditions?: TransitionCondition[];
  blendMode?: 'replace' | 'additive' | 'multiply';
  weight?: number;
}

export interface TransitionCondition {
  type: 'parameter' | 'state' | 'time' | 'trigger';
  parameter?: string;
  operator: '>' | '<' | '>=' | '<=' | '==' | '!=';
  value: number | string | boolean;
}

export interface ShaderUniforms {
  [key: string]: {
    value: number | number[] | THREE.Color | THREE.Texture;
    target?: number;
  };
}

export interface BlendCurve {
  type: 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out' | 'custom';
  controlPoints?: number[];
}

export type EasingFunction =
  | 'linear'
  | 'easeInQuad'
  | 'easeOutQuad'
  | 'easeInOutQuad'
  | 'easeInCubic'
  | 'easeOutCubic'
  | 'easeInOutCubic'
  | 'easeInQuart'
  | 'easeOutQuart'
  | 'easeInOutQuart'
  | 'easeInSine'
  | 'easeOutSine'
  | 'easeInOutSine'
  | 'easeInExpo'
  | 'easeOutExpo'
  | 'easeInOutExpo'
  | 'easeInCirc'
  | 'easeOutCirc'
  | 'easeInOutCirc'
  | 'easeInBack'
  | 'easeOutBack'
  | 'easeInOutBack'
  | 'easeInElastic'
  | 'easeOutElastic'
  | 'easeInOutElastic'
  | 'easeOutBounce';

export interface AnimationInstance {
  state: AnimationState;
  startTime: number;
  currentTime: number;
  blendWeight: number;
  isPlaying: boolean;
}

export interface ParticleConfig {
  count: number;
  velocity: { min: number; max: number };
  lifetime: { min: number; max: number };
  colors: string[];
  gravity: number;
  spread: number;
}

export interface PerformanceMetrics {
  fps: number;
  cpu: number;
  memory: number;
  drawCalls: number;
}
