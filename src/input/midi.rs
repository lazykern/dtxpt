use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use midir::{Ignore, MidiInput, MidiInputConnection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiNoteEvent {
    pub device_name: String,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
    pub stamp: u64,
}

#[derive(Resource, Default)]
pub struct MidiInputState {
    pub note_on_events: Vec<MidiNoteEvent>,
    pub connected_devices: Vec<String>,
}

#[derive(Resource)]
struct MidiThreadReceiver {
    receiver: Mutex<mpsc::Receiver<MidiThreadMessage>>,
}

enum MidiThreadMessage {
    Devices(Vec<String>),
    NoteOn(MidiNoteEvent),
    LogInfo(String),
    LogWarn(String),
}

struct MidiConnectionHandle {
    _connection: MidiInputConnection<()>,
}

const MIDI_THREAD_SLEEP: Duration = Duration::from_millis(1000);
const CLIENT_NAME: &str = "dtxpt";
const PORT_NAME: &str = "dtxpt";

pub fn plugin(app: &mut App) {
    app.init_resource::<MidiInputState>()
        .add_systems(Startup, start_midi_thread)
        .add_systems(PreUpdate, poll_midi_thread);
}

fn start_midi_thread(mut commands: Commands) {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || midi_thread_main(tx));
    commands.insert_resource(MidiThreadReceiver {
        receiver: Mutex::new(rx),
    });
}

fn poll_midi_thread(receiver: Res<MidiThreadReceiver>, mut midi_state: ResMut<MidiInputState>) {
    midi_state.note_on_events.clear();

    let Ok(receiver) = receiver.receiver.lock() else {
        return;
    };

    while let Ok(message) = receiver.try_recv() {
        match message {
            MidiThreadMessage::Devices(devices) => {
                midi_state.connected_devices = devices;
            }
            MidiThreadMessage::NoteOn(event) => {
                midi_state.note_on_events.push(event);
            }
            MidiThreadMessage::LogInfo(message) => info!("{message}"),
            MidiThreadMessage::LogWarn(message) => warn!("{message}"),
        }
    }
}

fn midi_thread_main(sender: mpsc::Sender<MidiThreadMessage>) {
    let mut connections: HashMap<String, MidiConnectionHandle> = HashMap::new();
    let mut last_devices = Vec::new();
    let mut warned_missing = false;

    loop {
        let discovered = discover_midi_ports();
        let current_keys: HashSet<_> = discovered.iter().map(|port| port.key.clone()).collect();

        let stale: Vec<_> = connections
            .keys()
            .filter(|key| !current_keys.contains(*key))
            .cloned()
            .collect();
        for key in stale {
            connections.remove(&key);
        }

        let device_names: Vec<_> = discovered
            .iter()
            .map(|port| port.display_name.clone())
            .collect();
        if device_names != last_devices {
            let _ = sender.send(MidiThreadMessage::Devices(device_names.clone()));
            last_devices = device_names.clone();
        }

        if discovered.is_empty() {
            if !warned_missing {
                let _ = sender.send(MidiThreadMessage::LogWarn(
                    "MIDI: no input ports found; retrying in background".to_string(),
                ));
                warned_missing = true;
            }
            thread::sleep(MIDI_THREAD_SLEEP);
            continue;
        }
        warned_missing = false;

        for port in discovered {
            if connections.contains_key(&port.key) {
                continue;
            }

            match connect_port(&port.display_name, &port.raw_port, sender.clone()) {
                Ok(connection) => {
                    let _ = sender.send(MidiThreadMessage::LogInfo(format!(
                        "MIDI: connected to \"{}\"",
                        port.display_name
                    )));
                    connections.insert(port.key, connection);
                }
                Err(error) => {
                    let _ = sender.send(MidiThreadMessage::LogWarn(format!(
                        "MIDI: failed to connect to \"{}\": {error}",
                        port.display_name
                    )));
                }
            }
        }

        thread::sleep(MIDI_THREAD_SLEEP);
    }
}

#[derive(Clone)]
struct MidiPortDescriptor {
    key: String,
    display_name: String,
    raw_port: midir::MidiInputPort,
}

fn discover_midi_ports() -> Vec<MidiPortDescriptor> {
    let Ok(input) = MidiInput::new(CLIENT_NAME) else {
        return Vec::new();
    };

    let ports = input.ports();
    let mut discovered = Vec::new();

    for (index, port) in ports.into_iter().enumerate() {
        let Ok(name) = input.port_name(&port) else {
            continue;
        };
        if !is_real_midi_port(&name) {
            continue;
        }

        discovered.push(MidiPortDescriptor {
            key: format!("{index}:{name}"),
            display_name: name,
            raw_port: port,
        });
    }

    discovered
}

fn connect_port(
    display_name: &str,
    port: &midir::MidiInputPort,
    sender: mpsc::Sender<MidiThreadMessage>,
) -> Result<MidiConnectionHandle, String> {
    let mut input = MidiInput::new(CLIENT_NAME).map_err(|error| error.to_string())?;
    input.ignore(Ignore::None);
    let name = display_name.to_string();

    let connection = input
        .connect(
            port,
            PORT_NAME,
            move |stamp, message, _| {
                if message.len() < 3 {
                    return;
                }

                let status = message[0];
                let kind = status & 0xF0;
                let velocity = message[2];
                if kind != 0x90 || velocity == 0 {
                    return;
                }

                let _ = sender.send(MidiThreadMessage::NoteOn(MidiNoteEvent {
                    device_name: name.clone(),
                    channel: status & 0x0F,
                    note: message[1],
                    velocity,
                    stamp,
                }));
            },
            (),
        )
        .map_err(|error| error.to_string())?;

    Ok(MidiConnectionHandle {
        _connection: connection,
    })
}

fn is_real_midi_port(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !lower.contains("midi through")
        && !lower.contains("pipewire-system")
        && !lower.contains("pipewire-rt-event")
}
