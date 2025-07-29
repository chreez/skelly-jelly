/**
 * MotivationEngine - Personalized Motivational Messaging
 * 
 * Generates contextually appropriate, personalized messages that encourage
 * without being patronizing. Adapts tone and content to user preferences.
 */

import {
  ADHDState,
  BehavioralMetrics,
  InterventionContext,
  MotivationalMessage,
  MessageStyle,
  MessageTone,
  MessageTemplates,
  ActionButton,
  UserProfile,
  SessionMetrics,
  WorkContext,
  SoundEffect,
  GamificationConfig
} from '../types/index.js';
import { Logger } from 'winston';
import { format } from 'date-fns';

export interface MessageContext {
  userState: ADHDState;
  previousState?: ADHDState;
  metrics: BehavioralMetrics;
  sessionMetrics: SessionMetrics;
  workContext: WorkContext;
  timeOfDay: string;
  recentInterventions: string[];
  userResponseHistory: string[];
}

export interface ContextAnalysis {
  userMood: 'positive' | 'neutral' | 'frustrated' | 'tired' | 'motivated';
  workType: 'creative' | 'analytical' | 'routine' | 'collaborative' | 'unknown';
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  urgencyLevel: 'low' | 'medium' | 'high' | 'critical';
  energyLevel: 'low' | 'medium' | 'high';
  focusQuality: 'poor' | 'fair' | 'good' | 'excellent';
}

export class MotivationEngine {
  private messageTemplates: MessageTemplates;
  private contextAnalyzer: ContextAnalyzer;
  private toneCalibrator: ToneCalibrator;
  private personalizationEngine: PersonalizationEngine;
  private logger: Logger;
  private config: GamificationConfig;

  constructor(logger: Logger, config: GamificationConfig) {
    this.logger = logger;
    this.config = config;
    
    this.messageTemplates = this.initializeMessageTemplates();
    this.contextAnalyzer = new ContextAnalyzer(logger);
    this.toneCalibrator = new ToneCalibrator(config);
    this.personalizationEngine = new PersonalizationEngine(logger, config);
  }

  /**
   * Generate a personalized motivational message
   * Focus on being helpful without being condescending
   */
  async generateMessage(context: InterventionContext): Promise<MotivationalMessage> {
    try {
      // Analyze the current context
      const analysis = await this.contextAnalyzer.analyze({
        userState: context.currentState,
        previousState: context.previousState,
        metrics: context.metrics,
        sessionMetrics: context.sessionMetrics,
        workContext: context.workContext,
        timeOfDay: this.getTimeOfDay(),
        recentInterventions: [], // Would come from intervention history
        userResponseHistory: [] // Would come from response tracking
      });

      // Select appropriate tone based on context
      const tone = this.toneCalibrator.selectTone(
        analysis.userMood,
        analysis.workType,
        analysis.timeOfDay,
        context.userProfile
      );

      // Generate base message content
      const baseMessage = await this.generateBaseMessage(
        context.interventionType,
        context.currentState,
        analysis,
        tone
      );

      // Personalize the message
      const personalizedContent = await this.personalizationEngine.personalize(
        baseMessage,
        context.userProfile,
        analysis
      );

      // Create final message object
      const message: MotivationalMessage = {
        text: personalizedContent.text,
        duration: this.calculateDisplayDuration(personalizedContent.text, context.userProfile),
        style: this.createMessageStyle(tone, analysis),
        tone,
        audio: this.shouldIncludeAudio(context.userProfile) ? this.createAudioCue(tone) : undefined,
        dismissible: true,
        actionButton: this.createActionButton(context.interventionType, analysis)
      };

      this.logger.info('Motivational message generated', {
        type: context.interventionType,
        mood: analysis.userMood,
        tone: `${tone.formality.toFixed(1)}/${tone.enthusiasm.toFixed(1)}`,
        length: message.text.length,
        duration: message.duration
      });

      return message;

    } catch (error) {
      this.logger.error('Error generating motivational message', { error });
      return this.createFallbackMessage(context.interventionType);
    }
  }

