use serenity::{
    all::ButtonStyle,
    builder::{CreateActionRow, CreateButton},
};

pub fn create_music_buttons() -> Vec<CreateActionRow> {
    let clear_button = CreateButton::new("clear")
        .label("📋 Clear")
        .style(ButtonStyle::Danger);

    let resume_button = CreateButton::new("resume")
        .label("▶️ Resume")
        .style(ButtonStyle::Success);

    let pause_button = CreateButton::new("pause")
        .label("⏸️ Pause")
        .style(ButtonStyle::Primary);

    let skip_button = CreateButton::new("skip")
        .label("⏭️ Skip")
        .style(ButtonStyle::Primary);

    let loop_button = CreateButton::new("loop")
        .label("🔄 Loop")
        .style(ButtonStyle::Primary);

    let row = CreateActionRow::Buttons(vec![
        clear_button,
        resume_button,
        pause_button,
        skip_button,
        loop_button,
    ]);

    vec![row]
}
