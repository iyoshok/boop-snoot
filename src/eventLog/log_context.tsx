import { createStore } from "solid-js/store";
import { createContext } from "solid-js";

export interface LogEntry {
    time: Date,
    content: string,
}

export const LogContext = createContext();

export function LogContextProvider(props) {
    const initial: { entries: LogEntry[] } = { entries: [] };
    const [state, setState] = createStore(initial);
    const store = [
            state,
            {
                addEntry(newEntry: LogEntry) {
                    setState("entries", (e) => [...e, newEntry]);
                }
            }
        ]
    
    return (
        <LogContext.Provider value={store}>
            {props.children}
        </LogContext.Provider>
    )
}