  /**
   * Get message templates for a specific category
   */
  getTemplatesForCategory(category: string): string[] {
    switch (category) {
      case 'encouragement':
        return [
          ...this.messageTemplates.encouragement.flow_entry,
          ...this.messageTemplates.encouragement.sustained_focus,
          ...this.messageTemplates.encouragement.recovery
        ];
      case 'suggestion':
        return [
          ...this.messageTemplates.suggestion.break_reminder,
          ...this.messageTemplates.suggestion.productivity_tip,
          ...this.messageTemplates.suggestion.environment_adjustment
        ];
      case 'celebration':
        return [
          ...this.messageTemplates.celebration.milestone_reached,
          ...this.messageTemplates.celebration.achievement_unlocked,
          ...this.messageTemplates.celebration.daily_goal
        ];
      case 'gentle_nudge':
        return [
          ...this.messageTemplates.gentle_nudge.distraction_detected,
          ...this.messageTemplates.gentle_nudge.refocus_tip
        ];
      default:
        return ['Keep up the great work!'];
    }
  }

  /**
   * Update message templates based on user feedback
   */
  updateTemplatesFromFeedback(
    category: string, 
    messageText: string, 
    userResponse: 'positive' | 'negative' | 'neutral'
  ): void {
    // In a full implementation, this would use machine learning to improve templates
    this.logger.debug('Message feedback received', {
      category,
      response: userResponse,
      messageLength: messageText.length
    });
  }

  // === Private Helper Methods ===

  private async generateBaseMessage(
    interventionType: string,
    state: ADHDState,
    analysis: ContextAnalysis,
    tone: MessageTone
  ): Promise<{ text: string; variables: Record<string, string> }> {
    const templates = this.getTemplatesForType(interventionType, state, analysis);
    
    if (templates.length === 0) {
      return {
        text: 'You\'re doing great! Keep it up!',
        variables: {}
      };
    }

    // Select template based on tone and context
    const selectedTemplate = this.selectBestTemplate(templates, tone, analysis);
    
    if (!selectedTemplate) {
      return {
        text: 'Keep up the great work!',
        variables: {}
      };
    }
    
    // Extract variables from context
    const variables = this.extractVariables(state, analysis);
    
    return {
      text: selectedTemplate,
      variables
    };
  }

  private getTemplatesForType(
    interventionType: string,
    state: ADHDState,
    analysis: ContextAnalysis
  ): string[] {
    switch (interventionType) {
      case 'encouragement':
        if (state.type === 'Flow') {
          return this.messageTemplates.encouragement.flow_entry;
        } else if (analysis.focusQuality === 'good' || analysis.focusQuality === 'excellent') {
          return this.messageTemplates.encouragement.sustained_focus;
        } else {
          return this.messageTemplates.encouragement.recovery;
        }

      case 'suggestion':
        if (analysis.energyLevel === 'low') {
          return this.messageTemplates.suggestion.break_reminder;
        } else if (analysis.focusQuality === 'poor') {
          return this.messageTemplates.suggestion.productivity_tip;
        } else {
          return this.messageTemplates.suggestion.environment_adjustment;
        }

      case 'celebration':
        return this.messageTemplates.celebration.milestone_reached;

      case 'gentle_nudge':
        if (state.type === 'Distracted' && state.duration && state.duration > 10 * 60 * 1000) {
          return this.messageTemplates.gentle_nudge.refocus_tip;
        } else {
          return this.messageTemplates.gentle_nudge.distraction_detected;
        }

      default:
        return this.messageTemplates.encouragement.sustained_focus;
    }
  }

  private selectBestTemplate(
    templates: string[],
    tone: MessageTone,
    analysis: ContextAnalysis
  ): string {
    // Score templates based on tone appropriateness
    let bestTemplate = templates[0];
    let bestScore = 0;

    for (const template of templates) {
      let score = 0;

      // Score based on formality match
      const templateFormality = this.assessTemplateFormality(template);
      score += 1 - Math.abs(tone.formality - templateFormality);

      // Score based on enthusiasm match
      const templateEnthusiasm = this.assessTemplateEnthusiasm(template);
      score += 1 - Math.abs(tone.enthusiasm - templateEnthusiasm);

      // Score based on context appropriateness
      if (analysis.userMood === 'frustrated' && this.isGentleTemplate(template)) {
        score += 0.5;
      }
      
      if (analysis.userMood === 'motivated' && this.isEnergeticTemplate(template)) {
        score += 0.5;
      }

      if (score > bestScore) {
        bestScore = score;
        bestTemplate = template;
      }
    }

    return bestTemplate || 'Keep up the great work!';
  }

