export enum MoodState {
  HAPPY = 'happy',
  FOCUSED = 'focused',
  TIRED = 'tired',
  EXCITED = 'excited',
  MELTING = 'melting',
  THINKING = 'thinking',
  CELEBRATING = 'celebrating',
}

export enum ActivityState {
  IDLE = 'idle',
  WORKING = 'working',
  INTERACTING = 'interacting',
  RESTING = 'resting',
  INTERVENING = 'intervening',
}

export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface CompanionState {
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
  particleCount: number;

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

export interface CompanionPreferences {
  position: Position;
  transparency: number;
  animationQuality: 'low' | 'medium' | 'high';
  reducedMotion: boolean;
  scale: number;
}

export interface Message {
  id?: string;
  text: string;
  duration?: number;
  style?: 'default' | 'intervention' | 'celebration' | 'encouragement';
  priority?: number;
}
