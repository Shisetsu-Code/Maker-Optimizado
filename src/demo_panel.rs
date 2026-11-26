//! Shared administrative demonstration. It operates exclusively on synthetic
//! preview records. There is no account creation, authentication, payment,
//! game outcome, real balance or external service behind these actions.
use crate::model::valid_text;
use crate::render::escape;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    #[default]
    Overview,
    Support,
    Partners,
    Team,
    Players,
    Requests,
    Records,
    Statistics,
    Alerts,
    Prizes,
    Flyers,
    Settings,
}
impl Section {
    pub const ALL: [Self; 12] = [
        Self::Overview,
        Self::Support,
        Self::Partners,
        Self::Team,
        Self::Players,
        Self::Requests,
        Self::Records,
        Self::Statistics,
        Self::Alerts,
        Self::Prizes,
        Self::Flyers,
        Self::Settings,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Support => "support",
            Self::Partners => "partners",
            Self::Team => "team",
            Self::Players => "players",
            Self::Requests => "requests",
            Self::Records => "records",
            Self::Statistics => "statistics",
            Self::Alerts => "alerts",
            Self::Prizes => "prizes",
            Self::Flyers => "flyers",
            Self::Settings => "settings",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "Inicio",
            Self::Support => "Soporte",
            Self::Partners => "Socios",
            Self::Team => "Equipo",
            Self::Players => "Jugadores",
            Self::Requests => "Pedidos",
            Self::Records => "Movimientos",
            Self::Statistics => "Estadísticas",
            Self::Alerts => "Alertas",
            Self::Prizes => "Premios",
            Self::Flyers => "Flyers",
            Self::Settings => "Ajustes",
        }
    }
    pub fn summary(self) -> &'static str {
        match self {
            Self::Overview => "Monitoreo y tareas de ejemplo",
            Self::Support => "Conversaciones simuladas",
            Self::Partners => "Organización de paneles de muestra",
            Self::Team => "Vista de roles y estados ficticios",
            Self::Players => "Listado de perfiles ficticios",
            Self::Requests => "Bandeja de revisión sin operaciones reales",
            Self::Records => "Historial de acciones de la demostración",
            Self::Statistics => "Indicadores exclusivamente ilustrativos",
            Self::Alerts => "Alertas ficticias para revisar el diseño",
            Self::Prizes => "Vista de estados de premios simulados",
            Self::Flyers => "Organización visual de material gráfico",
            Self::Settings => "Preferencias locales de la vista de muestra",
        }
    }
    fn searchable(self) -> bool {
        !matches!(self, Self::Overview | Self::Settings | Self::Statistics)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Open,
    Closed,
    Pending,
    Reviewed,
    Active,
    Paused,
}
impl Status {
    fn label(self) -> &'static str {
        match self {
            Self::Open => "Abierta",
            Self::Closed => "Cerrada",
            Self::Pending => "Pendiente",
            Self::Reviewed => "Revisada",
            Self::Active => "Activo",
            Self::Paused => "Pausado",
        }
    }
    fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Pending => "pending",
            Self::Reviewed => "reviewed",
            Self::Active => "active",
            Self::Paused => "paused",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