  private extractVariables(state: ADHDState, analysis: ContextAnalysis): Record<string, string> {
    const minutes = state.duration ? Math.floor(state.duration / (60 * 1000)) : 0;
    
    return {
      duration: minutes.toString(),
      duration_text: this.formatDuration(minutes),
      time_of_day: analysis.timeOfDay,
      focus_quality: analysis.focusQuality,
      energy_level: analysis.energyLevel,
      state_type: state.type.toLowerCase()
    };
  }

  private formatDuration(minutes: number): string {
    if (minutes < 1) return 'a moment';
    if (minutes === 1) return '1 minute';
    if (minutes < 60) return `${minutes} minutes`;
    
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    
    if (hours === 1 && remainingMinutes === 0) return '1 hour';
    if (remainingMinutes === 0) return `${hours} hours`;
    if (hours === 1) return `1 hour and ${remainingMinutes} minutes`;
    
    return `${hours} hours and ${remainingMinutes} minutes`;
  }

  private calculateDisplayDuration(text: string, userProfile: UserProfile): number {
    // Base duration on reading speed and user preferences
    const wordsPerMinute = 200; // Average reading speed
    const words = text.split(' ').length;
    const readingTime = (words / wordsPerMinute) * 60 * 1000;
    
    // Add buffer time for comprehension
    const bufferTime = Math.min(readingTime * 0.5, 3000);
    
    // Minimum and maximum durations
    const minDuration = 2000;
    const maxDuration = 8000;
    
    let duration = readingTime + bufferTime;
    
    // Adjust based on user preferences
    if (userProfile.preferences.messageFrequency === 'low') {
      duration *= 1.2; // Show longer for users who want fewer messages
    }
    
    return Math.max(minDuration, Math.min(duration, maxDuration));
  }

  private createMessageStyle(tone: MessageTone, analysis: ContextAnalysis): MessageStyle {
    return {
      fontSize: analysis.userMood === 'tired' ? 'large' : 'medium',
      color: this.getMessageColor(tone, analysis),
      backgroundColor: this.getBackgroundColor(tone, analysis),
      position: 'companion',
      animation: tone.enthusiasm > 0.7 ? 'bounce' : 'fade',
      urgency: analysis.urgencyLevel === 'high' ? 'high' : 'low'
    };
  }

  private getMessageColor(tone: MessageTone, analysis: ContextAnalysis): string {
    if (analysis.userMood === 'frustrated') return '#E67E22'; // Warm orange
    if (analysis.userMood === 'motivated') return '#27AE60'; // Motivating green
    if (tone.enthusiasm > 0.8) return '#3498DB'; // Bright blue
    if (tone.formality > 0.7) return '#2C3E50'; // Professional dark blue
    
    return '#34495E'; // Default neutral color
  }

  private getBackgroundColor(tone: MessageTone, analysis: ContextAnalysis): string {
    const alpha = 0.9;
    
    if (analysis.urgencyLevel === 'high') return `rgba(231, 76, 60, ${alpha})`; // Red
    if (tone.enthusiasm > 0.8) return `rgba(52, 152, 219, ${alpha})`; // Blue
    if (analysis.userMood === 'positive') return `rgba(46, 204, 113, ${alpha})`; // Green
    
    return `rgba(149, 165, 166, ${alpha})`; // Default gray
  }

  private shouldIncludeAudio(userProfile: UserProfile): boolean {
    return userProfile.preferences.soundEnabled && 
           userProfile.preferences.visualNotifications;
  }

