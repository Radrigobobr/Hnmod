use horizon_mod_sdk::{self as sdk, game, hud, input, world, Mod, Registration};

const ITEM_ID: &str = "ak_prototype:ak_prototype";
const MAGAZINE: u32 = 30;
const FIRE_INTERVAL: u8 = 2;

struct AkPrototype {
    magazine: u32,
    reserve: u32,
    trigger: bool,
    cooldown: u8,
    reload_ticks: u8,
}

impl AkPrototype {
    fn reload(&mut self) -> Option<String> {
        if self.reload_ticks > 0 { return Some("AK Prototype: перезарядка уже идёт".into()); }
        if self.magazine >= MAGAZINE { return Some("AK Prototype: магазин полный".into()); }
        if self.reserve == 0 { return Some("AK Prototype: нет запасных патронов 7.42".into()); }
        self.trigger = false;
        self.reload_ticks = 36;
        world::play_sound("minecraft:item.crossbow.loading_start", None, 0.8, 0.9);
        Some("AK Prototype: перезарядка началась".into())
    }

    fn fire(&mut self) {
        let Some(p) = game::get_player() else { return };
        if self.magazine == 0 {
            self.trigger = false;
            world::play_sound("minecraft:block.dispenser.fail", Some((p.x,p.y,p.z)), 0.5, 0.8);
            return;
        }
        let yaw = p.yaw.to_radians();
        let pitch = p.pitch.to_radians();
        let dx = (-yaw.sin() * pitch.cos()) as f64;
        let dy = (-pitch.sin()) as f64;
        let dz = (yaw.cos() * pitch.cos()) as f64;
        let speed = 3.2_f64;
        let command = format!(
            "summon minecraft:arrow {:.3} {:.3} {:.3} {{Motion:[{:.4}d,{:.4}d,{:.4}d],pickup:0b,damage:4.0d}}",
            p.x + dx*0.55, p.y + 1.42 + dy*0.55, p.z + dz*0.55,
            dx*speed, dy*speed, dz*speed
        );
        world::run_commands(&[command]);
        self.magazine -= 1;
        self.cooldown = FIRE_INTERVAL;
        world::play_sound("minecraft:entity.arrow.shoot", Some((p.x,p.y+1.2,p.z)), 1.0, 0.88);
        world::push_player((-dx*0.12) as f32, 0.015, (-dz*0.12) as f32);
        sdk::render::shake(0.18, 0.07);
    }

    fn draw_hud(&self) {
        let status = if self.reload_ticks > 0 {
            format!("RELOADING  {:.1}s", self.reload_ticks as f32 / 20.0)
        } else if self.magazine == 0 {
            "EMPTY  •  PRESS R".to_string()
        } else {
            "7.42 mm  |  FULL AUTO".to_string()
        };
        hud::set_panel(hud::Corner::BottomRight, &[
            hud::Line { text: "AK PROTOTYPE".into(), color: 0xD6B56D },
            hud::Line { text: status, color: 0xE8E8E8 },
            hud::Line {
                text: format!("{:02} / {:03}", self.magazine, self.reserve),
                color: if self.magazine == 0 { 0xFF5555 } else { 0xFFFFFF },
            },
        ]);
    }
}

impl Mod for AkPrototype {
    fn init() -> Result<(Self, Registration), String> {
        input::claim_items(&[ITEM_ID.to_string()]);
        input::register_key("ak_reload", "AK Prototype — перезарядка", "key.keyboard.r");
        sdk::info("AK Prototype 0.1.0 loaded — 7.42 mm");
        Ok((Self {
            magazine: MAGAZINE, reserve: 120, trigger: false, cooldown: 0, reload_ticks: 0,
        }, Registration::new().on_tick().command("ak_reload")))
    }

    fn tick(&mut self, _tick: u64) {
        input::claim_items(&[ITEM_ID.to_string()]);
        for event in input::poll() {
            match event {
                input::Event::UseStart(e) if e.item.id == ITEM_ID => self.trigger = true,
                input::Event::UseStop(e) if e.item.id == ITEM_ID => self.trigger = false,
                input::Event::KeyDown(name) if name == "ak_reload" => { let _ = self.reload(); }
                _ => {}
            }
        }
        if !input::held().is_some_and(|held| held.id == ITEM_ID) { self.trigger = false; }

        if self.reload_ticks > 0 {
            self.reload_ticks -= 1;
            if self.reload_ticks == 0 {
                let loaded = MAGAZINE.saturating_sub(self.magazine).min(self.reserve);
                self.magazine += loaded;
                self.reserve -= loaded;
                world::play_sound("minecraft:item.crossbow.loading_end", None, 0.8, 1.0);
                sdk::chat::show(&format!("AK Prototype: заряжено {} патронов 7.42", loaded));
            }
        } else {
            if self.cooldown > 0 { self.cooldown -= 1; }
            if self.trigger && self.cooldown == 0 { self.fire(); }
        }
        self.draw_hud();
    }

    fn command(&mut self, name: &str, _args: &str) -> Result<Option<String>, String> {
        if name == "ak_reload" { Ok(self.reload()) } else { Ok(None) }
    }
}

horizon_mod_sdk::export_mod!(AkPrototype);
