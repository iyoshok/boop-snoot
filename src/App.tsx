import { Component, onMount, useContext } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri'

import Settings from './settings';
import ConnectionIndicator from './indicator';
import EventLog from './eventLog/event_log';
import { LogContextProvider, LogEntry, LogContext } from './eventLog/log_context';

import './app.css';

const App: Component = () => {
  return (
    <>      
      <LogContextProvider>
        <div class="app-container">
          <div class="header"></div>
          <div class="log-side">
            <EventLog />
          </div>
          <div class="boop-side">

          </div>
          <ConnectionIndicator />
        </div>           
      </LogContextProvider>
    </>
  );
};

export default App;

// for boop: https://codepen.io/Zaku/pen/gOrjOGp