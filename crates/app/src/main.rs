//! MCProster desktop app (iced): one window — a dense table of servers × hosts, a detail
//! panel with copy/open buttons and the probe, the findings underneath. Reads the same
//! core as the CLI; writes only its settings and the probe results (BRIEF R8).
#![forbid(unsafe_code)]

mod i18n;
mod settings;

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, mouse_area, pick_list,
    row, scrollable, text, text_input, Space,
};
use iced::{Background, Border, Color, Element, Length, Task, Theme};

use mcproster_core::actions::{self, Action};
use mcproster_core::cost;
use mcproster_core::hosts::SourceKind;
use mcproster_core::model::{Declaration, FindingKind, Transport};
use mcproster_core::probe::{self, ProbeResult};
use mcproster_core::{inventory, Env, Inventory, Rules};

use i18n::{Lang, Strings};
use settings::Settings;

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .window_size((1280.0, 800.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<Arc<Inventory>, String>),
    Rescan,
    SetLang(Lang),
    ToggleTheme,
    Filter(String),
    HostFilter(Option<String>),
    ToggleFindingsOnly(bool),
    ToggleShowOff(bool),
    Select(usize),
    SelectFinding(usize),
    Copy(String),
    OpenFile(PathBuf),
    ProbeRequest(usize),
    ProbeAllRequest,
    ProbeAskToggle(bool),
    ProbeConfirm,
    ProbeCancel,
    Probed(String, Arc<ProbeResult>),
    ToggleTools(String),
}

/// Beyond this many hosts with configuration, the table shows one "hosts" column.
const MAX_HOST_COLUMNS: usize = 6;

struct App {
    lang: Lang,
    strings: Strings,
    dark: bool,
    env: Env,
    rules: Arc<Rules>,
    inventory: Option<Arc<Inventory>>,
    loading: bool,
    error: Option<String>,
    filter: String,
    host_filter: Option<String>,
    findings_only: bool,
    show_off: bool,
    selected: Option<usize>,
    probes: BTreeMap<String, ProbeResult>,
    probing: Option<String>,
    ask_before_probe: bool,
    pending_probe: Option<Vec<usize>>,
    probe_queue: Vec<usize>,
    dont_ask: bool,
    open_tools: HashSet<String>,
    status: String,
}

impl App {
    fn new() -> (App, Task<Message>) {
        let settings = Settings::load();
        let lang = settings.initial_lang();
        let strings = Strings::load(lang);
        let env = Env::current();
        let rules = Arc::new(mcproster_core::hosts::embedded());
        let probes = probe::load_results(&env.home);
        let status = strings.t("header.scanning", &[]);
        let task = load(rules.clone(), env.clone());
        let app = App {
            lang,
            strings,
            dark: settings.initial_dark(),
            env,
            rules,
            inventory: None,
            loading: true,
            error: None,
            filter: String::new(),
            host_filter: None,
            findings_only: false,
            show_off: true,
            selected: None,
            probes,
            probing: None,
            ask_before_probe: settings.asks_before_probe(),
            pending_probe: None,
            probe_queue: Vec::new(),
            dont_ask: false,
            open_tools: HashSet::new(),
            status,
        };
        (app, task)
    }

    fn title(&self) -> String {
        self.strings.t("app.title", &[])
    }

