import { listen } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/tauri';
import { For, onCleanup, Show, useContext } from "solid-js";
import { createSignal, onMount } from "solid-js";
import { createStore, unwrap } from "solid-js/store";
import { PartnerLockContext } from "./partners_update_lock";
import PartnerRow from "./partner_row";

import './partners_list.css';
import Swal from "sweetalert2";

export interface Partner {
    nickname: string,
    user_key: string,
    online: number
}

interface PartnerEventPayload {
    user_key: string,
    online: number
}

interface PartnerStore {
    partners: Partner[]
}

export default function PartnersList(props) {
    const [partnersContainer, setPartners] = createStore({ partners: [] } as PartnerStore);
    const [errored, setErrored] = createSignal(false);

    let unlisten;
    onMount(async () => {
        await fetchAll();

        unlisten = await listen("partner-status-changed", event => {
            const casted = event.payload as PartnerEventPayload;
            setPartners("partners", p => p.user_key === casted.user_key, "online", casted.online);
        });
    })

    onCleanup(() => {
        unlisten();
    });

    const fetchAll = async () => {
        try {
            const partnersState: PartnerStore = {
                partners: await invoke("get_partners")
            };
            setPartners(partnersState);
        }
        catch(err) {
            setErrored(true);
        }
    }

    const removeItem = async (idx: number, user_key: string | null) => {
        setPartners("partners", idx, entry => undefined);

        if (user_key != null && user_key.length > 0) {
            try {
                await invoke("del_partner", { partnerKey: user_key });
            }
            catch (err) {
                await Swal.fire({
                    title: "Deleting failed",
                    text: "The partner entry couldn't be deleted.",
                    icon: "error",
                    toast: true,
                    timer: 5000,
                    timerProgressBar: true
                });
            }
        }

        await fetchAll();
    }

    const saveItem = async (idx: number, previous_user_key: string, user_key: string, nickname: string) => {
        try {
            if (user_key != previous_user_key) {
                await invoke("del_partner", { partnerKey: previous_user_key });
            }

            await invoke("add_or_update_partner", {
                partner: {
                    nickname: nickname,
                    userKey: user_key
                }
            });
        }
        catch (err) {
            await Swal.fire({
                title: "Saving failed",
                text: "The partner entry couldn't be saved.",
                icon: "error",
                toast: true,
                timer: 5000,
                timerProgressBar: true
            });
        }

        await fetchAll();
    }

    let partnersListElement: HTMLUListElement;

    return (
        <>
            <Show when={!errored()} 
                fallback={<p style={{ "font-size": "1.5em", width: "100%", "margin-top": "10%", "text-align": "center", display: "flex", "justify-content": "center", "align-items": "center"}}>
                    <b style={{ "margin-right": "20px", "letter-spacing": "2px" }}>Failed to load Partners</b> (╥﹏╥)
                </p>}
            >
                <ul ref={partnersListElement}>
                    <For each={partnersContainer.partners}>{(partner, i) =>
                        <PartnerRow {...partner} idx={i} remover={removeItem} saver={saveItem} boopAnim={props.boopAnim} />
                    }</For>
                </ul>

                <div id="add-container">
                    <button id="add-partner" onClick={() => setPartners("partners", partners => [...partners, { nickname: "", user_key: "", online: 0 }])}>add new partner</button>
                </div>
            </Show>
        </>
    )
}