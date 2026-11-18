//! Object Viewer do Agral CAD: uma janela própria (painel destacado) que mostra os objetos selecionados em 3D,
//! com sombra, para girar e olhar (pedido do JP, 18/11). O desenho é feito aqui mesmo (canvas), com a câmera
//! guardada no estado do widget: arrastar gira, botão do meio ou direito arrasta a vista, rodinha aproxima,
//! duplo clique volta à vista inicial (isométrica sudeste). Não mexe no desenho.

use crate::app::Message;
use crate::ui::dock::PanelId;
use iced::mouse::{self, Button, Cursor, ScrollDelta};
use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Program, Stroke};
use iced::widget::{column, container, text};
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Theme};
use std::sync::Arc;

/// O que o viewer mostra: triângulos com cor (superfícies) e linhas 3D com cor.
#[derive(Debug, Default)]
pub struct Cena {
    pub titulo: String,
    pub triangulos: Vec<([[f32; 3]; 3], [u8; 3])>,
    pub linhas: Vec<(Vec<[f32; 3]>, [u8; 3])>,
    pub centro: [f32; 3],
    pub raio: f32,
}

impl Cena {
    /// Centro e raio a partir do que tem.
    pub fn enquadrar(&mut self) {
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        let mut ver = |p: &[f32; 3]| {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        };
        self.triangulos.iter().flat_map(|t| t.0.iter()).for_each(&mut ver);
        self.linhas.iter().flat_map(|l| l.0.iter()).for_each(&mut ver);
        if lo[0].is_finite() {
            self.centro = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
            self.raio = ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2) + (hi[2] - lo[2]).powi(2)).sqrt().max(1e-3) / 2.0;
        } else {
            self.raio = 1.0;
        }
    }
}

const YAW0: f32 = -std::f32::consts::FRAC_PI_4;
const PITCH0: f32 = 0.95;

#[derive(Debug)]
pub struct Camera {
    yaw: f32,
    pitch: f32,
    zoom: f32,
    pan: (f32, f32),
    arrasto: Option<(Point, bool)>,
    ultimo_clique: Option<std::time::Instant>,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { yaw: YAW0, pitch: PITCH0, zoom: 1.0, pan: (0.0, 0.0), arrasto: None, ultimo_clique: None }
    }
}

struct Visor(Arc<Cena>);

impl Visor {
    /// Ponto do mundo para a tela: (x, y) em pixels e a profundidade (maior = mais perto).
    fn projetar(&self, c: &Camera, b: Rectangle, p: &[f32; 3]) -> (f32, f32, f32) {
        let s = self.0.raio.max(1e-3);
        let (dx, dy, dz) = (p[0] - self.0.centro[0], p[1] - self.0.centro[1], p[2] - self.0.centro[2]);
        let (sy, cy) = c.yaw.sin_cos();
        let xr = cy * dx - sy * dy;
        let yr = sy * dx + cy * dy;
        let (sp, cp) = c.pitch.sin_cos();
        let tela_y = yr * cp + dz * sp;
        let prof = -yr * sp + dz * cp;
        let k = b.width.min(b.height) * 0.45 / s * c.zoom;
        (b.width / 2.0 + xr * k + c.pan.0, b.height / 2.0 - tela_y * k + c.pan.1, prof)
    }
}

fn cor(c: [u8; 3], luz: f32) -> Color {
    Color::from_rgb8(
        (c[0] as f32 * luz).clamp(0.0, 255.0) as u8,
        (c[1] as f32 * luz).clamp(0.0, 255.0) as u8,
        (c[2] as f32 * luz).clamp(0.0, 255.0) as u8,
    )
}

impl Program<Message> for Visor {
    type State = Camera;

