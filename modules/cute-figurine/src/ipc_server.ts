#!/usr/bin/env node
/**
 * IPC Server for Cute Figurine Module
 * 
 * Handles communication with the Rust main process and manages the skeleton
 * companion window with WebGL animations
 */

import { SkeletonCompanion } from './core/skeleton_companion.js';
import { AnimationType, EmotionState } from './types/animations.js';

interface TypeScriptMessage {
  module: string;
  action: string;
  payload: any;
  timestamp: number;
}

interface StateDetection {
  state: 'Flow' | 'Distracted' | 'Hyperfocus';
  confidence: number;
  timestamp: number;
}

interface Intervention {
  message: string;
  animation: string;
  urgency: 'low' | 'medium' | 'high';
  timestamp: number;
}

class FigurineIPCServer {
  private companion: SkeletonCompanion;
  private isShuttingDown = false;

  constructor() {
    this.companion = new SkeletonCompanion({
      width: 300,
      height: 300,
      alwaysOnTop: true,
      transparent: true,
      frame: false
    });
    
    this.setupProcessHandlers();
    this.startListening();
  }

  private setupProcessHandlers(): void {
    process.on('SIGINT', () => this.shutdown());
    process.on('SIGTERM', () => this.shutdown());
    process.on('uncaughtException', (error) => {
      console.error('Uncaught exception in Figurine IPC:', error);
      this.shutdown();
    });
  }

  private async startListening(): Promise<void> {
    console.log('💀 Cute Figurine IPC Server starting...');
    
    // Initialize the skeleton companion window
    await this.companion.initialize();
    console.log('✅ Skeleton companion window created');
    
    // Start with idle animation
    await this.companion.playAnimation(AnimationType.Idle, {
      emotion: EmotionState.Neutral,
      duration: 0 // Continuous
    });
    
    // Listen for messages from Rust via stdin
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (data: string) => {
      try {
        const lines = data.trim().split('\n');
        for (const line of lines) {
          if (line.trim()) {
            const message: TypeScriptMessage = JSON.parse(line);
            this.handleMessage(message);
          }
        }
      } catch (error) {
        console.error('Failed to parse IPC message:', error);
      }
    });

    process.stdin.on('end', () => {
      console.log('Stdin closed, shutting down...');
      this.shutdown();
    });

