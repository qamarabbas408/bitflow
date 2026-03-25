// src/Settings.tsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './Settings.css'; // Create this CSS file

interface InterfaceInfo {
    name: string;
    selected: boolean;
}

function Settings() {
    const [interfaces, setInterfaces] = useState<InterfaceInfo[]>([]);
    const [initialLoad, setInitialLoad] = useState(true);

    useEffect(() => {
        async function fetchInterfaces() {
            try {
                const allInterfaces: string[] = await invoke('get_all_network_interfaces');
                const loadedSelected: string[] = await invoke('load_selected_interfaces');

                const combined = allInterfaces.map(name => ({
                    name,
                    selected: loadedSelected.includes(name)
                }));
                setInterfaces(combined);
                setInitialLoad(false);
            } catch (error) {
                console.error("Failed to fetch network interfaces:", error);
            }
        }
        fetchInterfaces();
    }, []);

    const handleCheckboxChange = (interfaceName: string) => {
        setInterfaces(prev =>
            prev.map(iface =>
                iface.name === interfaceName ? { ...iface, selected: !iface.selected } : iface
            )
        );
    };

    const handleSave = async () => {
        const selectedNames = interfaces.filter(iface => iface.selected).map(iface => iface.name);
        try {
            await invoke('save_selected_interfaces', { selected: selectedNames });
            console.log("Settings saved successfully!");
            // Optionally close window or show confirmation
        } catch (error) {
            console.error("Failed to save settings:", error);
        }
    };

    const handleClear = async () => {
        try {
            await invoke('clear_selected_interfaces');
            // After clearing, reload the interfaces to reflect the cleared state
            const allInterfaces = await invoke<string[]>('get_all_network_interfaces');
            const combined = allInterfaces.map(name => ({
                name,
                selected: false
            }));
            setInterfaces(combined);
        } catch (error) {
            console.error("Failed to clear selection:", error);
        }
    };

    // If no interfaces are explicitly selected, we assume all are monitored.
    // The save button should indicate this default behavior.
    const noneAreSelected = interfaces.every(iface => !iface.selected);

    return (
        <div className="settings-container">
            <h2>Network Interfaces</h2>
            <p className="settings-description">
                Select the network interfaces you wish to monitor. If none are selected, all available physical interfaces will be monitored by default.
            </p>
            {initialLoad ? (
                <div className="spinner-small" />
            ) : interfaces.length === 0 ? (
                <p>No network interfaces found.</p>
            ) : (
                <div className="interface-list">
                    {interfaces.map(iface => (
                        <label key={iface.name} className="interface-item">
                            <input
                                type="checkbox"
                                checked={iface.selected}
                                onChange={() => handleCheckboxChange(iface.name)}
                            />
                            {iface.name}
                        </label>
                    ))}
                </div>
            )}
            
            <div className="button-row">
                <button 
                    className="clear-button" 
                    onClick={handleClear}
                    disabled={initialLoad}
                >
                    Clear Selection
                </button>
                <button 
                    className="save-button" 
                    onClick={handleSave}
                    disabled={initialLoad}
                >
                    {noneAreSelected && interfaces.length > 0 ? "Save (Monitoring All)" : "Save Selection"}
                </button>
            </div>
        </div>
    );
}

export default Settings;