import React from 'react';
import ReactDOM from 'react-dom/client';
import { SkellyCompanion } from './components/SkellyCompanion/SkellyCompanion';
import './index.css';

// Demo application showcasing the Skelly Companion
function App() {
  return (
    <div style={{ width: '100vw', height: '100vh', position: 'relative' }}>
      <SkellyCompanion
        id="demo-skelly"
        initialState={{
          mood: 'happy',
          energy: 75,
          meltLevel: 0.3
        }}
        initialPosition={{ x: 200, y: 200 }}
        enableDragAndDrop={true}
        enableKeyboardNavigation={true}
        onStateChange={(state) => {
          console.log('Skelly state changed:', state);
        }}
        onUserInteraction={(interaction) => {
          console.log('User interaction:', interaction);
        }}
      />
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);