    fn theme(&self) -> Theme {
        if self.dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn remember(&self) {
        Settings {
            lang: Some(self.lang.code().to_string()),
            theme: Some(if self.dark { "dark" } else { "light" }.to_string()),
            ask_before_probe: Some(self.ask_before_probe),
        }
        .save();
    }

    fn refresh_status(&mut self) {
        self.status = match &self.inventory {
            None => self.strings.t("header.scanning", &[]),
            Some(inv) => {
                let with = inv.hosts.iter().filter(|h| h.files_found > 0).count();
                let observed = inv
                    .declarations
                    .iter()
                    .filter(|d| d.source_kind == SourceKind::Observed)
                    .count();
                self.strings.t(
                    "footer.summary",
                    &[
                        ("known", &inv.hosts.len().to_string()),
                        ("with", &with.to_string()),
                        ("observed", &observed.to_string()),
                    ],
                )
            }
        };
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(Ok(inv)) => {
                self.inventory = Some(inv);
                self.loading = false;
                self.error = None;
                if let Some(s) = self.selected {
                    if s >= self
                        .inventory
                        .as_ref()
                        .map(|i| i.servers.len())
                        .unwrap_or(0)
                    {
                        self.selected = None;
                    }
                }
                self.refresh_status();
            }
            Message::Loaded(Err(e)) => {
                self.loading = false;
                self.error = Some(e);
            }
            Message::Rescan => {
                self.loading = true;
                self.status = self.strings.t("header.scanning", &[]);
                return load(self.rules.clone(), self.env.clone());
            }
            Message::SetLang(l) => {
                self.lang = l;
                self.strings = Strings::load(l);
                self.remember();
                self.refresh_status();
            }
            Message::ToggleTheme => {
                self.dark = !self.dark;
                self.remember();
            }
            Message::Filter(f) => self.filter = f,
            Message::HostFilter(h) => {
                self.host_filter = h.and_then(|name| {
                    self.hosts_with_config()
                        .into_iter()
                        .find(|(_, n)| *n == name)
                        .map(|(id, _)| id)
                });
            }
            Message::ToggleFindingsOnly(b) => self.findings_only = b,
            Message::ToggleShowOff(b) => self.show_off = b,
            Message::Select(i) => self.selected = Some(i),
            Message::SelectFinding(fi) => {
                if let Some(inv) = &self.inventory {
                    if let Some(f) = inv.findings.get(fi) {
                        if let Some(&d) = f.declarations.first() {
                            self.selected =
                                inv.servers.iter().position(|s| s.declarations.contains(&d));
                        }
                    }
                }
            }
            Message::Copy(s) => {
                self.status = self.strings.t("footer.copied", &[]);
                return iced::clipboard::write(s);
            }
            Message::OpenFile(p) => {
                let _ = std::process::Command::new("xdg-open").arg(&p).spawn();
                self.status = self
                    .strings
                    .t("footer.opened", &[("file", &short(&p, &self.env.home))]);
            }
            Message::ProbeRequest(si) => {
                if self.ask_before_probe {
                    self.pending_probe = Some(vec![si]);
                    self.dont_ask = false;
                } else {
                    self.probe_queue = vec![si];
                    return self.next_probe();
                }
            }
            Message::ProbeAllRequest => {
                let all = self.probeable_servers();
                if all.is_empty() {
                    return Task::none();
                }
                if self.ask_before_probe {
                    self.pending_probe = Some(all);
                    self.dont_ask = false;
                } else {
                    self.probe_queue = all;
                    return self.next_probe();
                }
            }
            Message::ProbeAskToggle(b) => self.dont_ask = b,
            Message::ProbeConfirm => {
                if self.dont_ask {
                    self.ask_before_probe = false;
                    self.remember();
                }
                if let Some(list) = self.pending_probe.take() {
                    self.probe_queue = list;
                    return self.next_probe();
                }
            }
            Message::ProbeCancel => self.pending_probe = None,
            Message::Probed(identity, result) => {
                self.probing = None;
                self.probes.insert(identity, (*result).clone());
                let _ = probe::save_results(&self.env.home, &self.probes);
                return self.next_probe();
            }
            Message::ToggleTools(id) => {
                if !self.open_tools.remove(&id) {
                    self.open_tools.insert(id);
                }
            }
        }
        Task::none()
    }

    fn fg_color(&self) -> Color {
        if self.dark {
            Color::from_rgb(0.92, 0.92, 0.94)
        } else {
            Color::from_rgb(0.1, 0.1, 0.12)
        }
    }

    /// Servers with at least one enabled stdio declaration, in table order.
    fn probeable_servers(&self) -> Vec<usize> {
        let Some(inv) = &self.inventory else {
            return Vec::new();
        };
        (0..inv.servers.len())
            .filter(|&si| {
                inv.servers[si]
                    .declarations
                    .iter()
                    .map(|&i| &inv.declarations[i])
                    .any(|d| d.enabled && d.transport == Transport::Stdio)
            })
            .collect()
    }

    /// Start the next queued probe, one at a time.
    fn next_probe(&mut self) -> Task<Message> {
        if self.probe_queue.is_empty() {
            return Task::none();
        }
        let si = self.probe_queue.remove(0);
        self.start_probe(si)
    }