  private createAudioCue(tone: MessageTone): SoundEffect {
    let soundType: SoundEffect['type'] = 'gentle_notification';
    
    if (tone.enthusiasm > 0.8) {
      soundType = 'chime';
    } else if (tone.supportiveness > 0.8) {
      soundType = 'gentle_notification';
    }

    return {
      type: soundType,
      volume: Math.max(0.1, Math.min(tone.enthusiasm * 0.4, 0.4)),
      respectsUserSettings: true
    };
  }

  private createActionButton(
    interventionType: string, 
    analysis: ContextAnalysis
  ): ActionButton | undefined {
    if (interventionType === 'suggestion' && analysis.energyLevel === 'low') {
      return {
        text: 'Take a break',
        action: 'take_break',
        style: 'secondary'
      };
    }
    
    if (interventionType === 'encouragement' && analysis.focusQuality === 'excellent') {
      return {
        text: 'View progress',
        action: 'view_progress',
        style: 'minimal'
      };
    }

    return undefined;
  }

  private createFallbackMessage(interventionType: string): MotivationalMessage {
    const fallbackMessages = {
      encouragement: 'You\'re doing great! Keep up the good work!',
      suggestion: 'Consider taking a moment to refocus.',
      celebration: 'Awesome achievement! Well done!',
      gentle_nudge: 'Hey there! Ready to get back on track?'
    };

    const text = fallbackMessages[interventionType as keyof typeof fallbackMessages] || 
                'Keep up the excellent work!';

    return {
      text,
      duration: 3000,
      style: {
        fontSize: 'medium',
        color: '#34495E',
        backgroundColor: 'rgba(149, 165, 166, 0.9)',
        position: 'companion',
        animation: 'fade',
        urgency: 'low'
      },
      tone: {
        formality: 0.5,
        enthusiasm: 0.6,
        supportiveness: 0.8,
        directness: 0.5,
        humor: 0.3
      },
      dismissible: true
    };
  }

  private getTimeOfDay(): string {
    const hour = new Date().getHours();
    
    if (hour >= 5 && hour < 12) return 'morning';
    if (hour >= 12 && hour < 17) return 'afternoon';
    if (hour >= 17 && hour < 21) return 'evening';
    return 'night';
  }

  private assessTemplateFormality(template: string): number {
    const formalIndicators = ['please', 'consider', 'perhaps', 'would you', 'might'];
    const informalIndicators = ['hey', 'awesome', 'great job', 'crushing it', '!'];
    
    let formalScore = 0;
    let informalScore = 0;
    
    const lowerTemplate = template.toLowerCase();
    
    for (const indicator of formalIndicators) {
      if (lowerTemplate.includes(indicator)) formalScore++;
    }
    
    for (const indicator of informalIndicators) {
      if (lowerTemplate.includes(indicator)) informalScore++;
    }
    
    const totalIndicators = formalScore + informalScore;
    if (totalIndicators === 0) return 0.5;
    
    return formalScore / totalIndicators;
  }

  private assessTemplateEnthusiasm(template: string): number {
    const enthusiasticIndicators = ['!', 'amazing', 'awesome', 'excellent', 'fantastic', 'great'];
    const calmIndicators = ['nice', 'good', 'well done', 'keep', 'continue'];
    
    let enthusiasmScore = 0;
    let calmScore = 0;
    
    const lowerTemplate = template.toLowerCase();
    
    for (const indicator of enthusiasticIndicators) {
      if (lowerTemplate.includes(indicator)) enthusiasmScore++;
    }
    
    for (const indicator of calmIndicators) {
      if (lowerTemplate.includes(indicator)) calmScore++;
    }
    
    const totalIndicators = enthusiasmScore + calmScore;
    if (totalIndicators === 0) return 0.5;
    
    return enthusiasmScore / totalIndicators;
  }

  private isGentleTemplate(template: string): boolean {
    const gentleWords = ['gently', 'perhaps', 'maybe', 'consider', 'might want to'];
    const lowerTemplate = template.toLowerCase();
    
    return gentleWords.some(word => lowerTemplate.includes(word));
  }

  private isEnergeticTemplate(template: string): boolean {
    const energeticWords = ['awesome', 'amazing', 'crushing', 'fantastic', 'excellent'];
    const lowerTemplate = template.toLowerCase();
    
    return energeticWords.some(word => lowerTemplate.includes(word));
  }

