import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { sendNotification } from "@tauri-apps/api/notification";
import { createSignal, onCleanup, onMount, Show, useContext } from "solid-js";
import { sendError } from "../notifications";
import { PartnerLockContext } from "./partners_update_lock";

import './partner_row.css';

interface BoopPayload {
    partner_key: string,
}

export default function PartnerRow(props) {
    const [editing, setEditing] = createSignal(props.editing);
    const [_, { lock, unlock }] = useContext(PartnerLockContext);

    const connText = () => {
        switch (props.online) {
            case -1:
                return "Offline";
            case 1:
                return "Online";
            default:
                return "Unknown";
        }
    }

    let fieldNickname: HTMLInputElement;
    let fieldUserKey: HTMLInputElement;

    let unlisten;
    onMount(async () => {
        if (editing()) {
            lock();
        }

        unlisten = await listen("booped", async event => {
            if ((event.payload as BoopPayload).partner_key == props.user_key) {
                sendNotification({
                    title: "BOOP!",
                    body: `You were booped by ${props.nickname}!`
                })
            }            
        });
    })

    onCleanup(() => {
        unlisten();
    })

    const saveEntry = async () => {
        let nickname = fieldNickname.value;
        let userkey = fieldUserKey.value;

        if (userkey.length == 0) {
            fieldUserKey.classList.add("wrong-input");
            return;
        }

        if (nickname.length == 0) {
            nickname = userkey;
        }

        try {
            if (userkey != props.user_key) {
                await invoke("del_partner", { partnerKey: props.user_key });
            }

            await invoke("add_or_update_partner", {
                partner: {
                    nickname: nickname,
                    userKey: userkey
                }
            });

            unlock();
            await invoke("trigger_partners_event");
        }
        catch (err) {
            await sendError(err);
        }

        setEditing(false);
    }

    const boop = async () => {
        try {
            if (props.online >= 1) {
                await invoke("boop", { partnerKey: props.user_key });
            }            
        }
        catch (err) {
            await sendError(err);
        }
    }

    const enableEdit = () => {
        lock();
        setEditing(true);
    }    

    const deletePartner = async () => {
        try {
            await invoke("del_partner", { partnerKey: props.user_key });
            unlock();
            await invoke("trigger_partners_event");
        }
        catch (err) {
            await sendError(err);
        }
    }

    return (
        <>
            <div class="row">
                <Show when={!editing()}>
                    <button class="boop-button" onClick={async () => await boop()} ><img src="images/heart.ico" /></button>
                    <span class="nickname">{props.nickname}</span>
                    <span class="userkey">({props.user_key})</span>
                    <span classList={{
                        userindicator: true,
                        online: props.online == 1
                    }}></span>
                    <button onClick={enableEdit}>...</button>
                </Show>

                <Show when={editing()} fallback={
                    <input type="text" value={props.nickname} placeholder={props.user_key} ref={fieldNickname} />
                    <input type="text" value={props.user_key} ref={fieldUserKey} onKeyDown={async (event) => {
                        if (event.key === "Enter") {
                            await saveEntry();
                        }
                    }} />
                    <button onClick={async () => await saveEntry()}>save</button>
                    <button onClick={async () => await deletePartner()}>delete</button>
                }>
                    <input type="text" value={props.nickname} placeholder={props.user_key} ref={fieldNickname} />
                    <input type="text" value={props.user_key} ref={fieldUserKey} onKeyDown={async (event) => {
                        if (event.key === "Enter") {
                            await saveEntry();
                        }
                    }} />
                    <button onClick={async () => await saveEntry()}>save</button>
                    <button onClick={async () => await deletePartner()}>delete</button>
                </Show>
            </div>
        </>
    )
}