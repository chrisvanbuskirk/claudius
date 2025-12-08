import React from 'react';
import ReactDOM from 'react-dom/client';
import { PopoverApp } from './PopoverApp';
import { ResearchProvider } from './contexts/ResearchContext';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ResearchProvider>
      <PopoverApp />
    </ResearchProvider>
  </React.StrictMode>
);
