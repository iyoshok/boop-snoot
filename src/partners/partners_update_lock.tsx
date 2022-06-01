import { createContext, createSignal } from "solid-js";

export const PartnerLockContext = createContext();

export function PartnerUpdateLockProvider(props) {
    const [lockStack, setLockStack] = createSignal(0);
    const isLocked = () => lockStack() > 0;
    const store = [
        isLocked,
        {
            lock() {
                setLockStack(s => s + 1);
            },
            unlock() {
                setLockStack(s => s - 1);
            }
        }        
    ]

    return (
        <PartnerLockContext.Provider value={store}>
            {props.children}
        </PartnerLockContext.Provider>
    )
}