import { invoke } from '@tauri-apps/api';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { sendNotification } from '@tauri-apps/api/notification';
import Swal from 'sweetalert2'
import tryer from 'tryer';
import { ConnectionStatusPayload } from './indicator/indicator';

interface ErrorEventPayload {
  message: string
}

export async function initConnection() {
  if (alreadyAttemptingConnection)
    return;

  let connected = false;
  let shouldRetry = true;
  alreadyAttemptingConnection = true;
  tryer({
    action: async () => { [connected, shouldRetry] = await tryConnection(); },
    when: () => !connected && shouldRetry,
    until: () => connected || !shouldRetry,
    interval: 1000,
    limit: 10,
    fail: () => {
      sendNotification({
        title: "Connection failed",
        body: "All connection attempts to the boop server failed. Please check your settings and internet connection."
      })
    },
    pass: async () => {
      await initNotifications();
      alreadyAttemptingConnection = false;
    }
  })
}

// first: was connection attempt successful?
// second: should it be retried?
async function tryConnection(): Promise<[boolean, boolean]> {
    try {
      const success: boolean = await invoke("connect");

      if (!success) {
        console.log("login was refused by server");

        await Swal.fire({
          title: "Login credentials refused (>_<)",
          text: "Please change your settings and try again. If the problem persists, there might be some more serious fuckery afoot.",
          icon: "error",
          showCancelButton: false,
          showConfirmButton: true,
        });

        return [false, false];
      }

      return [true, false];
    }
    catch (err) {
      console.log("connection attempt failed");
      return [false, true];
    }
}

let notifUnlisten: UnlistenFn;
let alreadyAttemptingConnection = false;

export async function initNotifications() {
  await listen("backend-error", async (event) => {
      await sendError((event.payload as ErrorEventPayload).message);
  });
  
  notifUnlisten = await listen("connection-state-changed", event => {
      if ((event.payload as ConnectionStatusPayload).status == -1) {
          if (alreadyAttemptingConnection)
            return;
            
          alreadyAttemptingConnection = true;

          let connected = false;
          tryer({
            action: async () => { connected = await tryConnection(); },
            until: () => connected,
            interval: 2000,
            pass: async () => {
              alreadyAttemptingConnection = false;
            }
          })
      }
  });
}

export function unlistenToConnectioNEvents() {
  if (notifUnlisten) {
    notifUnlisten();
  }

  notifUnlisten = null;
}

export async function sendError(err: string) {
  await Swal.fire({
      title: "Error",
      text: `An error occurred: ${err}`,
      showConfirmButton: true,
      showCancelButton: false
  })
}