  private initializeMessageTemplates(): MessageTemplates {
    return {
      encouragement: {
        flow_entry: [
          "Nice! You're settling into focus mode 🎯",
          "Your concentration is building beautifully",
          "Feel that focus energy? You've got this!",
          "Excellent - you're finding your rhythm",
          "Great start! Your attention is sharpening"
        ],
        sustained_focus: [
          "You've been focused for {duration_text} - impressive!",
          "Solid concentration happening here 💪",
          "Your productivity streak is looking strong",
          "Excellent sustained focus for {duration_text}!",
          "You're in a really good groove right now"
        ],
        recovery: [
          "Good job getting back on track!",
          "Nice recovery! That's the spirit",
          "Way to bounce back from that distraction",
          "Resilience in action - well done! 🦴",
          "Your ability to refocus is improving"
        ],
        personal_record: [
          "New personal best! {duration_text} of focus! 🏆",
          "You just crushed your previous record!",
          "Amazing - that's your longest focus session yet!",
          "Record-breaking concentration! Outstanding!"
        ]
      },
      gentle_nudge: {
        distraction_detected: [
          "Hey, everything going okay over there?",
          "Noticed you might be stuck - need a moment?",
          "Your skeleton pal is here if you need support",
          "No judgment - we all get sidetracked sometimes",
          "Ready for a gentle reset when you are"
        ],
        long_distraction: [
          "Been a while - want to try a different approach?",
          "Sometimes a fresh perspective helps",
          "No pressure, but maybe time for a new strategy?",
          "Your skeleton friend suggests a quick change of pace",
          "When you're ready, we can tackle this together"
        ],
        break_suggestion: [
          "You've been at this for a while - break time?",
          "Your brain might appreciate a quick recharge",
          "Consider a 5-minute reset - you've earned it",
          "Even focus champions need breaks! 🦴"
        ],
        refocus_tip: [
          "Try the 2-minute rule: just start for 2 minutes",
          "Sometimes changing your environment helps",
          "Deep breath, clear the clutter, fresh start",
          "Break the task into smaller, manageable pieces"
        ]
      },
      celebration: {
        milestone_reached: [
          "🎉 Milestone unlocked! You're on fire!",
          "Achievement earned! Your skeleton pal is dancing! 🦴💃",
          "Incredible progress! That's worth celebrating!",
          "New level of awesome reached! Well done!",
          "Outstanding achievement! You should be proud!"
        ],
        achievement_unlocked: [
          "🏆 New achievement: {achievement_name}!",
          "Badge earned! Your skills are evolving!",
          "Achievement unlocked! You're leveling up!",
          "Congratulations on your new achievement!",
          "Your skeleton friend is impressed! 🦴✨"
        ],
        daily_goal: [
          "Daily goal crushed! Time to celebrate 🎊",
          "You did it! Another productive day complete",
          "Goal achieved! Your consistency is paying off",
          "Daily mission accomplished! Excellent work!"
        ],
        streak_milestone: [
          "🔥 {streak_count} day streak! You're unstoppable!",
          "Streak milestone reached! Your momentum is incredible",
          "Consistency champion! {streak_count} days and counting!",
          "Your dedication is showing - {streak_count} day streak!"
        ]
      },
      suggestion: {
        break_reminder: [
          "Your brain has been working hard - how about a 5-minute break?",
          "Time for a quick recharge? Your skeleton pal recommends it",
          "Consider stepping away for a moment - you'll come back stronger",
          "A brief pause might be just what you need right now"
        ],
        productivity_tip: [
          "Try the Pomodoro technique: 25 minutes focus, 5 minute break",
          "Consider removing distractions from your workspace",
          "Sometimes background music or silence can help focus",
          "Breaking tasks into smaller chunks often helps"
        ],
        environment_adjustment: [
          "Your environment affects focus - lighting okay?",
          "Consider adjusting your workspace for better concentration",
          "Sometimes a change of scenery boosts productivity",
          "Check if your setup is supporting your focus goals"
        ],
        time_management: [
          "Planning your most important task for your peak energy time?",
          "Consider batching similar tasks together",
          "Time-blocking might help structure your day",
          "Prioritizing the most important tasks first often works well"
        ]
      }
    };
  }
}

