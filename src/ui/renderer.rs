use crate::app::App;
use crate::scripts::{Script, ScriptNaoOficial};
use ratatui::{
    layout::{Margin, Rect},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

// função que desenha a interface a cada "frame"
pub fn ui(frame: &mut Frame, app: &mut App) {
    let mut itens: Vec<ListItem> = Vec::new();
    let mut linha_visual_selecionada = 0;

    // conta quantos scripts de cada tipo temos
    let scripts_oficiais: Vec<_> = app
        .codigos
        .iter()
        .enumerate()
        .filter(|(_, c)| c.as_any().downcast_ref::<Script>().is_some())
        .collect();

    let scripts_auto: Vec<_> = app
        .codigos
        .iter()
        .enumerate()
        .filter(|(_, c)| c.as_any().downcast_ref::<ScriptNaoOficial>().is_some())
        .collect();

    // header de oficiais se tiver
    if !scripts_oficiais.is_empty() {
        itens.push(
            ListItem::new("═══ oficiais ═══").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        itens.push(ListItem::new("")); // espaço
    }

    // adiciona scripts oficiais
    for (idx, codigo) in &scripts_oficiais {
        let eh_selecionado = *idx == app.script_selecionado;

        // calcula a linha visual correspondente ao script selecionado
        if eh_selecionado {
            linha_visual_selecionada = itens.len();
        }

        // estilo do nome e descrição quando selecionado
        let nome_style = if eh_selecionado {
            Style::default()
                .bg(Color::LightMagenta)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let desc_style = if eh_selecionado {
            Style::default()
                .bg(Color::Rgb(200, 150, 200)) // magenta mais transparente
                .fg(Color::Black)
        } else {
            Style::default().fg(Color::Gray)
        };

        itens.push(ListItem::new(format!("  ☭ {}", codigo.nome())).style(nome_style));
        itens.push(ListItem::new(format!("     {}", codigo.descricao())).style(desc_style));
    }

    // header de auto-detectados se tiver
    if !scripts_auto.is_empty() {
        if !scripts_oficiais.is_empty() {
            itens.push(ListItem::new("")); // espaço entre seções
        }

        itens.push(
            ListItem::new("═══ perdidos no root ═══").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        itens.push(ListItem::new("")); // espaço
    }

    // adiciona scripts auto-detectados
    for (idx, codigo) in &scripts_auto {
        let eh_selecionado = *idx == app.script_selecionado;

        // calcula a linha visual correspondente ao script selecionado
        if eh_selecionado {
            linha_visual_selecionada = itens.len();
        }

        let nome_style = if eh_selecionado {
            Style::default()
                .bg(Color::LightMagenta)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let desc_style = if eh_selecionado {
            Style::default()
                .bg(Color::Rgb(200, 150, 200))
                .fg(Color::Black)
        } else {
            Style::default().fg(Color::Gray)
        };

        itens.push(ListItem::new(format!("  ⚡︎ {}", codigo.nome())).style(nome_style));
        itens.push(ListItem::new(format!("     {}", codigo.descricao())).style(desc_style));
    }

    // se nao tiver nada, mostra mensagem
    if itens.is_empty() {
        itens.push(
            ListItem::new("   nenhum script encontrado :(").style(Style::default().fg(Color::Gray)),
        );
    }

    // atualiza o estado da UI com a linha visual selecionada
    app.ui_state.select(Some(linha_visual_selecionada));

    // cria a lista visual (sem highlight automático porque controlamos manualmente)
    let lista = List::new(itens)
        .block(Block::default().title("painel fodão").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    // renderiza a lista com estado pro scroll automático
    frame.render_stateful_widget(lista, frame.area(), &mut app.ui_state);

    // um rodape com instruções
    let instrucoes = Paragraph::new("use <enter> para executar | <q> para sair | ↑↓ para navegar")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);

    // cria uma area pro rodape
    let area_instrucoes = Rect {
        x: frame.area().x,
        y: frame.area().height.saturating_sub(1), // bota na ultima linha (kk la ele)
        width: frame.area().width,
        height: 1,
    };
    frame.render_widget(instrucoes, area_instrucoes);

    // renderiza qualquer overlay (terminal ou mensagem)
    if let Some(ov) = &mut app.overlay {
        let need_w = 98 + 2;
        let need_h = 36 + 2;
        let area = centered_rect(need_w, need_h, frame.area());
        let final_area = if area.width < need_w || area.height < need_h {
            frame.area()
        } else {
            area
        };
        ov.render(
            frame,
            final_area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
    }
}

// função auxiliar para centralizar um retângulo
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let cw = r.width.saturating_sub(width) / 2;
    let ch = r.height.saturating_sub(height) / 2;
    Rect {
        x: r.x + cw,
        y: r.y + ch,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}
