import React from 'react';

export interface ControlPanelProps {
  position: 'top' | 'bottom' | 'left' | 'right';
  onInteraction?: (type: string, data?: any) => void;
  compact?: boolean;
}

// Placeholder component - to be implemented
export const ControlPanel: React.FC<ControlPanelProps> = ({
  position,
  onInteraction,
  compact = false,
}) => {
  return (
    <div
      style={{
        position: 'absolute',
        [position]: position === 'bottom' ? '100%' : position === 'top' ? '-40px' : '50%',
        left: '50%',
        transform: 'translateX(-50%)',
        backgroundColor: 'rgba(0,0,0,0.8)',
        color: 'white',
        borderRadius: '4px',
        padding: compact ? '4px 8px' : '8px 12px',
        fontSize: compact ? '10px' : '12px',
        display: 'flex',
        gap: '8px',
        marginTop: position === 'bottom' ? '8px' : '0',
      }}
    >
      <button
        onClick={() => onInteraction?.('settings')}
        style={{
          background: 'none',
          border: '1px solid #666',
          color: 'white',
          padding: '2px 6px',
          borderRadius: '2px',
          cursor: 'pointer',
          fontSize: 'inherit',
        }}
      >
        ⚙️
      </button>
      <button
        onClick={() => onInteraction?.('close')}
        style={{
          background: 'none',
          border: '1px solid #666',
          color: 'white',
          padding: '2px 6px',
          borderRadius: '2px',
          cursor: 'pointer',
          fontSize: 'inherit',
        }}
      >
        ×
      </button>
    </div>
  );
};

export default ControlPanel;