/**
 * Context Analyzer - Understands user's current situation
 */
class ContextAnalyzer {
  private logger: Logger;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  async analyze(context: MessageContext): Promise<ContextAnalysis> {
    return {
      userMood: this.assessUserMood(context),
      workType: this.categorizeWorkType(context.workContext),
      timeOfDay: context.timeOfDay as ContextAnalysis['timeOfDay'],
      urgencyLevel: this.assessUrgencyLevel(context),
      energyLevel: this.assessEnergyLevel(context),
      focusQuality: this.assessFocusQuality(context)
    };
  }

  private assessUserMood(context: MessageContext): ContextAnalysis['userMood'] {
    // Analyze based on state transitions and metrics
    if (context.userState.type === 'Flow' && context.userState.confidence && context.userState.confidence > 0.8) {
      return 'motivated';
    }
    
    if (context.userState.type === 'Distracted' && context.userState.duration && context.userState.duration > 15 * 60 * 1000) {
      return 'frustrated';
    }
    
    if (context.timeOfDay === 'night' || context.metrics.productive_time_ratio < 0.3) {
      return 'tired';
    }
    
    if (context.sessionMetrics.recoveryCount > 3) {
      return 'positive'; // Good at recovering
    }
    
    return 'neutral';
  }

  private categorizeWorkType(workContext: WorkContext): ContextAnalysis['workType'] {
    return workContext.task_category === 'creative' ? 'creative' :
           workContext.task_category === 'work' ? 'analytical' :
           workContext.task_category === 'research' ? 'analytical' :
           workContext.task_category === 'communication' ? 'collaborative' :
           'routine';
  }

  private assessUrgencyLevel(context: MessageContext): ContextAnalysis['urgencyLevel'] {
    if (context.workContext.urgency === 'critical') return 'critical';
    if (context.workContext.urgency === 'high') return 'high';
    if (context.workContext.urgency === 'medium') return 'medium';
    return 'low';
  }

  private assessEnergyLevel(context: MessageContext): ContextAnalysis['energyLevel'] {
    const productivity = context.metrics.productive_time_ratio;
    const sessionLength = context.sessionMetrics.totalFocusTime / (60 * 60 * 1000); // hours
    
    if (productivity > 0.7 && sessionLength < 2) return 'high';
    if (productivity < 0.3 || sessionLength > 4) return 'low';
    return 'medium';
  }

  private assessFocusQuality(context: MessageContext): ContextAnalysis['focusQuality'] {
    const productivity = context.metrics.productive_time_ratio;
    const confidence = context.userState.confidence || 0;
    
    if (productivity > 0.8 && confidence > 0.8) return 'excellent';
    if (productivity > 0.6 && confidence > 0.6) return 'good';
    if (productivity > 0.4 && confidence > 0.4) return 'fair';
    return 'poor';
  }
}

/**
 * Tone Calibrator - Selects appropriate communication tone
 */
class ToneCalibrator {
  private config: GamificationConfig;

  constructor(config: GamificationConfig) {
    this.config = config;
  }