    /// The first stdio declaration of a server gives the launch; the probe runs off-thread.
    fn start_probe(&mut self, si: usize) -> Task<Message> {
        let Some(inv) = &self.inventory else {
            return Task::none();
        };
        let Some(server) = inv.servers.get(si) else {
            return Task::none();
        };
        let Some(d) = server
            .declarations
            .iter()
            .map(|&i| &inv.declarations[i])
            .find(|d| d.transport == Transport::Stdio)
        else {
            return Task::none();
        };
        let Some((command, args, child_env, cwd)) = inventory::raw_launch(&self.rules, d) else {
            return Task::none();
        };
        let dir = d
            .file
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let command = command.replace("${__dirname}", &dir);
        let args: Vec<String> = args
            .iter()
            .map(|a| a.replace("${__dirname}", &dir))
            .collect();
        let identity = server.identity.clone();
        self.probing = Some(identity.clone());
        let vars = self.env.vars.clone();
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let r = probe::probe_stdio(
                &command,
                &args,
                &child_env,
                cwd.as_deref(),
                &vars,
                Duration::from_secs(10),
            );
            let _ = tx.send(r);
        });
        Task::perform(async move { rx.await.ok() }, move |r| match r {
            Some(r) => Message::Probed(identity.clone(), Arc::new(r)),
            None => Message::Probed(
                identity.clone(),
                Arc::new(ProbeResult {
                    at: String::new(),
                    command: String::new(),
                    ok: false,
                    error: Some("probe thread stopped".into()),
                    server_name: None,
                    server_version: None,
                    protocol_version: None,
                    tools: vec![],
                    schema_bytes: 0,
                    estimated_tokens: 0,
                    duration_ms: 0,
                    stderr: vec![],
                }),
            ),
        })
    }

    // ------------------------------------------------------------------ views

    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = match (&self.inventory, &self.error) {
            (Some(inv), _) => row![
                column![self.view_filters(), self.view_table(inv)]
                    .spacing(8)
                    .width(Length::FillPortion(13))
                    .height(Length::Fill),
                container(self.view_side(inv))
                    .width(Length::FillPortion(9))
                    .height(Length::Fill),
            ]
            .spacing(14)
            .height(Length::Fill)
            .into(),
            (None, Some(e)) => container(text(e.clone()).size(16))
                .center(Length::Fill)
                .into(),
            (None, None) => container(
                text(self.strings.t("header.scanning", &[]))
                    .size(16)
                    .style(text::secondary),
            )
            .center(Length::Fill)
            .into(),
        };
        let theme_label = if self.dark {
            self.strings.t("theme.light", &[])
        } else {
            self.strings.t("theme.dark", &[])
        };
        let footer = row![
            text(self.status.clone()).size(12).style(text::secondary),
            horizontal_space(),
            self.view_lang_picker(),
            Space::with_width(6),
            button(text(theme_label).size(12))
                .style(button::text)
                .on_press(Message::ToggleTheme),
        ]
        .align_y(iced::Alignment::Center);
        column![self.view_header(), body, footer]
            .spacing(14)
            .padding(16)
            .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let inv = self.inventory.as_deref();
        let n = |v: Option<usize>| v.map(|x| x.to_string()).unwrap_or_else(|| "—".into());
        let servers = inv.map(|i| i.servers.len());
        let decls = inv.map(|i| i.declarations.len());
        let with = inv.map(|i| i.hosts.iter().filter(|h| h.files_found > 0).count());
        let known = inv.map(|i| i.hosts.len());
        let findings = inv.map(|i| {
            i.findings
                .iter()
                .filter(|f| f.kind != FindingKind::Disabled)
                .count()
                + cost::report(i, &self.probes).findings.len()
        });
        let secrets = inv.map(|i| i.count(FindingKind::PlainSecret));
        let findings_dot = findings.map(|f| {
            if f == 0 {
                good(self.dark)
            } else {
                alert(self.dark)
            }
        });
        let secrets_dot = secrets.map(|f| {
            if f == 0 {
                good(self.dark)
            } else {
                alert(self.dark)
            }
        });
        row![
            self.kpi(
                &s.t("kpi.servers", &[]),
                &n(servers),
                &s.t("kpi.declarations", &[("n", &n(decls))]),
                None
            ),
            self.kpi(
                &s.t("kpi.hosts", &[]),
                &n(with),
                &s.t("kpi.hosts_known", &[("n", &n(known))]),
                None
            ),
            self.kpi(&s.t("kpi.findings", &[]), &n(findings), "", findings_dot),
            self.kpi(&s.t("kpi.secrets", &[]), &n(secrets), "", secrets_dot),
        ]
        .spacing(12)
        .into()
    }

    fn kpi<'a>(
        &self,
        label: &str,
        value: &str,
        sub: &str,
        dot: Option<Color>,
    ) -> Element<'a, Message> {
        let mut head = row![].spacing(6).align_y(iced::Alignment::Center);
        if let Some(c) = dot {
            head = head.push(swatch(c, 8));
        }
        head = head.push(text(label.to_string()).size(12).style(text::secondary));
        card(
            column![
                head,
                text(value.to_string()).size(28),
                text(sub.to_string()).size(11).style(text::secondary),
            ]
            .spacing(2)
            .into(),
        )
        .width(Length::Fill)
        .into()
    }

    fn view_lang_picker(&self) -> Element<'_, Message> {
        let pick = |lang: Lang, label: &'static str| -> Element<'_, Message> {
            let b = button(text(label).size(11)).padding([2, 7]);
            if lang == self.lang {
                b.style(button::primary).into()
            } else {
                b.style(button::text)
                    .on_press(Message::SetLang(lang))
                    .into()
            }
        };
        row![pick(Lang::En, "EN"), pick(Lang::Fr, "FR")]
            .spacing(2)
            .align_y(iced::Alignment::Center)
            .into()
    }

    fn view_filters(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let all = s.t("filter.all_hosts", &[]);
        let mut options: Vec<String> = vec![all.clone()];
        options.extend(self.hosts_with_config().into_iter().map(|(_, n)| n));
        let selected = self
            .host_filter
            .as_ref()
            .and_then(|id| {
                self.hosts_with_config()
                    .into_iter()
                    .find(|(i, _)| i == id)
                    .map(|(_, n)| n)
            })
            .unwrap_or_else(|| all.clone());
        let hosts = pick_list(options, Some(selected), move |choice: String| {
            if choice == all {
                Message::HostFilter(None)
            } else {
                Message::HostFilter(Some(choice))
            }
        })
        .text_size(12)
        .padding([4, 8])
        .width(Length::Fixed(170.0));
        row![
            text_input(&s.t("filter.placeholder", &[]), &self.filter)
                .on_input(Message::Filter)
                .size(13)
                .width(Length::Fill),
            hosts,
            checkbox(s.t("filter.findings_only", &[]), self.findings_only)
                .on_toggle(Message::ToggleFindingsOnly)
                .size(16)
                .text_size(12),
            checkbox(s.t("filter.show_off", &[]), self.show_off)
                .on_toggle(Message::ToggleShowOff)
                .size(16)
                .text_size(12),
            if self.probing.is_some() {
                button(text(s.t("probe.testing", &[])).size(12)).style(button::primary)
            } else {
                button(text(s.t("probe.test_all", &[])).size(12))
                    .style(button::primary)
                    .on_press(Message::ProbeAllRequest)
            },
            if self.loading {
                button(text(s.t("header.scanning", &[])).size(12)).style(button::secondary)
            } else {
                button(text(s.t("header.rescan", &[])).size(12))
                    .style(button::secondary)
                    .on_press(Message::Rescan)
            },
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Hosts that have at least one configuration file, `(id, name)`, in display order.
    fn hosts_with_config(&self) -> Vec<(String, String)> {
        match &self.inventory {
            Some(inv) => inv
                .hosts
                .iter()
                .filter(|h| h.files_found > 0)
                .map(|h| (h.id.clone(), h.name.clone()))
                .collect(),
            None => Vec::new(),
        }
    }

    fn server_has_finding(&self, inv: &Inventory, si: usize) -> bool {
        let server = &inv.servers[si];
        inv.findings
            .iter()
            .filter(|f| f.kind != FindingKind::Disabled)
            .any(|f| {
                f.declarations
                    .iter()
                    .any(|d| server.declarations.contains(d))
            })
    }

    fn visible_servers(&self, inv: &Inventory) -> Vec<usize> {
        let needle = self.filter.trim().to_lowercase();
        (0..inv.servers.len())
            .filter(|&si| {
                let server = &inv.servers[si];
                let decls: Vec<&Declaration> = server
                    .declarations
                    .iter()
                    .map(|&i| &inv.declarations[i])
                    .collect();
                if let Some(h) = &self.host_filter {
                    if !decls.iter().any(|d| d.hosts.contains(h)) {
                        return false;
                    }
                }
                if !self.show_off && decls.iter().all(|d| !d.enabled || d.legacy) {
                    return false;
                }
                if self.findings_only && !self.server_has_finding(inv, si) {
                    return false;
                }
                if !needle.is_empty() {
                    let hay = format!(
                        "{} {} {}",
                        server.identity,
                        decls
                            .iter()
                            .map(|d| d.name.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                        decls
                            .iter()
                            .map(|d| d.file.display().to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                    .to_lowercase();
                    if !hay.contains(&needle) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn view_table(&self, inv: &Inventory) -> Element<'_, Message> {
        let s = &self.strings;
        let hosts = self.hosts_with_config();
        if inv.servers.is_empty() {
            return container(text(
                s.t("table.none", &[("hosts", &inv.hosts.len().to_string())]),
            ))
            .padding(20)
            .into();
        }
        let per_host = hosts.len() <= MAX_HOST_COLUMNS;
        let header_cell =
            |label: String, w: Length| text(label).size(12).style(text::secondary).width(w);
        let mut header = row![
            Space::with_width(14),
            header_cell(s.t("table.server", &[]), Length::FillPortion(5)),
            header_cell(s.t("table.transport", &[]), Length::Fixed(64.0)),
            header_cell(s.t("table.version", &[]), Length::Fixed(110.0)),
            header_cell(s.t("table.tokens", &[]), Length::Fixed(70.0)),
        ]
        .spacing(6)
        .padding([4, 6]);
        if per_host {
            for (_, name) in &hosts {
                header = header.push(header_cell(short_host(name), Length::FillPortion(2)));
            }
        } else {
            header = header.push(header_cell(s.t("table.hosts", &[]), Length::FillPortion(5)));
        }
        let mut rows = column![header].spacing(1);
        let visible = self.visible_servers(inv);
        if visible.is_empty() {
            rows = rows
                .push(container(text(s.t("table.empty", &[])).style(text::secondary)).padding(12));
        }
        for si in visible {
            let server = &inv.servers[si];
            let decls: Vec<&Declaration> = server
                .declarations
                .iter()
                .map(|&i| &inv.declarations[i])
                .collect();
            let has_finding = self.server_has_finding(inv, si);
            let dot: Element<'_, Message> = if has_finding {
                container(swatch(alert(self.dark), 8))
                    .width(Length::Fixed(14.0))
                    .into()
            } else {
                Space::with_width(14).into()
            };
            let transport = decls
                .first()
                .map(|d| transport_label(d.transport))
                .unwrap_or("");
            let versions = if server.versions.is_empty() {
                String::from("—")
            } else {
                server.versions.join(" / ")
            };
            let all_off = decls.iter().all(|d| !d.enabled);
            let (tokens_cell, tokens_color) = match self.probes.get(&server.identity) {
                Some(r) if r.ok => (
                    format!("~{}", cost::thousands(r.estimated_tokens)),
                    if r.estimated_tokens > cost::SERVER_THRESHOLD {
                        alert(self.dark)
                    } else {
                        self.fg_color()
                    },
                ),
                Some(_) => ("✗".to_string(), alert(self.dark)),
                None => (String::new(), self.fg_color()),
            };
            let mut r = row![
                dot,
                text(
                    server
                        .identity
                        .replace(&self.env.home.display().to_string(), "~")
                )
                .size(12)
                .font(iced::Font::MONOSPACE)
                .width(Length::FillPortion(5))
                .style(if all_off {
                    text::secondary
                } else {
                    text::default
                }),
                text(transport).size(12).width(Length::Fixed(64.0)),
                text(versions).size(12).width(Length::Fixed(110.0)),
                text(tokens_cell)
                    .size(12)
                    .font(iced::Font::MONOSPACE)
                    .color(tokens_color)
                    .width(Length::Fixed(70.0)),
            ]
            .spacing(6)
            .padding([5, 6])
            .align_y(iced::Alignment::Center);
            if per_host {
                for (id, _) in &hosts {
                    let cell = decls.iter().find(|d| d.hosts.contains(id)).map(|d| {
                        if !d.enabled {
                            s.t("cell.off", &[])
                        } else if let Some(v) = &d.version {
                            format!("@{v}")
                        } else {
                            "✓".to_string()
                        }
                    });
                    r = r.push(
                        text(cell.unwrap_or_default())
                            .size(12)
                            .width(Length::FillPortion(2)),
                    );
                }
            } else {
                let mut names: Vec<String> = Vec::new();
                for d in &decls {
                    for n in &d.host_names {
                        let n = short_host(n);
                        let n = if d.enabled {
                            n
                        } else {
                            format!("{n} ({})", s.t("cell.off", &[]))
                        };
                        if !names.contains(&n) {
                            names.push(n);
                        }
                    }
                }
                r = r.push(
                    text(names.join(", "))
                        .size(12)
                        .width(Length::FillPortion(5)),
                );
            }
            let selected = self.selected == Some(si);
            let line = container(r)
                .width(Length::Fill)
                .style(move |theme: &Theme| {
                    let p = theme.extended_palette();
                    container::Style {
                        background: Some(
                            if selected {
                                p.primary.weak.color
                            } else {
                                p.background.weak.color
                            }
                            .into(),
                        ),
                        text_color: if selected {
                            Some(p.primary.weak.text)
                        } else {
                            None
                        },
                        border: iced::Border::default().rounded(3),
                        ..container::Style::default()
                    }
                });
            rows = rows.push(mouse_area(line).on_press(Message::Select(si)));
        }
        card(scrollable(rows).height(Length::Fill).into())
            .height(Length::Fill)
            .into()
    }

    fn view_side(&self, inv: &Inventory) -> Element<'_, Message> {
        let s = &self.strings;
        let head = column![
            text(s.t("panel.title", &[])).size(16),
            text(s.t("panel.subtitle", &[]))
                .size(11)
                .style(text::secondary),
        ]
        .spacing(2);
        let detail: Element<'_, Message> = match (&self.pending_probe, self.selected) {
            (Some(list), _) => self.view_probe_confirm(inv, list),
            (None, Some(si)) if si < inv.servers.len() => self.view_detail(inv, si),
            _ => container(
                text(s.t("detail.pick", &[]))
                    .size(12)
                    .style(text::secondary),
            )
            .padding([8, 0])
            .into(),
        };
        card(
            column![
                head,
                scrollable(detail).height(Length::FillPortion(3)),
                horizontal_rule(1),
                scrollable(self.view_findings(inv)).height(Length::FillPortion(2)),
            ]
            .spacing(10)
            .into(),
        )
        .height(Length::Fill)
        .into()
    }

    fn view_detail(&self, inv: &Inventory, si: usize) -> Element<'_, Message> {
        let s = &self.strings;
        let server = &inv.servers[si];
        let mut col = column![text(
            server
                .identity
                .replace(&self.env.home.display().to_string(), "~")
        )
        .size(18)]
        .spacing(10);
        col = col.push(
            text(s.t("detail.declared_in", &[]))
                .size(12)
                .style(text::secondary),
        );
        for &di in &server.declarations {
            let d = &inv.declarations[di];
            let mut tags: Vec<String> = Vec::new();
            if !d.enabled {
                tags.push(s.t("detail.disabled", &[]));
            }
            if d.trusted {
                tags.push(s.t("detail.trusted", &[]));
            }
            if d.legacy {
                tags.push(s.t("detail.legacy", &[]));
            }
            if d.source_kind == SourceKind::Observed {
                tags.push(s.t("detail.observed", &[]));
            }
            if let Some(v) = &d.version {
                tags.push(format!("@{v}"));
            }
            let file_line = match d.line {
                Some(l) => format!("{}:{l}", short(&d.file, &self.env.home)),
                None => short(&d.file, &self.env.home),
            };
            let launch = match (&d.command, &d.url) {
                (Some(c), _) => {
                    let args: Vec<&str> = d.args.iter().map(|a| a.shown.as_str()).collect();
                    format!("{c} {}", args.join(" ")).trim_end().to_string()
                }
                (None, Some(u)) => u.shown.clone(),
                (None, None) => String::from("—"),
            };
            let mut block = column![
                row![
                    text(format!("{} · as `{}`", d.host_names.join(" + "), d.name)).size(13),
                    horizontal_space(),
                    text(tags.join(" · ")).size(11).style(text::secondary),
                ]
                .spacing(6),
                row![
                    text(file_line)
                        .size(12)
                        .style(text::secondary)
                        .width(Length::Fill),
                    small_button(s.t("detail.open", &[]), Message::OpenFile(d.file.clone())),
                    small_button(
                        s.t("detail.copy_path", &[]),
                        Message::Copy(d.file.display().to_string())
                    ),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
                row![
                    text(launch.clone())
                        .size(11)
                        .font(iced::Font::MONOSPACE)
                        .width(Length::Fill),
                    small_button(
                        s.t("detail.copy_command", &[]),
                        Message::Copy(launch.clone())
                    ),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(4);
            if let Some(p) = &d.project {
                block = block.push(
                    text(s.t("detail.project", &[("path", &short(p, &self.env.home))]))
                        .size(11)
                        .style(text::secondary),
                );
            }
            for (label, pairs) in [("detail.env", &d.env), ("detail.headers", &d.headers)] {
                if !pairs.is_empty() {
                    let masked = pairs
                        .iter()
                        .any(|(_, v)| v.kind == mcproster_core::secrets::ValueKind::Secret);
                    let title = if masked {
                        format!("{} ({})", s.t(label, &[]), s.t("detail.masked", &[]))
                    } else {
                        s.t(label, &[])
                    };
                    let mut kv = column![text(title).size(11).style(text::secondary)].spacing(2);
                    for (k, v) in pairs {
                        kv = kv.push(
                            text(format!("{k} = {}", v.shown))
                                .size(11)
                                .font(iced::Font::MONOSPACE),
                        );
                    }
                    block = block.push(kv);
                }
            }
            col = col.push(inner(block.into()));
        }
        // Probe
        col = col.push(
            text(s.t("probe.title", &[]))
                .size(12)
                .style(text::secondary),
        );
        let stdio = server
            .declarations
            .iter()
            .any(|&i| inv.declarations[i].transport == Transport::Stdio);
        if !stdio {
            col = col.push(text(s.t("probe.remote", &[])).size(12));
        } else {
            let result = self.probes.get(&server.identity);
            let line: Element<'_, Message> = match result {
                Some(r) if r.ok => text(format!(
                    "{} — {}",
                    s.t(
                        "probe.ok",
                        &[
                            ("date", r.at.get(..10).unwrap_or("")),
                            ("name", r.server_name.as_deref().unwrap_or("server")),
                            ("ms", &r.duration_ms.to_string()),
                            ("proto", r.protocol_version.as_deref().unwrap_or("?")),
                            ("n", &r.tools.len().to_string()),
                        ],
                    ),
                    s.t(
                        "probe.tokens",
                        &[("n", &cost::thousands(r.estimated_tokens))]
                    )
                ))
                .size(12)
                .color(if r.estimated_tokens > cost::SERVER_THRESHOLD {
                    alert(self.dark)
                } else {
                    self.fg_color()
                })
                .into(),
                Some(r) => text(s.t(
                    "probe.failed",
                    &[
                        ("date", r.at.get(..10).unwrap_or("")),
                        ("error", r.error.as_deref().unwrap_or("")),
                    ],
                ))
                .size(12)
                .color(alert(self.dark))
                .into(),
                None => text(s.t("probe.none", &[]))
                    .size(12)
                    .style(text::secondary)
                    .into(),
            };
            let testing = self.probing.as_deref() == Some(server.identity.as_str());
            let test = if testing {
                button(text(s.t("probe.testing", &[])).size(12)).padding([3, 10])
            } else {
                button(text(s.t("probe.test", &[])).size(12))
                    .padding([3, 10])
                    .on_press(Message::ProbeRequest(si))
            };
            let mut probe_row = row![line, horizontal_space(), test.style(button::primary)]
                .spacing(6)
                .align_y(iced::Alignment::Center);
            if let Some(r) = result.filter(|r| r.ok && !r.tools.is_empty()) {
                let open = self.open_tools.contains(&server.identity);
                probe_row = probe_row.push(small_button(
                    s.t(
                        if open {
                            "probe.hide_tools"
                        } else {
                            "probe.show_tools"
                        },
                        &[],
                    ),
                    Message::ToggleTools(server.identity.clone()),
                ));
                col = col.push(probe_row);
                if open {
                    let mut tools = column![].spacing(2);
                    for t in &r.tools {
                        tools = tools.push(
                            row![
                                text(t.name.clone()).size(12).width(Length::Fixed(200.0)),
                                text(format!("~{}", cost::thousands(t.tokens)))
                                    .size(11)
                                    .font(iced::Font::MONOSPACE)
                                    .width(Length::Fixed(70.0)),
                                text(t.description.clone().unwrap_or_default())
                                    .size(11)
                                    .style(text::secondary),
                            ]
                            .spacing(8),
                        );
                    }
                    col = col.push(inner(tools.into()));
                }
            } else {
                col = col.push(probe_row);
            }
        }
        container(col).padding([4, 0]).width(Length::Fill).into()
    }

    fn view_probe_confirm(&self, inv: &Inventory, list: &[usize]) -> Element<'_, Message> {
        let s = &self.strings;
        let home = self.env.home.display().to_string();
        let mut lines = column![].spacing(2);
        let mut hosts: Vec<String> = Vec::new();
        for &si in list {
            let Some(d) = inv.servers[si]
                .declarations
                .iter()
                .map(|&i| &inv.declarations[i])
                .find(|d| d.enabled && d.transport == Transport::Stdio)
            else {
                continue;
            };
            let args: Vec<&str> = d.args.iter().map(|a| a.shown.as_str()).collect();
            let cmd = format!(
                "{} {}",
                d.command.clone().unwrap_or_default(),
                args.join(" ")
            )
            .trim_end()
            .replace(&home, "~");
            let host = d.host_names.join(" + ");
            if !hosts.contains(&host) {
                hosts.push(host.clone());
            }
            lines = lines.push(
                row![
                    text(host)
                        .size(11)
                        .style(text::secondary)
                        .width(Length::Fixed(170.0)),
                    text(cmd).size(11).font(iced::Font::MONOSPACE),
                ]
                .spacing(8),
            );
        }
        let title = if list.len() > 1 {
            s.t("probe.confirm_all_title", &[("n", &list.len().to_string())])
        } else {
            s.t("probe.confirm_title", &[])
        };
        container(
            column![
                text(title).size(16),
                text(s.t("probe.confirm_intro", &[("hosts", &hosts.join(", "))])).size(13),
                inner(lines.into()),
                text(s.t("probe.confirm_outro", &[]))
                    .size(12)
                    .style(text::secondary),
                checkbox(s.t("probe.confirm_ask", &[]), self.dont_ask)
                    .on_toggle(Message::ProbeAskToggle)
                    .size(16)
                    .text_size(12),
                row![
                    button(text(s.t("probe.confirm_go", &[])).size(13))
                        .style(button::primary)
                        .on_press(Message::ProbeConfirm),
                    button(text(s.t("probe.confirm_cancel", &[])).size(13))
                        .style(button::secondary)
                        .on_press(Message::ProbeCancel),
                ]
                .spacing(8),
            ]
            .spacing(12),
        )
        .padding(12)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .into()
    }

    fn view_findings(&self, inv: &Inventory) -> Element<'_, Message> {
        let s = &self.strings;
        let report = cost::report(inv, &self.probes);
        let mut col = column![].spacing(6);
        // Context cost per host, when something has been probed.
        if report.hosts.iter().any(|h| h.probed > 0) {
            col = col.push(text(s.t("cost.title", &[])).size(12).style(text::secondary));
            let mut lines = column![].spacing(2);
            for h in report
                .hosts
                .iter()
                .filter(|h| h.probed + h.unprobed + h.failed > 0)
            {
                let mut notes: Vec<String> = Vec::new();
                if h.unprobed > 0 {
                    notes.push(s.t("cost.unprobed", &[("n", &h.unprobed.to_string())]));
                }
                if h.failed > 0 {
                    notes.push(s.t("cost.failed", &[("n", &h.failed.to_string())]));
                }
                if h.may_defer {
                    notes.push(s.t("cost.defer", &[]));
                }
                let heavy = h.tokens > cost::HOST_THRESHOLD;
                lines = lines.push(
                    row![
                        text(h.host_name.clone())
                            .size(12)
                            .width(Length::Fixed(190.0)),
                        text(format!("~{}", cost::thousands(h.tokens)))
                            .size(12)
                            .font(iced::Font::MONOSPACE)
                            .color(if heavy {
                                alert(self.dark)
                            } else {
                                good(self.dark)
                            })
                            .width(Length::Fixed(80.0)),
                        text(notes.join(" · ")).size(11).style(text::secondary),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                );
            }
            col = col.push(inner(lines.into()));
        }
        col = col.push(
            text(s.t("findings.title", &[]))
                .size(12)
                .style(text::secondary),
        );
        let shown: Vec<(usize, &mcproster_core::model::Finding)> = inv
            .findings
            .iter()
            .chain(report.findings.iter())
            .enumerate()
            .filter(|(_, f)| self.show_off || f.kind != FindingKind::Disabled)
            .collect();
        if shown.is_empty() {
            col = col.push(
                row![
                    swatch(good(self.dark), 8),
                    text(s.t("findings.none", &[])).size(12)
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            );
        }
        let home = self.env.home.display().to_string();
        let n_inv = inv.findings.len();
        for (fi, f) in shown {
            let key = format!("finding.{}", kind_slug(f.kind));
            let color = match f.kind {
                FindingKind::PlainSecret
                | FindingKind::WideAccess
                | FindingKind::ConfirmationsSkipped
                | FindingKind::CommandNotFound
                | FindingKind::VersionMismatch
                | FindingKind::Unreadable
                | FindingKind::ContextCost => alert(self.dark),
                FindingKind::Duplicate | FindingKind::Disabled | FindingKind::PackageNotCached => {
                    neutral_pill(self.dark)
                }
            };
            let mut block = column![
                pill(s.t(&key, &[]), color),
                text(f.message.replace(&home, "~")).size(12),
            ]
            .spacing(4);
            let acts = actions::actions_for(&self.rules, inv, f);
            if !acts.is_empty() {
                let mut a = column![text(s.t("action.title", &[]))
                    .size(11)
                    .style(text::secondary)]
                .spacing(2);
                for act in acts {
                    let (label, copy) = match act {
                        Action::HostCommand { command, .. } => {
                            (s.t("action.cli", &[("cmd", &command)]), Some(command))
                        }
                        Action::EditInHost { host } => {
                            (s.t("action.edit", &[("host", &host)]), None)
                        }
                        Action::OpenFile => (s.t("action.open", &[]), None),
                        Action::ReduceContext { host, may_defer } => (
                            s.t(
                                if may_defer {
                                    "action.reduce_defer"
                                } else {
                                    "action.reduce"
                                },
                                &[("host", &host)],
                            ),
                            None,
                        ),
                    };
                    let mut r = row![text(label).size(11).width(Length::Fill)]
                        .spacing(6)
                        .align_y(iced::Alignment::Center);
                    if let Some(c) = copy {
                        r = r.push(small_button(
                            s.t("detail.copy_command", &[]),
                            Message::Copy(c),
                        ));
                    }
                    a = a.push(r);
                }
                block = block.push(a);
            }
            let selected = f
                .declarations
                .first()
                .and_then(|&d| {
                    self.selected
                        .map(|si| inv.servers[si].declarations.contains(&d))
                })
                .unwrap_or(false);
            let c = container(block)
                .padding(iced::Padding {
                    top: 6.0,
                    right: 10.0,
                    bottom: 6.0,
                    left: 10.0,
                })
                .width(Length::Fill)
                .style(move |theme: &Theme| {
                    let p = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(if selected {
                            p.primary.weak.color
                        } else {
                            p.background.base.color
                        })),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });
            // Cost findings come after the inventory ones: select by their first declaration.
            let msg = if fi < n_inv {
                Message::SelectFinding(fi)
            } else {
                match f.declarations.first().and_then(|&d| {
                    inv.servers
                        .iter()
                        .position(|sv| sv.declarations.contains(&d))
                }) {
                    Some(si) => Message::Select(si),
                    None => Message::SelectFinding(0),
                }
            };
            col = col.push(mouse_area(c).on_press(msg));
        }
        container(col).padding([4, 0]).into()
    }
}

/// Cleanator's palette: green for "fine", orange for "look at this", grey for information.
fn hex(rgb: u32) -> Color {
    Color::from_rgb8(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    )
}

fn good(dark: bool) -> Color {
    if dark {
        hex(0x2fa32f)
    } else {
        hex(0x008300)
    }
}

fn alert(dark: bool) -> Color {
    if dark {
        hex(0xd95926)
    } else {
        hex(0xeb6834)
    }
}

fn neutral_pill(_dark: bool) -> Color {
    hex(0x7a869a)
}

fn card<'a>(content: Element<'a, Message>) -> container::Container<'a, Message> {
    container(content).padding(14).style(|theme: &Theme| {
        let p = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(p.background.weak.color)),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: p.background.strong.color,
            },
            ..Default::default()
        }
    })
}

