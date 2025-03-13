use std::num::NonZero;

use godot::builtin::Side;
use godot::classes::control::{LayoutPreset, SizeFlags};
use godot::classes::{
    window, Button, Control, EditorInterface, HBoxContainer, IPopupPanel, Label, MarginContainer,
    PopupPanel, ProgressBar, VBoxContainer,
};
use godot::global::maxf;
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(tool, base = PopupPanel, no_init, rename = ScProgressDialog)]
pub struct ProgressDialog {
    main: Gd<VBoxContainer>,
    progress: Gd<ProgressBar>,
    progress_state: Gd<Label>,
    task_progress: Gd<ProgressBar>,
    task_progress_state: Gd<Label>,
    base: Base<PopupPanel>,
}

#[godot_api]
impl IPopupPanel for ProgressDialog {
    fn ready(&mut self) {
        let main = self.main.clone();
        let mut base = self.base_mut();

        base.add_child(&main);
        base.set_exclusive(true);
        base.set_flag(window::Flags::POPUP, false);
    }
}

impl ProgressDialog {
    pub fn new(process: ForgroundProcess) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            let mut main = VBoxContainer::new_alloc();
            let mut cancel_hb = HBoxContainer::new_alloc();
            let mut cancel = Button::new_alloc();

            cancel.set_text("Cancel");

            cancel_hb.hide();
            cancel_hb.add_spacer(false);
            cancel_hb.add_child(&cancel);
            cancel_hb.add_spacer(false);

            main.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            main.add_child(&cancel_hb);

            let (progress, progress_state, progress_container) =
                Self::progress_bar(process.title, process.tasks);
            let (task_progress, task_progress_state, task_progress_container) =
                Self::progress_bar(process.task_title, process.steps);

            main.add_child(&progress_container);
            main.add_child(&task_progress_container);

            Self {
                main,
                progress,
                progress_state,
                task_progress,
                task_progress_state,
                base,
            }
        })
    }

    fn progress_bar(
        title: &str,
        steps: NonZero<u32>,
    ) -> (Gd<ProgressBar>, Gd<Label>, Gd<VBoxContainer>) {
        let mut vb = VBoxContainer::new_alloc();
        let mut vb2 = VBoxContainer::new_alloc();

        add_margin_child(vb.upcast_mut(), title, &vb2.clone().upcast(), false);

        let mut progress = ProgressBar::new_alloc();

        progress.set_max(steps.get().into());
        progress.set_value(0.0);

        vb2.add_child(&progress);

        let mut state = Label::new_alloc();

        state.set_clip_text(true);

        vb2.add_child(&state);

        (progress, state, vb)
    }

    pub fn popup(&mut self) {
        let mut ms = self.main.get_combined_minimum_size();
        ms.x = maxf(
            500.0 * EditorInterface::singleton().get_editor_scale() as f64,
            ms.x.into(),
        ) as f32;

        let Some(style) = self
            .main
            .get_theme_stylebox_ex("panel")
            .theme_type("PopupMenu")
            .done()
        else {
            return;
        };
        ms += style.get_minimum_size();

        self.main
            .set_offset(Side::LEFT, style.get_margin(Side::LEFT));
        self.main
            .set_offset(Side::RIGHT, -style.get_margin(Side::RIGHT));
        self.main.set_offset(Side::TOP, style.get_margin(Side::TOP));
        self.main
            .set_offset(Side::BOTTOM, -style.get_margin(Side::BOTTOM));

        let mut base = self.base_mut();

        if base.is_inside_tree() {
            base.popup();
        } else {
            // No host window found, use main window.
            EditorInterface::singleton()
                .popup_dialog_centered_ex(&*base)
                .minsize(ms.cast_int())
                .done();
        }
    }

    pub fn push_task(&mut self, task_name: &str) {
        let new_value = self.progress.get_value() + 1.0;

        self.progress.set_value(new_value);
        self.progress_state.set_text(task_name);

        self.task_progress.set_value(0.0);
        self.task_progress_state.set_text("");
    }

    pub fn push_task_step(&mut self, step_name: &str) {
        let new_progress = self.task_progress.get_value() + 1.0;

        self.task_progress.set_value(new_progress);
        self.task_progress_state.set_text(step_name);
    }
}

pub struct ForgroundProcess {
    pub title: &'static str,
    pub tasks: NonZero<u32>,
    pub task_title: &'static str,
    pub steps: NonZero<u32>,
}

fn add_margin_child(
    container: &mut VBoxContainer,
    label: &str,
    control: &Gd<Control>,
    expand: bool,
) -> Gd<MarginContainer> {
    let mut l = Label::new_alloc();

    l.set_theme_type_variation("HeaderSmall");
    l.set_text(label);

    container.add_child(&l);

    let mut mc = MarginContainer::new_alloc();

    mc.add_theme_constant_override("margin_left", 0);
    mc.add_child_ex(control).force_readable_name(true).done();

    container.add_child(&mc);

    if expand {
        mc.set_v_size_flags(SizeFlags::EXPAND_FILL);
    }

    mc
}