  selectTone(
    mood: ContextAnalysis['userMood'],
    workType: ContextAnalysis['workType'],
    timeOfDay: ContextAnalysis['timeOfDay'],
    userProfile: UserProfile
  ): MessageTone {
    let formality = this.config.companion.personalityTraits.formality;
    let enthusiasm = this.config.companion.personalityTraits.cheerfulness;
    let supportiveness = this.config.companion.personalityTraits.supportiveness;
    let directness = 0.5;
    let humor = this.config.companion.personalityTraits.humor;

    // Adjust based on user mood
    if (mood === 'frustrated') {
      formality += 0.1;
      enthusiasm -= 0.2;
      supportiveness += 0.3;
      humor -= 0.2;
    } else if (mood === 'motivated') {
      enthusiasm += 0.2;
      directness += 0.1;
    } else if (mood === 'tired') {
      formality -= 0.1;
      enthusiasm -= 0.1;
      supportiveness += 0.2;
    }

    // Adjust based on work type
    if (workType === 'analytical') {
      formality += 0.1;
      directness += 0.1;
    } else if (workType === 'creative') {
      humor += 0.1;
      enthusiasm += 0.1;
    }

    // Adjust based on time of day
    if (timeOfDay === 'morning') {
      enthusiasm += 0.1;
    } else if (timeOfDay === 'evening' || timeOfDay === 'night') {
      formality -= 0.1;
      supportiveness += 0.1;
    }

    // Apply user preferences
    if (userProfile.preferences.messageStyle === 'minimal') {
      formality += 0.2;
      enthusiasm -= 0.2;
      directness += 0.2;
    } else if (userProfile.preferences.messageStyle === 'encouraging') {
      enthusiasm += 0.2;
      supportiveness += 0.2;
    }

    // Clamp values to 0-1 range
    return {
      formality: Math.max(0, Math.min(1, formality)),
      enthusiasm: Math.max(0, Math.min(1, enthusiasm)),
      supportiveness: Math.max(0, Math.min(1, supportiveness)),
      directness: Math.max(0, Math.min(1, directness)),
      humor: Math.max(0, Math.min(1, humor))
    };
  }
}

/**
 * Personalization Engine - Customizes messages to user preferences
 */
class PersonalizationEngine {
  private logger: Logger;
  private config: GamificationConfig;

  constructor(logger: Logger, config: GamificationConfig) {
    this.logger = logger;
    this.config = config;
  }

  async personalize(
    baseMessage: { text: string; variables: Record<string, string> },
    userProfile: UserProfile,
    analysis: ContextAnalysis
  ): Promise<{ text: string; audio?: SoundEffect }> {
    let personalizedText = baseMessage.text;

    // Replace variables
    for (const [key, value] of Object.entries(baseMessage.variables)) {
      personalizedText = personalizedText.replace(`{${key}}`, value);
    }

    // Apply personality-based modifications
    personalizedText = this.applyPersonalityModifications(
      personalizedText,
      userProfile.personalityProfile
    );

    // Apply accessibility modifications
    personalizedText = this.applyAccessibilityModifications(
      personalizedText,
      userProfile.accessibilityNeeds
    );

    return {
      text: personalizedText
    };
  }

  private applyPersonalityModifications(
    text: string,
    personality: UserProfile['personalityProfile']
  ): string {
    let modified = text;

    // Adjust for communication style
    if (personality.communicationStyle === 'direct') {
      // Remove hedging language
      modified = modified.replace(/maybe\s+/gi, '');
      modified = modified.replace(/perhaps\s+/gi, '');
      modified = modified.replace(/might want to\s+/gi, '');
    } else if (personality.communicationStyle === 'humorous') {
      // Add more playful elements where appropriate
      modified = modified.replace(/skeleton pal/gi, 'your bony buddy');
      modified = modified.replace(/well done/gi, 'bone-afide success');
    }

    // Adjust for motivation style
    if (personality.motivationStyle === 'achievement') {
      // Emphasize progress and accomplishments
      modified = modified.replace(/good job/gi, 'achievement unlocked');
      modified = modified.replace(/nice/gi, 'excellent progress');
    }

    return modified;
  }

  private applyAccessibilityModifications(
    text: string,
    accessibility: UserProfile['accessibilityNeeds']
  ): string {
    let modified = text;

    // Screen reader compatibility
    if (accessibility.screenReaderCompatible) {
      // Remove emoji or replace with text
      modified = modified.replace(/🎯/g, '(target)');
      modified = modified.replace(/💪/g, '(strength)');
      modified = modified.replace(/🦴/g, '(skeleton)');
      modified = modified.replace(/🎉/g, '(celebration)');
      modified = modified.replace(/✨/g, '(sparkles)');
    }

    // Simplified language if needed
    if (accessibility.largeText) {
      // Shorter sentences for easier reading
      modified = modified.replace(/([.!?])\s+/g, '$1\n');
    }

    return modified;
  }
}