import { createSignal, createResource, onMount, createEffect, Suspense, Accessor, Setter } from "solid-js";
import { invoke } from '@tauri-apps/api/tauri'

import './style.settings.css';

interface SettingsPayload {
    serverAddress: string;
    user: string;
    password: string;
}

export default function Settings(props) {
    let fieldServer: HTMLInputElement;
    let fieldUser: HTMLInputElement;
    let fieldPassword: HTMLInputElement;
    let settingsContainer: HTMLDivElement;

    onMount(() => {
        invoke('get_settings')
            .then((fetchedSettings: SettingsPayload) => {
                fieldServer.value = fetchedSettings.serverAddress;
                fieldUser.value = fetchedSettings.user;
                fieldPassword.value = fetchedSettings.password;
            })
            .catch(err => alert(err));
    });

    const saveClick = async () => {
        const newSettings: SettingsPayload = {
            serverAddress: fieldServer.value,
            user: fieldUser.value,
            password: fieldPassword.value
        };

        invoke("save_settings", { newSettings: newSettings })
            .then(() => console.log("save successful"))
            .catch(err => {
                console.error(err);
                alert("save failed");
            })
    };

    return (
        <>
            < div class="settings" ref={settingsContainer}>
                <Suspense fallback={<p>Loading settings...</p>}>
                    <h2>Settings</h2>
                    <div class="settings-field">
                        <label for="settings-server-address" >Server</label>
                        <input type="text" id="settings-server-address" class="textbox" ref={fieldServer} />
                    </div>
                    <div class="settings-field">
                        <label for="settings-user">User</label>
                        <input type="text" id="settings-user" class="textbox" ref={fieldUser} />
                    </div>
                    <div class="settings-field">
                        <label for="settings-password">Password</label>
                        <input type="password" id="settings-password" class="textbox" ref={fieldPassword} />
                    </div>
                    <div class="buttons">
                        <button onClick={() => settingsContainer.style.display = "none"}>Close</button>
                        <button onClick={async () => await saveClick()} >Save</button>
                    </div>
                </Suspense>
            </div >
        </>
    )
};