struct Row {
    id: String,
    section: Section,
    title: String,
    detail: String,
    status: Status,
    notes: Vec<String>,
}
impl Row {
    fn new(id: &str, section: Section, title: &str, detail: &str, status: Status) -> Self {
        Self {
            id: id.into(),
            section,
            title: title.into(),
            detail: detail.into(),
            status,
            notes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Navigate { section: Section },
    Search { query: String },
    Filter { filter: String },
    Select { id: String },
    Note { id: String, text: String },
    Toggle { id: String },
    Review { id: String },
    Maintenance { enabled: bool },
    Reset,
}

#[derive(Debug, Clone, Serialize)]
pub struct Panel {
    section: Section,
    query: String,
    filter: String,
    selected: Option<String>,
    maintenance: bool,
    rows: Vec<Row>,
    log: Vec<String>,
}
impl Default for Panel {
    fn default() -> Self {
        use Section::*;
        use Status::*;
        Self {
            section: Overview,
            query: String::new(),
            filter: String::new(),
            selected: None,
            maintenance: false,
            rows: vec![
                Row::new(
                    "conversation-1",
                    Support,
                    "Consulta de ejemplo",
                    "¿Cómo cambio los datos de mi perfil de muestra?",
                    Open,
                ),
                Row::new(
                    "conversation-2",
                    Support,
                    "Ayuda de navegación",
                    "Consulta resuelta en la demostración.",
                    Closed,
                ),
                Row::new(
                    "partner-1",
                    Partners,
                    "Panel Norte",
                    "Organización de ejemplo",
                    Active,
                ),
                Row::new(
                    "partner-2",
                    Partners,
                    "Panel Centro",
                    "Organización de ejemplo",
                    Paused,
                ),
                Row::new(
                    "team-1",
                    Team,
                    "Persona A",
                    "Administrador · ejemplo",
                    Active,
                ),
                Row::new("team-2", Team, "Persona B", "Atención · ejemplo", Active),
                Row::new(
                    "player-1",
                    Players,
                    "Perfil Alba",
                    "Perfil ficticio · sin cuenta real",
                    Active,
                ),
                Row::new(
                    "player-2",
                    Players,
                    "Perfil Bruno",
                    "Perfil ficticio · sin cuenta real",
                    Paused,
                ),
                Row::new(
                    "request-1",
                    Requests,
                    "Solicitud de muestra",
                    "Entrada de revisión · sin dinero",
                    Pending,
                ),
                Row::new(
                    "request-2",
                    Requests,
                    "Solicitud archivada",
                    "Entrada de revisión de ejemplo",
                    Reviewed,
                ),
                Row::new(
                    "alert-1",
                    Alerts,
                    "Aviso ilustrativo",
                    "Comprobar el estado visual del aviso.",
                    Pending,
                ),
                Row::new(
                    "prize-1",
                    Prizes,
                    "Premio ilustrativo",
                    "No acredita ni modifica ningún saldo.",
                    Pending,
                ),
                Row::new(
                    "folder-1",
                    Flyers,
                    "Bienvenida",
                    "Colección gráfica de muestra",
                    Active,
                ),
                Row::new(
                    "folder-2",
                    Flyers,
                    "Novedades",
                    "Colección gráfica de muestra",
                    Active,
                ),
            ],
            log: vec!["Se inició la vista de ejemplo.".into()],
        }
    }
}

impl Panel {
    pub fn maintenance(&self) -> bool {
        self.maintenance
    }
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"section":self.section,"maintenance":self.maintenance})
    }
    fn row_mut(&mut self, id: &str) -> Result<&mut Row, String> {
        self.rows
            .iter_mut()
            .find(|r| r.id == id && r.section == self.section)
            .ok_or_else(|| "Registro de ejemplo no encontrado.".into())
    }
    fn row_visible(&self, row: &Row) -> bool {
        row.section == self.section
            && (self.filter.is_empty() || self.filter == row.status.id())
            && format!("{} {}", row.title, row.detail)
                .to_lowercase()
                .contains(&self.query.trim().to_lowercase())
    }

    fn clear_hidden_selection(&mut self) {
        if self.selected.as_ref().is_some_and(|id| {
            !self
                .rows
                .iter()
                .any(|row| &row.id == id && self.row_visible(row))
        }) {
            self.selected = None;
        }
    }

    fn filter_options(&self) -> &'static [Status] {
        match self.section {
            Section::Support => &[Status::Open, Status::Closed],
            Section::Requests | Section::Alerts | Section::Prizes => {
                &[Status::Pending, Status::Reviewed]
            }
            Section::Partners | Section::Team | Section::Players | Section::Flyers => {
                &[Status::Active, Status::Paused]
            }
            _ => &[],
        }
    }

    pub fn apply(&mut self, action: Action) -> Result<(), String> {
        let mut next = self.clone();
        let event = match action {
            Action::Navigate { section } => {
                next.section = section;
                next.query.clear();
                next.filter.clear();
                next.selected = None;
                None
            }
            Action::Search { query } => {
                if !valid_text(&query, 120) {
                    return Err("Búsqueda inválida.".into());
                }
                next.query = query;
                next.clear_hidden_selection();
                None
            }
            Action::Filter { filter } => {
                if !filter.is_empty() && !next.filter_options().iter().any(|s| s.id() == filter) {
                    return Err("Filtro inválido.".into());
                }
                next.filter = filter;
                next.clear_hidden_selection();
                None
            }
            Action::Select { id } => {
                if !next.rows.iter().any(|r| r.id == id && next.row_visible(r)) {
                    return Err("Registro de ejemplo no encontrado.".into());
                }
                next.selected = Some(id);
                None
            }
            Action::Note { id, text } => {
                if text.trim().is_empty() || !valid_text(&text, 300) {
                    return Err("Escribí un texto de hasta 300 caracteres.".into());
                }
                let row = next.row_mut(&id)?;
                if row.notes.len() >= 32 {
                    return Err("Límite de notas de demostración.".into());
                }
                row.notes.push(text.trim().into());
                Some(format!("Nota local: {}", row.title))
            }
            Action::Toggle { id } => {
                let row = next.row_mut(&id)?;
                row.status = match row.status {
                    Status::Open => Status::Closed,
                    Status::Closed => Status::Open,
                    Status::Active => Status::Paused,
                    Status::Paused => Status::Active,
                    _ => return Err("Este estado no admite alternancia.".into()),
                };
                Some(format!(
                    "Estado de muestra: {} · {}",
                    row.title,
                    row.status.label()
                ))
            }
            Action::Review { id } => {
                let row = next.row_mut(&id)?;
                if row.status != Status::Pending {
                    return Err("La entrada ya fue revisada.".into());
                }
                row.status = Status::Reviewed;
                Some(format!("Revisión visual: {}", row.title))
            }
            Action::Maintenance { enabled } => {
                if next.section != Section::Settings {
                    return Err("Abrí Ajustes para cambiar la vista.".into());
                }
                next.maintenance = enabled;
                Some(if enabled {
                    "Vista de mantenimiento activada.".into()
                } else {
                    "Vista de mantenimiento desactivada.".into()
                })
            }
            Action::Reset => {
                next = Self::default();
                None
            }
        };
        if let Some(event) = event {
            next.log.push(event);
            if next.log.len() > 128 {
                next.log.remove(0);
            }
        }
        next.clear_hidden_selection();
        *self = next;
        Ok(())
    }
    pub fn navigation(&self) -> String {
        let mut out = String::from(
            "<nav class=\"panel-navigation\" aria-label=\"Secciones del panel\"><span class=\"panel-nav-label\">PANEL DE MUESTRA</span>",
        );
        for section in Section::ALL {
            write!(out,"<button type=\"button\" data-panel-section=\"{}\" aria-pressed=\"{}\"><span class=\"panel-nav-indicator\" aria-hidden=\"true\"></span>{}</button>",section.id(),self.section==section,section.label()).unwrap();
        }
        out.push_str("</nav>");
        out
    }
    pub fn content(&self) -> String {
        let mut out = String::new();
        write!(out,"<div class=\"panel-content-heading\"><div><span class=\"eyebrow\">VISTA LOCAL · DEMO</span><h2>{}</h2><p>{}</p></div><span class=\"panel-status\">Datos ficticios</span></div>",self.section.label(),self.section.summary()).unwrap();
        match self.section {
            Section::Overview => {
                let pending = self
                    .rows
                    .iter()
                    .filter(|r| r.status == Status::Pending)
                    .count();
                let open = self
                    .rows
                    .iter()
                    .filter(|r| r.section == Section::Support && r.status == Status::Open)
                    .count();
                write!(out,"<div class=\"panel-dashboard\"><article><span>Monitoreo</span><strong>Local</strong><small>Sin conexión a proveedores</small></article><article><span>Conversaciones abiertas</span><strong>{open}</strong><small>Ejemplo</small></article><article><span>Revisiones pendientes</span><strong>{pending}</strong><small>Ejemplo</small></article><article><span>Estado de la vista</span><strong>{}</strong><small>Solo este navegador</small></article></div><h3>Lo que sigue</h3><div class=\"panel-list\">",if self.maintenance{"Mantenimiento"}else{"Disponible"}).unwrap();
                for (section, label) in [
                    (Section::Support, "Revisar conversaciones"),
                    (Section::Requests, "Revisar solicitudes"),
                    (Section::Players, "Explorar perfiles"),
                    (Section::Settings, "Configurar vista"),
                ] {
                    write!(out,"<button type=\"button\" class=\"panel-row\" data-panel-section=\"{}\"><span>{label}</span><span aria-hidden=\"true\">›</span></button>",section.id()).unwrap();
                }
                out.push_str("</div>");
            }
            Section::Settings => {
                write!(out,"<div class=\"panel-settings\"><h3>Vista de mantenimiento</h3><p>Permite comprobar la pantalla de mantenimiento del lobby. No modifica ningún sitio ni servicio.</p><button type=\"button\" class=\"action\" data-panel-maintenance=\"{}\">{}</button><h3>Datos de ejemplo</h3><p>Restaurá las conversaciones, estados y notas ficticias de esta sesión.</p><button type=\"button\" data-panel-reset=\"true\" class=\"panel-secondary\">Restablecer demostración</button></div>",!self.maintenance,if self.maintenance{"Quitar mantenimiento"}else{"Mostrar mantenimiento"}).unwrap();
            }
            Section::Records => {
                write!(out,"<label class=\"search-label\">Buscar<input data-panel-search type=\"search\" value=\"{}\" placeholder=\"Buscar en el historial\"></label><div class=\"panel-list\">",escape(&self.query)).unwrap();
                let query = self.query.trim().to_lowercase();
                let mut count = 0;
                for event in self
                    .log
                    .iter()
                    .rev()
                    .filter(|event| event.to_lowercase().contains(&query))
                {
                    count += 1;
                    write!(
                        out,
                        "<div class=\"panel-row\"><span>{}</span><small>Local</small></div>",
                        escape(event)
                    )
                    .unwrap();
                }
                if count == 0 {
                    out.push_str("<p class=\"panel-empty\">No hay movimientos de ejemplo que coincidan.</p>");
                }
                out.push_str("</div>");
            }
            Section::Statistics => {
                let profiles = self
                    .rows
                    .iter()
                    .filter(|r| r.section == Section::Players)
                    .count();
                let teams = self
                    .rows
                    .iter()
                    .filter(|r| r.section == Section::Team)
                    .count();
                let reviewed = self
                    .rows
                    .iter()
                    .filter(|r| r.status == Status::Reviewed)
                    .count();
                write!(out,"<div class=\"panel-dashboard\"><article><span>Perfiles de ejemplo</span><strong>{profiles}</strong></article><article><span>Equipo de ejemplo</span><strong>{teams}</strong></article><article><span>Revisiones</span><strong>{reviewed}</strong></article><article><span>Eventos locales</span><strong>{}</strong></article></div><p>No hay métricas de juego ni importes reales.</p>",self.log.len()).unwrap();
            }
            section if section.searchable() => {
                write!(out,"<div class=\"panel-filters\"><label class=\"search-label\">Buscar<input type=\"search\" data-panel-search value=\"{}\" placeholder=\"Buscar registro\"></label><label class=\"search-label\">Estado<select data-panel-filter><option value=\"\">Todos</option>",escape(&self.query)).unwrap();
                for status in self.filter_options() {
                    write!(
                        out,
                        "<option value=\"{}\"{}>{}</option>",
                        status.id(),
                        if self.filter == status.id() {
                            " selected"
                        } else {
                            ""
                        },
                        status.label()
                    )
                    .unwrap();
                }
                out.push_str("</select></label></div><div class=\"panel-list\">");
                let mut count = 0;
                for row in self.rows.iter().filter(|r| self.row_visible(r)) {
                    count += 1;
                    write!(out,"<button class=\"panel-row\" type=\"button\" data-panel-row=\"{}\" aria-pressed=\"{}\"><span><strong>{}</strong><small>{}</small></span><span class=\"panel-status\" data-state=\"{}\">{}</span></button>",escape(&row.id),self.selected.as_deref()==Some(&row.id),escape(&row.title),escape(&row.detail),row.status.id(),row.status.label()).unwrap();
                }
                if count == 0 {
                    out.push_str(
                        "<p class=\"panel-empty\">No hay registros de ejemplo que coincidan.</p>",
                    );
                }
                out.push_str("</div>");
                if let Some(row) = self.rows.iter().find(|r| {
                    Some(r.id.as_str()) == self.selected.as_deref() && self.row_visible(r)
                }) {
                    write!(
                        out,
                        "<div class=\"panel-detail\"><h3>{}</h3><p>{}</p>",
                        escape(&row.title),
                        escape(&row.detail)
                    )
                    .unwrap();
                    for note in &row.notes {
                        write!(out, "<p class=\"panel-note\">{}</p>", escape(note)).unwrap();
                    }
                    write!(out,"<label>Nota o respuesta de prueba<textarea maxlength=\"300\" data-panel-note=\"{}\" rows=\"3\" placeholder=\"Texto local de demostración\"></textarea></label><button type=\"button\" class=\"action\" data-panel-save-note=\"{}\">Guardar texto local</button>",escape(&row.id),escape(&row.id)).unwrap();
                    match row.status {
                        Status::Open | Status::Closed | Status::Active | Status::Paused => {
                            write!(out,"<button type=\"button\" class=\"panel-secondary\" data-panel-toggle=\"{}\">{}</button>",escape(&row.id),match row.status{Status::Open=>"Cerrar conversación",Status::Closed=>"Reabrir conversación",Status::Active=>"Pausar muestra",_=>"Activar muestra"}).unwrap();
                        }
                        Status::Pending => {
                            write!(out,"<button type=\"button\" class=\"panel-secondary\" data-panel-review=\"{}\">Marcar como revisado</button>",escape(&row.id)).unwrap();
                        }
                        Status::Reviewed => {}
                    }
                    out.push_str("</div>");
                }
            }
            _ => {}
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_and_navigation_are_isolated_and_validated() {
        let mut p = Panel::default();
        assert_eq!(p.navigation().matches("data-panel-section").count(), 12);
        p.apply(Action::Navigate {
            section: Section::Players,
        })
        .unwrap();
        assert!(
            p.apply(Action::Toggle {
                id: "conversation-1".into()
            })
            .is_err()
        );
        p.apply(Action::Select {
            id: "player-1".into(),
        })
        .unwrap();
        p.apply(Action::Note {
            id: "player-1".into(),
            text: "<script>test</script>".into(),
        })
        .unwrap();
        assert!(p.content().contains("&lt;script&gt;test&lt;/script&gt;"));
        assert!(!p.content().contains("<script>"));
        p.apply(Action::Toggle {
            id: "player-1".into(),
        })
        .unwrap();
        assert!(p.content().contains("Activar muestra"));
        p.apply(Action::Search {
            query: "no-match".into(),
        })
        .unwrap();
        assert!(p.content().contains("No hay registros"));
    }
    #[test]
    fn filters_clear_hidden_details_and_records_report_empty_results() {
        let mut p = Panel::default();
        p.apply(Action::Navigate {
            section: Section::Players,
        })
        .unwrap();
        p.apply(Action::Select {
            id: "player-1".into(),
        })
        .unwrap();
        assert!(p.content().contains("data-panel-note"));
        p.apply(Action::Filter {
            filter: "paused".into(),
        })
        .unwrap();
        assert!(!p.content().contains("data-panel-note"));
        assert!(
            p.apply(Action::Select {
                id: "player-1".into(),
            })
            .is_err()
        );
        assert!(
            p.apply(Action::Filter {
                filter: "open".into(),
            })
            .is_err()
        );
        p.apply(Action::Navigate {
            section: Section::Records,
        })
        .unwrap();
        p.apply(Action::Search {
            query: "no-match".into(),
        })
        .unwrap();
        assert!(p.content().contains("No hay movimientos"));
    }
    #[test]
    fn maintenance_is_preview_only_and_resettable() {
        let mut p = Panel::default();
        assert!(p.apply(Action::Maintenance { enabled: true }).is_err());
        p.apply(Action::Navigate {
            section: Section::Settings,
        })
        .unwrap();
        p.apply(Action::Maintenance { enabled: true }).unwrap();
        assert!(p.maintenance());
        p.apply(Action::Reset).unwrap();
        assert!(!p.maintenance());
    }
    #[test]
    fn reviewing_never_changes_a_balance_or_creates_accounts() {
        let mut p = Panel::default();
        p.apply(Action::Navigate {
            section: Section::Requests,
        })
        .unwrap();
        p.apply(Action::Review {
            id: "request-1".into(),
        })
        .unwrap();
        assert!(
            p.apply(Action::Review {
                id: "request-1".into()
            })
            .is_err()
        );
        p.apply(Action::Navigate {
            section: Section::Records,
        })
        .unwrap();
        assert!(
            p.content()
                .contains("Revisión visual: Solicitud de muestra")
        );
    }
}
