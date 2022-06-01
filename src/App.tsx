import { Component, createSignal, onMount, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from '@tauri-apps/api/window'
import { isPermissionGranted, requestPermission } from '@tauri-apps/api/notification';

import Settings from './settings/settings';
import ConnectionIndicator from './indicator/indicator';
import PartnersList from './partners/partners_list';

import './app.css';
import { PartnerUpdateLockProvider  } from './partners/partners_update_lock';


const App: Component = () => {
  const [isLoggedIn, setIsLoggedIn] = createSignal(false);
  const [showSettings, setShowSettings] = createSignal(false);

  onMount(async () => {
    await invoke("show_main_window");

    appWindow.listen('tauri://close-requested', async (_) => {
      await invoke("disconnect");
      appWindow.close();
    })

    if (!(await isPermissionGranted())) {
      await requestPermission();
    }

    try {
      const success: boolean = await invoke("connect");
      setIsLoggedIn(success);
    }
    catch(err) {
      console.error("connection failed");
    }
  })

  return (
    <>
      <Show when={showSettings()}>
          <Settings />
      </Show>
      <Show when={!showSettings()}>
        <PartnerUpdateLockProvider>
          <PartnersList />
        </PartnerUpdateLockProvider>
      </Show>  
      <ConnectionIndicator />      
      <button onClick={() => setShowSettings(show => !show)} style={{ position: 'fixed', bottom: "10px", left: "10px" }}>{showSettings() ? "Close" : "Open"} Settings</button>
    </>
  );
};

export default App;

// for boop: https://codepen.io/Zaku/pen/gOrjOGp