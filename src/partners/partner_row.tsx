import { invoke } from "@tauri-apps/api";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { sendNotification } from "@tauri-apps/api/notification";
import { createSignal, onCleanup, onMount, Show, useContext } from "solid-js";
import { unwrap } from "solid-js/store";
import { sendError } from "../connection";
import { BoopTimerContext } from "./booptimers";

import './partner_row.css';
import editIcon from '../../icons/edit.svg';
import saveIcon from '../../icons/save.svg';
import deleteIcon from '../../icons/delete.svg';

interface BoopPayload {
    partner_key: string,
}

export default function PartnerRow(props) {
    const [boops, { updateBoops }] = useContext(BoopTimerContext);
    const [lastBoopTimeText, setBoopTimeText] = createSignal("none yet (╥﹏╥)");
    const [editing, setEditing] = createSignal(false);
    const [hovering, setHovering] = createSignal(false);
    const [clicking, setClicking] = createSignal(false);

    let mainDiv: HTMLDivElement;
    let editButton: HTMLButtonElement;
    let fieldNickname: HTMLInputElement;
    let fieldUserKey: HTMLInputElement;

    let boopUnlisten: UnlistenFn;
    let boopTimeInterval;    

    onMount(async () => {
        boopUnlisten = await listen("booped", async event => {
            if ((event.payload as BoopPayload).partner_key == props.user_key) {
                updateBoops(props.user_key);
                sendNotification({
                    title: "BOOP!",
                    body: `You were booped by ${props.nickname}!`
                })

                // play animation
                props.boopAnim(props.nickname);

                // update boop time now
                setBoopTimeText(formatTimeSince());
                
                if (!boopTimeInterval) {
                    // update last boop time text every minute 
                    boopTimeInterval = setInterval(() => {
                        setBoopTimeText(formatTimeSince());
                    }, 60000);
                }
            }            
        });

        mainDiv.addEventListener("mouseover", () => setHovering(true));
        mainDiv.addEventListener("mouseleave", () => setHovering(false));
        mainDiv.addEventListener("click", handleClick);
    })

    onCleanup(() => {
        boopUnlisten();
        clearInterval(boopTimeInterval);

        mainDiv.removeEventListener("mouseover", () => setHovering(true));
        mainDiv.removeEventListener("mouseleave", () => setHovering(false));
        mainDiv.removeEventListener("click", handleClick);
    })

    const handleClick = async (event) => {        
        if (!editButton.contains(event.target) && !editing() && !clicking()) {
            setClicking(true);
            await boop();
            setTimeout(() => {setClicking(false)}, 3000);
        }        
    }

    const saveEntry = async () => {
        let nickname = fieldNickname.value;
        let new_user_key = fieldUserKey.value;

        if (new_user_key.length == 0) {
            fieldUserKey.classList.add("wrong-input");
            return;
        }

        if (nickname.length == 0) {
            nickname = new_user_key;
        }

        setEditing(false);

        //idx: number, previous_user_key: string, user_key: string, nickname: string | null
        await props.saver(props.idx, props.user_key, new_user_key, nickname);
    }

    const deleteEntry = async () => {
        await props.remover(props.idx, props.user_key.length > 0 ? props.user_key : null);
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

    const formatTimeSince = () => {
        if (boops[props.user_key] === undefined) {
            return "none yet (╥﹏╥)";
        }

        const diff = (new Date().getTime()) / 1000 - boops[props.user_key];
        if (diff < 15) {
            return "just now";
        }
        else if (diff < 60) {
            return "less than a minute ago";
        }
        else if (diff >= 60 && diff < 3600) {
            const minutes = Math.floor(diff / 60);
            return `${minutes} minute${minutes > 1 ? "s" : ""} ago`;
        }
        else {
            const hours = Math.floor(diff / 3600);
            return `${hours} hour${hours > 1 ? "s" : ""} ago`;
        }
    }

    return (
        <>
            <div ref={mainDiv} classList={{
                row: true,
                hovering: !editing() && hovering() && !clicking(),
                clicking: !editing() && clicking()
            }}>
                <Show when={!editing()}>
                    <div class="first-row" >
                        <p class="nickname">{props.nickname && props.nickname.length > 0 ? props.nickname : props.user_key}</p>
                        <span classList={{
                            indicator: true,
                            online: props.online == 1
                        }}>{}</span>
                        <button class="edit" onClick={() => setEditing(true)} ref={editButton}><img src={editIcon} alt="edit" /></button>
                    </div>
                    <p class="last-boop">last boop: {lastBoopTimeText()}</p>
                </Show>

                <Show when={editing()}>
                    <div class="editing-section">
                        <div class="group">
                            <label>
                                Nickname
                                <input class="input-nickname" type="text" name="nickname" value={props.nickname} placeholder={"nickname"} ref={fieldNickname} />
                            </label>
                        </div>
                        <div class="group">
                            <label>
                                Username
                                <input class="input-user" type="text" value={props.user_key} placeholder={"username"} ref={fieldUserKey} onKeyDown={async (event) => {
                                    if (event.key === "Enter") {
                                        await saveEntry();
                                    }
                                }} /> 
                            </label>
                        </div>          
                        <div class="buttons">
                            <button class="button-edit" onClick={async () => await saveEntry()}><img src={saveIcon} alt="save" /></button>
                            <button class="button-delete"onClick={async () => await deleteEntry()}><img src={deleteIcon} alt="delete" /></button>
                        </div>
                    </div>
                </Show>
            </div>
        </>
    )
}