import type { Position } from './state.types';

// Base component interfaces
export interface BaseComponentProps {
  className?: string;
  style?: React.CSSProperties;
  'data-testid'?: string;
}

export interface InteractiveComponentProps extends BaseComponentProps {
  onClick?: (event: React.MouseEvent) => void;
  onHover?: (event: React.MouseEvent) => void;
  onFocus?: (event: React.FocusEvent) => void;
  onBlur?: (event: React.FocusEvent) => void;
  disabled?: boolean;
  tabIndex?: number;
}

// Animation Canvas component types
export interface AnimationCanvasProps extends BaseComponentProps {
  width: number;
  height: number;
  visible?: boolean;
  onAnimationChange?: (animationName: string) => void;
  performanceMode?: 'active' | 'paused' | 'hidden';
  quality?: 'low' | 'medium' | 'high';
  enableInteraction?: boolean;
}

// Text Bubble component types
export interface TextBubbleProps extends InteractiveComponentProps {
  message: {
    text: string;
    style?: 'default' | 'intervention' | 'celebration' | 'encouragement';
    duration?: number;
  };
  position: 'top' | 'bottom' | 'left' | 'right';
  onDismiss?: () => void;
  animated?: boolean;
  autoHide?: boolean;
}

// Control Panel component types
export interface ControlPanelProps extends BaseComponentProps {
  position: 'top' | 'bottom' | 'left' | 'right';
  onInteraction?: (type: string, data?: any) => void;
  compact?: boolean;
  visible?: boolean;
  controls?: ControlItem[];
}

export interface ControlItem {
  id: string;
  label: string;
  icon?: string;
  action: () => void;
  disabled?: boolean;
  tooltip?: string;
}

// Drag Handle component types
export interface DragHandleProps extends InteractiveComponentProps {
  onDragStart?: (position: Position) => void;
  onDrag?: (position: Position) => void;
  onDragEnd?: (position: Position) => void;
  constrainToViewport?: boolean;
  snapToGrid?: boolean;
  gridSize?: number;
}

// Settings Panel component types
export interface SettingsPanelProps extends BaseComponentProps {
  visible: boolean;
  onClose: () => void;
  onSettingChange: (setting: string, value: any) => void;
  settings: SettingsConfig;
}

export interface SettingsConfig {
  animation: {
    quality: 'low' | 'medium' | 'high';
    fps: number;
    enableShaders: boolean;
    reducedMotion: boolean;
  };
  appearance: {
    transparency: number;
    scale: number;
    position: Position;
  };
  behavior: {
    interventionFrequency: 'low' | 'medium' | 'high';
    autoHideControls: boolean;
    enableSounds: boolean;
  };
  accessibility: {
    enableHighContrast: boolean;
    fontSize: 'small' | 'medium' | 'large';
    enableScreenReader: boolean;
    keyboardNavigation: boolean;
  };
}

// Debug Panel component types
export interface DebugPanelProps extends BaseComponentProps {
  visible: boolean;
  metrics: {
    fps: number;
    frameTime: number;
    memoryUsage: number;
    cpuUsage: number;
    animationState: string;
    eventCount: number;
  };
  onToggleMetric: (metric: string) => void;
  onExportData: () => void;
}

// Notification component types
export interface NotificationProps extends BaseComponentProps {
  type: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
  duration?: number;
  onDismiss?: () => void;
  actions?: NotificationAction[];
}

export interface NotificationAction {
  label: string;
  action: () => void;
  style?: 'primary' | 'secondary' | 'danger';
}

// Loading component types
export interface LoadingSpinnerProps extends BaseComponentProps {
  size?: 'small' | 'medium' | 'large';
  color?: string;
  message?: string;
}

// Error Boundary component types
export interface ErrorBoundaryProps {
  children: React.ReactNode;
  fallback?: React.ComponentType<ErrorBoundaryFallbackProps>;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

export interface ErrorBoundaryFallbackProps {
  error: Error;
  resetError: () => void;
}

// Theme provider types
export interface ThemeProviderProps {
  children: React.ReactNode;
  theme?: Theme;
}

export interface Theme {
  colors: {
    primary: string;
    secondary: string;
    background: string;
    text: string;
    border: string;
    error: string;
    warning: string;
    success: string;
    info: string;
  };
  spacing: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
  typography: {
    fontFamily: string;
    fontSize: {
      xs: string;
      sm: string;
      md: string;
      lg: string;
      xl: string;
    };
  };
  borderRadius: string;
  shadows: {
    sm: string;
    md: string;
    lg: string;
  };
}

// Common prop patterns
export interface WithTooltip {
  tooltip?: string;
  tooltipPosition?: 'top' | 'bottom' | 'left' | 'right';
}

export interface WithLoading {
  loading?: boolean;
  loadingText?: string;
}

export interface WithError {
  error?: string | Error;
  onRetry?: () => void;
}
