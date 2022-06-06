import { Component, createSignal, onCleanup, onMount, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from '@tauri-apps/api/window'
import { isPermissionGranted, requestPermission } from '@tauri-apps/api/notification';
import { UnlistenFn } from '@tauri-apps/api/event';
import "@lottiefiles/lottie-player";

import './app.css';
import closeIcon from '../icons/close.svg';
import settingsIcon from '../icons/settings.svg';

import Settings from './settings/settings';
import ConnectionIndicator from './indicator/indicator';
import PartnersList from './partners/partners_list';
import { BoopTimer } from './partners/booptimers';
import { initConnection, unlistenToConnectioNEvents } from './connection';

import 'sweetalert2/src/sweetalert2.scss'
import { LottiePlayer } from '@lottiefiles/lottie-player';

const App: Component = () => {
  const [showSettings, setShowSettings] = createSignal(false);
  const [showBoop, setShowBoop] = createSignal();

  let closeEventUnlisten: UnlistenFn;
  let player;
  onMount(async () => {
    // loading complete -> show main window
    await invoke("show_main_window");

    // ask for notifications permission
    if (!(await isPermissionGranted())) {
      await requestPermission();
    }

    // listen to the window close event to make sure the connection doesnt just get cut,
    // but instead disconnects properly
    closeEventUnlisten = await appWindow.listen('tauri://close-requested', async (_) => {
      await invoke("disconnect");
      appWindow.close();
    })

    await initConnection();
  })

  onCleanup(() => {
    if (closeEventUnlisten) {
      closeEventUnlisten();
    }

    unlistenToConnectioNEvents();
  })

  const closeSettingsAndReconnect = async () => {
    setShowSettings(false);
    await initConnection();
  }

  const showBoopAnimation = (name) => {
    setShowBoop(name);
    setTimeout(() => setShowBoop(null), 3000);
    console.log(player)
  }

  return (
    <>
      <div id="header">
        <div id="greeter">
          <span class='logo'>boop</span>
          <ConnectionIndicator />
        </div>

        <button class='settings-button' onClick={() => setShowSettings(show => !show)}>
          <Show when={!showSettings()}
            fallback={<img src={closeIcon} alt="Close Settings" />}
          >
            <img src={settingsIcon} alt="Open Settings" />
          </Show>
        </button>
      </div>

      <Show when={showBoop()}>
        <div id="boop-report">
          <lottie-player ref={player} src="https://assets7.lottiefiles.com/packages/lf20_yieehufy.json"  background="transparent"  speed="1" style="height: 40%" autoplay>
          </lottie-player>
          <p><b>{showBoop()}</b> booped you!</p>
        </div>
      </Show>

      <Show when={showSettings()}>
        <Settings savedSettings={closeSettingsAndReconnect} />
      </Show>
      <Show when={!showSettings()}>
        <BoopTimer>
          <PartnersList boopAnim={showBoopAnimation} />
        </BoopTimer>
      </Show>
    </>
  );
};

export default App;

// for boop: https://codepen.io/Zaku/pen/gOrjOGp