import { listen } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/tauri';
import { For, useContext } from "solid-js";
import { Accessor, createSignal, onMount, Setter } from "solid-js";
import { PartnerLockContext } from "./partners_update_lock";
import PartnerRow from "./partner_row";

export interface Partner {
    nickname: string,
    user_key: string,
    online: number
}

interface PartnerEventPayload {
    partners: Partner[]
}

export default function PartnersList(props) {
    const [partners, setPartners]: [Accessor<Partner[]>, Setter<Partner[]>] = createSignal([])
    const [locked, _] = useContext(PartnerLockContext);

    onMount(async () => {
        await listen("partners-update", event => {
            if (!locked()) {
                const deconstructed = event.payload as PartnerEventPayload;
                setPartners(deconstructed.partners);
            }
        });

        await invoke("trigger_partners_event");
    })

    let partnersListElement: HTMLUListElement;

    return (
        <>
            <ul ref={partnersListElement}>
                <For each={partners()}>{(partner, i) =>
                    <PartnerRow {...partner} editing={partner.user_key.length == 0} />
                }</For>
            </ul>

            <button onClick={() => setPartners(curr => [...curr, { nickname: "", user_key: "", online: 0 }])}>Add New Partner</button>
        </>
    )
}