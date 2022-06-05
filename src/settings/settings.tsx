import { createSignal, createResource, onMount, createEffect, Suspense, Accessor, Setter } from "solid-js";
import { invoke } from '@tauri-apps/api/tauri'

import './settings.css';

interface SettingsPayload {
    serverAddress: string;
    user: string;
    password: string;
}

export default function Settings(props) {
    let fieldServer: HTMLInputElement;
    let fieldUser: HTMLInputElement;
    let fieldPassword: HTMLInputElement;

    const fetchAndAssignSettings = async () => {
        try {
            const fetchedSettings: SettingsPayload = await invoke('get_settings');
            fieldServer.value = fetchedSettings.serverAddress;
            fieldUser.value = fetchedSettings.user;
            fieldPassword.value = fetchedSettings.password;
        }
        catch(err) {
            console.error(err);
        }
    }

    onMount(async () => { await fetchAndAssignSettings(); });

    const saveClick = async () => {
        const newSettings: SettingsPayload = {
            serverAddress: fieldServer.value,
            user: fieldUser.value,
            password: fieldPassword.value
        };

        try {
            await invoke("save_settings", { newSettings: newSettings });
            console.log("successfully saved settings")
        }
        catch(err) {
            console.error("failed to save settings", err);
        }

        props.savedSettings();
    };

    return (
        <>
            < div class="settings" >
                <h2>Settings</h2>
                <div class="settings-inputs">
                    <label for="settings-server-address" >Server
                        <input type="text" id="settings-server-address" class="textbox" ref={fieldServer} />
                    </label>
                    <div class="login">
                        <label for="settings-user">Username
                            <input type="text" id="settings-user" class="textbox" ref={fieldUser} />
                        </label>
                        <label for="settings-password">Password
                            <input type="password" id="settings-password" class="textbox" ref={fieldPassword} />
                        </label>
                    </div>
                </div>
                <div class="save-container">
                    <button id="save-settings" onClick={async () => await saveClick()} >Save Settings</button>
                </div>
            </div >
        </>
    )
};