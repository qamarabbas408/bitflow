// src/settings-main.tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import Settings from './Settings';
import './App.css'; // Import global styles for variables

ReactDOM.createRoot(document.getElementById('root-settings') as HTMLElement).render(
  <React.StrictMode>
    <Settings />
  </React.StrictMode>,
);