    console.log('✅ Cute Figurine IPC Server ready');
  }

  private async handleMessage(message: TypeScriptMessage): Promise<void> {
    if (this.isShuttingDown) return;

    console.log(`📨 Received: ${message.action}`, message.payload);

    try {
      switch (message.action) {
        case 'play_animation':
          await this.handlePlayAnimation(message.payload);
          break;
          
        case 'show_intervention':
          await this.handleShowIntervention(message.payload);
          break;
          
        case 'update_emotion':
          await this.handleUpdateEmotion(message.payload);
          break;
          
        case 'set_position':
          await this.handleSetPosition(message.payload);
          break;
          
        case 'set_visibility':
          await this.handleSetVisibility(message.payload.visible);
          break;
          
        case 'shutdown':
          this.shutdown();
          break;
          
        default:
          console.warn(`Unknown action: ${message.action}`);
      }
    } catch (error) {
      console.error(`Error handling ${message.action}:`, error);
    }
  }

  private async handlePlayAnimation(payload: any): Promise<void> {
    const { animation, state } = payload;
    
    let animationType: AnimationType;
    let emotion: EmotionState;
    
    // Map ADHD states to animations and emotions
    switch (state?.state) {
      case 'Flow':
        animationType = AnimationType.Happy;
        emotion = EmotionState.Focused;
        break;
      case 'Distracted':
        animationType = animation === 'gentle_wave' ? AnimationType.Wave : AnimationType.Thinking;
        emotion = EmotionState.Concerned;
        break;
      case 'Hyperfocus':
        animationType = AnimationType.Celebration;
        emotion = EmotionState.Excited;
        break;
      default:
        animationType = this.mapAnimationName(animation);
        emotion = EmotionState.Neutral;
    }
    
    await this.companion.playAnimation(animationType, {
      emotion,
      duration: 3000, // 3 seconds
      loop: false
    });
    
    // Send confirmation back to Rust
    this.sendMessage({
      module: 'cute-figurine',
      action: 'animation_started',
      payload: {
        animation: animationType,
        emotion,
        state: state
      },
      timestamp: Date.now()
    });
  }

  private async handleShowIntervention(intervention: Intervention): Promise<void> {
    console.log(`💬 Showing intervention: ${intervention.message}`);
    
    // Choose animation based on urgency
    let animationType: AnimationType;
    let emotion: EmotionState;
    
    switch (intervention.urgency) {
      case 'high':
        animationType = AnimationType.Alert;
        emotion = EmotionState.Concerned;
        break;
      case 'medium':
        animationType = AnimationType.Wave;
        emotion = EmotionState.Supportive;
        break;
      default:
        animationType = AnimationType.Gentle;
        emotion = EmotionState.Caring;
    }
    
    // Play animation
    await this.companion.playAnimation(animationType, {
      emotion,
      duration: 2000
    });
    
    // Show message bubble
    await this.companion.showMessage(intervention.message, {
      duration: 5000,
      position: 'top',
      style: 'friendly'
    });
    
    // Send confirmation
    this.sendMessage({
      module: 'cute-figurine',
      action: 'intervention_displayed',
      payload: {
        message: intervention.message,
        animation: animationType
      },
      timestamp: Date.now()
    });
  }

  private async handleUpdateEmotion(payload: any): Promise<void> {
    const emotion = this.mapEmotionState(payload.emotion);
    await this.companion.setEmotion(emotion);
  }

  private async handleSetPosition(payload: any): Promise<void> {
    const { x, y } = payload;
    await this.companion.setPosition(x, y);
  }

  private async handleSetVisibility(visible: boolean): Promise<void> {
    if (visible) {
      await this.companion.show();
    } else {
      await this.companion.hide();
    }
  }

  private mapAnimationName(name: string): AnimationType {
    const animationMap: Record<string, AnimationType> = {
      'idle': AnimationType.Idle,
      'happy': AnimationType.Happy,
      'wave': AnimationType.Wave,
      'thinking': AnimationType.Thinking,
      'celebration': AnimationType.Celebration,
      'gentle': AnimationType.Gentle,
      'alert': AnimationType.Alert,
      'happy_focused': AnimationType.Happy,
      'gentle_wave': AnimationType.Wave
    };
    
    return animationMap[name] || AnimationType.Idle;
  }

  private mapEmotionState(emotion: string): EmotionState {
    const emotionMap: Record<string, EmotionState> = {
      'neutral': EmotionState.Neutral,
      'happy': EmotionState.Happy,
      'focused': EmotionState.Focused,
      'concerned': EmotionState.Concerned,
      'supportive': EmotionState.Supportive,
      'caring': EmotionState.Caring,
      'excited': EmotionState.Excited
    };
    
    return emotionMap[emotion] || EmotionState.Neutral;
  }

  private sendMessage(message: TypeScriptMessage): void {
    if (this.isShuttingDown) return;
    
    try {
      const json = JSON.stringify(message);
      process.stdout.write(json + '\n');
      console.log(`📤 Sent: ${message.action}`);
    } catch (error) {
      console.error('Failed to send IPC message:', error);
    }
  }

  private async shutdown(): Promise<void> {
    if (this.isShuttingDown) return;
    this.isShuttingDown = true;
    
    console.log('🛑 Shutting down Cute Figurine IPC Server...');
    
    // Play goodbye animation
    try {
      await this.companion.playAnimation(AnimationType.Wave, {
        emotion: EmotionState.Happy,
        duration: 2000
      });
      
      await this.companion.showMessage('See you later! 💀✨', {
        duration: 1500,
        position: 'center',
        style: 'farewell'
      });
    } catch (error) {
      console.warn('Error during shutdown animation:', error);
    }
    
    // Send final message
    this.sendMessage({
      module: 'cute-figurine',
      action: 'shutdown_complete',
      payload: {},
      timestamp: Date.now()
    });
    
    // Close companion window and exit
    setTimeout(async () => {
      try {
        await this.companion.destroy();
      } catch (error) {
        console.warn('Error destroying companion:', error);
      }
      
      console.log('✅ Cute Figurine IPC Server shutdown complete');
      process.exit(0);
    }, 2500);
  }
}

// Start the server
new FigurineIPCServer();