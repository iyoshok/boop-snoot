import { listen } from '@tauri-apps/api/event';
import { sendNotification } from '@tauri-apps/api/notification';
import { ConnectionStatusPayload } from './indicator/indicator';

interface ErrorEventPayload {
    message: string
}

await listen("backend-error", async (event) => {
    await sendError((event.payload as ErrorEventPayload).message);
});

await listen("connection-state-changed", event => {
    switch ((event.payload as ConnectionStatusPayload).status) {
        case -1:
            sendNotification({
                title: "Disconnected",
                body: "Disconnected from boop server"
            });
            break;
        case 1:
            sendNotification({
                title: "Connected",
                body: "Connected to boop server"
            });
            break;
        default:
            break;
    }
})

export async function sendError(err: string) {
    sendNotification({
        title: "Error",
        body: `An error occurred: ${err}`
      })
}