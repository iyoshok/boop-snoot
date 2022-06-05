import { createSignal, Match, onCleanup, onMount, Switch } from "solid-js";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

import './indicator.css';

export interface ConnectionStatusPayload {
    // -1 -> dis- / not connected
    // 0 -> attempting connection
    // 1 -> connected
    status: number
}

export default function ConnectionIndicator(props) {
    const [connectionState, setConnectionState] = createSignal(-1);

    let unlisten: UnlistenFn;
    onMount(async () => {
        unlisten = await listen("connection-state-changed", event => {
            setConnectionState((event.payload as ConnectionStatusPayload).status);
        })
    })

    onCleanup(() => {
        unlisten();
    })

    const indicatorText = () => {
        switch (connectionState()) {
            case 1:
                return "connected";
            case 0:
                return "connecting...";
            case -1: 
                return "disconnected";
        }
    }

    return (
        <>
            <span id="indicator" style={{ cursor: "default" }} classList={{
                connected: connectionState() == 1,
                disconnected: connectionState() == -1,
                connecting: connectionState() == 0,
            }}>{indicatorText()}</span>
        </>
    )
}