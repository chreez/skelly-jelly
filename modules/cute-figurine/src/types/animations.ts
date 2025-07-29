// Simple animation types for IPC server
export enum AnimationType {
  Idle = 'idle',
  Happy = 'happy', 
  Wave = 'wave',
  Thinking = 'thinking',
  Celebration = 'celebration',
  Gentle = 'gentle',
  Alert = 'alert'
}

export enum EmotionState {
  Neutral = 'neutral',
  Happy = 'happy',
  Focused = 'focused',
  Concerned = 'concerned',
  Supportive = 'supportive',
  Caring = 'caring',
  Excited = 'excited'
}

export interface AnimationOptions {
  emotion: EmotionState;
  duration: number;
  loop?: boolean;
}

export interface MessageOptions {
  duration: number;
  position: 'top' | 'center' | 'bottom';
  style: 'friendly' | 'urgent' | 'farewell';
}