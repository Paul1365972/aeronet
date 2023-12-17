//!

use aeronet::{ClientState, ToServer, TransportClient, TransportClientPlugin};
use aeronet_example::{client_log, log_lines, msg_buf, url_buf, AppProtocol, Log, LogLine};
use aeronet_wt_wasm::{WebTransportClient, WebTransportConfig};
use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use wasm_bindgen_futures::spawn_local;

type Client = WebTransportClient<AppProtocol>;

// logic

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: tracing::Level::DEBUG,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
            EguiPlugin,
            TransportClientPlugin::<_, Client>::default(),
        ))
        .init_resource::<Client>()
        .init_resource::<ClientUiState>()
        .add_systems(Update, (client_log::<_, Client, ClientUiState>, ui).chain())
        .run();
}

#[derive(Debug, Default, Resource)]
struct ClientUiState {
    log: Vec<LogLine>,
    url: String,
    buf: String,
}

impl Log for ClientUiState {
    fn lines(&mut self) -> &mut Vec<LogLine> {
        &mut self.log
    }
}

fn client_config() -> WebTransportConfig {
    WebTransportConfig::default()
}

fn ui(
    mut egui: EguiContexts,
    mut client: ResMut<Client>,
    mut ui_state: ResMut<ClientUiState>,
    mut send: EventWriter<ToServer<AppProtocol>>,
) {
    egui::CentralPanel::default().show(egui.ctx_mut(), |ui| {
        let can_disconnect = matches!(
            client.state(),
            ClientState::Connecting | ClientState::Connected(_)
        );
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!can_disconnect, |ui| {
                if let Some(url) = url_buf(ui, &mut ui_state.url) {
                    let backend = client
                        .connect(client_config(), url)
                        .expect("backend should be disconnected");
                    spawn_local(backend);
                }
            });

            ui.add_enabled_ui(can_disconnect, |ui| {
                if ui.button("Disconnect").clicked() {
                    let _ = client.disconnect();
                }
            });
        });

        log_lines(ui, &ui_state.log);

        if let ClientState::Connected(info) = client.state() {
            if let Some(msg) = msg_buf(ui, &mut ui_state.buf) {
                send.send(ToServer { msg });
            }

            //ui.label(format!("RTT: {:?}", info.rtt));
        }
    });
}