    fn update(&self, c: &mut Camera, event: &Event, b: Rectangle, cursor: Cursor) -> Option<canvas::Action<Message>> {
        let pos = cursor.position_in(b);
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(botao)) => {
                let p = pos?;
                if *botao == Button::Left {
                    let agora = std::time::Instant::now();
                    if c.ultimo_clique.is_some_and(|t| agora.duration_since(t).as_millis() < 350) {
                        *c = Camera::default();
                        return Some(canvas::Action::request_redraw().and_capture());
                    }
                    c.ultimo_clique = Some(agora);
                }
                c.arrasto = Some((p, *botao != Button::Left));
                Some(canvas::Action::capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(_)) => {
                c.arrasto = None;
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let (inicio, mover) = c.arrasto?;
                let p = cursor.position_in(b).unwrap_or(inicio);
                let (dx, dy) = (p.x - inicio.x, p.y - inicio.y);
                if mover {
                    c.pan.0 += dx;
                    c.pan.1 += dy;
                } else {
                    c.yaw -= dx * 0.01;
                    c.pitch = (c.pitch + dy * 0.01).clamp(0.0, std::f32::consts::FRAC_PI_2);
                }
                c.arrasto = Some((p, mover));
                Some(canvas::Action::request_redraw().and_capture())
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                pos?;
                let d = match delta {
                    ScrollDelta::Lines { y, .. } => *y,
                    ScrollDelta::Pixels { y, .. } => *y / 40.0,
                };
                c.zoom = (c.zoom * 1.15f32.powf(d)).clamp(0.05, 200.0);
                Some(canvas::Action::request_redraw().and_capture())
            }
            _ => None,
        }
    }

    fn draw(&self, c: &Camera, renderer: &Renderer, theme: &Theme, b: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        let fundo = theme.palette().background.base.color;
        f.fill_rectangle(Point::ORIGIN, b.size(), fundo);
        // Luz fixa vinda de cima, do lado do observador.
        let (sy, cy) = c.yaw.sin_cos();
        let luz = {
            let l = [-0.35 * cy - 0.6 * sy, 0.35 * sy - 0.6 * cy, 0.75];
            let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
            [l[0] / n, l[1] / n, l[2] / n]
        };
        let mut tris: Vec<(f32, [Point; 3], Color)> = self
            .0
            .triangulos
            .iter()
            .map(|(t, rgb)| {
                let q = t.map(|p| self.projetar(c, b, &p));
                let (u, v) = (
                    [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]],
                    [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]],
                );
                let mut n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                if n[2] < 0.0 {
                    n = [-n[0], -n[1], -n[2]];
                }
                let m = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
                let i = 0.35 + 0.65 * ((n[0] * luz[0] + n[1] * luz[1] + n[2] * luz[2]) / m).max(0.0);
                ((q[0].2 + q[1].2 + q[2].2) / 3.0, q.map(|p| Point::new(p.0, p.1)), cor(*rgb, i))
            })
            .collect();
        // Do mais longe para o mais perto.
        tris.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, p, c) in &tris {
            let caminho = Path::new(|b| {
                b.move_to(p[0]);
                b.line_to(p[1]);
                b.line_to(p[2]);
                b.close();
            });
            f.fill(&caminho, *c);
        }
        for (l, rgb) in &self.0.linhas {
            if l.len() < 2 {
                continue;
            }
            let caminho = Path::new(|bp| {
                let q = self.projetar(c, b, &l[0]);
                bp.move_to(Point::new(q.0, q.1));
                for p in &l[1..] {
                    let q = self.projetar(c, b, p);
                    bp.line_to(Point::new(q.0, q.1));
                }
            });
            f.stroke(&caminho, Stroke::default().with_width(1.5).with_color(cor(*rgb, 1.0)));
        }
        vec![f.into_geometry()]
    }

    fn mouse_interaction(&self, c: &Camera, b: Rectangle, cursor: Cursor) -> mouse::Interaction {
        match (c.arrasto, cursor.is_over(b)) {
            (Some(_), _) => mouse::Interaction::Grabbing,
            (None, true) => mouse::Interaction::Grab,
            _ => mouse::Interaction::default(),
        }
    }
}

pub fn view<'a>(cena: Option<Arc<Cena>>, width: f32, chrome: crate::ui::dock::Chrome) -> Element<'a, Message> {
    let titulo = cena.as_ref().map_or("Object Viewer".to_string(), |c| format!("Object Viewer - {}", c.titulo));
    let title_bar = crate::ui::dock::title_bar(PanelId::ObjectViewer, titulo, chrome);
    let corpo: Element<'a, Message> = match cena {
        Some(c) => column![
            canvas::Canvas::new(Visor(c)).width(Fill).height(Fill),
            text("Drag: orbit   Middle/right drag: pan   Wheel: zoom   Double click: reset").size(11),
        ]
        .spacing(4)
        .into(),
        None => text("Select objects and run OBJECTVIEWER.").size(12).into(),
    };
    container(column![title_bar, container(corpo).padding(6).width(Fill).height(Fill)])
        .width(width)
        .height(Fill)
        .into()
}