/// A detail block inside a card: the page background, rounded, like Cleanator's details.
fn inner<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .padding(iced::Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 10.0,
        })
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn swatch<'a>(color: Color, size: u16) -> Element<'a, Message> {
    container(Space::new(size, size))
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: (size as f32 / 3.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn pill<'a>(label: String, color: Color) -> Element<'a, Message> {
    container(text(label).size(11).color(Color::WHITE))
        .padding([2, 8])
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn small_button(label: String, msg: Message) -> Element<'static, Message> {
    button(text(label).size(11))
        .padding([2, 7])
        .style(button::secondary)
        .on_press(msg)
        .into()
}

fn load(rules: Arc<Rules>, env: Env) -> Task<Message> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let inv = inventory::run(&rules, &env);
        let _ = tx.send(Arc::new(inv));
    });
    Task::perform(
        async move {
            rx.await
                .map_err(|_| "the inventory thread stopped unexpectedly".to_string())
        },
        Message::Loaded,
    )
}

fn short(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

/// A column header that fits: "Claude Desktop (extensions, .mcpb)" → "Claude Desktop (ext.)".
fn short_host(name: &str) -> String {
    name.replace(" (extensions, .mcpb)", " (ext.)")
        .replace("Visual Studio Code (Copilot)", "VS Code")
        .replace("Windsurf (Cascade)", "Windsurf")
        .replace("GitHub Copilot CLI", "Copilot CLI")
}

fn transport_label(t: Transport) -> &'static str {
    match t {
        Transport::Stdio => "stdio",
        Transport::Http => "http",
        Transport::Sse => "sse",
        Transport::Ws => "ws",
        Transport::Unknown => "?",
    }
}

fn kind_slug(k: FindingKind) -> &'static str {
    match k {
        FindingKind::PlainSecret => "plain-secret",
        FindingKind::WideAccess => "wide-access",
        FindingKind::ConfirmationsSkipped => "confirmations-skipped",
        FindingKind::CommandNotFound => "command-not-found",
        FindingKind::VersionMismatch => "version-mismatch",
        FindingKind::Duplicate => "duplicate",
        FindingKind::Unreadable => "unreadable",
        FindingKind::ContextCost => "context-cost",
        FindingKind::PackageNotCached => "package-not-cached",
        FindingKind::Disabled => "disabled",
    }
}
