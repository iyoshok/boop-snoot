import { createSignal, Match, onCleanup, onMount, Switch } from "solid-js";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface ConnectionStatusPayload {
    // -1 -> dis- / not connected
    // 0 -> attempting connection
    // 1 -> connected
    status: number
}

export default function ConnectionIndicator(props) {
    const [connectionState, setConnectionState] = createSignal(-1);

    onMount(async () => {
        await listen("connection-state-changed", event => {
            setConnectionState((event.payload as ConnectionStatusPayload).status);
        })
    })

    const indicatorColor = () => {
        switch (connectionState()) {
            case -1:
                return "red";
            case 0:
                return "yellow";
            case 1: 
                return "green";
        }
    }

    return (
        <>
            <div id="indicator" style={{ 
                "background-color": indicatorColor(),
                width: "15px",
                height: "15px",
                "border-radius": "100%",
                position: "absolute",
                top: "10px",
                right: "10px" }}></div>
        </>
    )
}