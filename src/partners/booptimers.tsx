import { createContext, createSignal } from "solid-js";
import { createStore } from "solid-js/store";

export const BoopTimerContext = createContext();

export function BoopTimer(props) {
    const [boops, setBoops] = createStore({} as { [id: string]: number; });
    const store = [
        boops,
        {
            updateBoops(source: string) {
                setBoops(source, _ => new Date().getTime() / 1000);
            }
        }
    ]

    return (
        <BoopTimerContext.Provider value={store}>
            {props.children}
        </BoopTimerContext.Provider>
    )
}