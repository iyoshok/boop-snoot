import { Accessor, createSignal, Setter, For, onMount, useContext } from "solid-js";
import { LogContext, LogEntry } from "./log_context";
import LogElement from "./log_element";

export default function EventLog(props) {
    const [ state, _ ] = useContext(LogContext);

    return (
        <>
            <ul id="log-container" style={{ "list-style-type": "none", padding: "0" }}>
                <For each={state.entries}>{(entry, i) =>
                    <LogElement {...entry} />
                }</For>
            </ul>
        </>
    )
}