pub struct Chat {
    json: String,
    first: bool,
    first_extra: bool,
    click_event_defined: bool,
    hover_event_defined: bool,
    is_first_text: bool,
    extra_exists: bool,
}

impl Chat {
    pub fn new() -> Self {
        let mut json = String::with_capacity(64);
        json += "{";

        Self {
            json,
            first: true,
            first_extra: true,
            click_event_defined: false,
            hover_event_defined: false,
            is_first_text: true,
            extra_exists: false,
        }
    }

    #[inline]
    pub fn text(mut self, v: &str) -> Self {
        if !self.is_first_text {
            self.first = true; // Prevent extra `,`
            return self.extra().text(v);
        }

        self.is_first_text = false;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"text\":\"";
        self.json += v;
        self.json += "\"";
        self
    }

    #[inline]
    pub fn bold(mut self) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"bold\":true";
        self
    }

    #[inline]
    pub fn italic(mut self) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"italic\":true";
        self
    }

    #[inline]
    pub fn underlined(mut self) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"underlined\":true";
        self
    }

    #[inline]
    pub fn strikethrough(mut self) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"strikethrough\":true";
        self
    }

    #[inline]
    pub fn obfuscated(mut self) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"obfuscated\":true";
        self
    }

    #[inline]
    pub fn reset(mut self) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"reset\":true";
        self
    }

    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        if !self.first {
            self.json += ",";
        }

        self.first = false;

        if color == Color::Black {
            self.json += "\"color\":0";
        } else {
            self.json += &format!("\"color\":\"{}\"", color);
        }

        self
    }

    /// The `url` must begin with `http`. Panics if it doesn't.
    #[inline]
    pub fn click_url(mut self, url: &str) -> Self {
        if self.click_event_defined {
            return self;
        }

        self.click_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        if !url.starts_with("http") {
            panic!("Chat::url() must begin with http!");
        }

        self.json += "\"clickEvent\":{\"action\":\"open_url\",\"value\":\"";
        self.json += url;
        self.json += "\"}";
        self
    }

    #[inline]
    pub fn click_run_command(mut self, command: &str) -> Self {
        if self.click_event_defined {
            return self;
        }

        self.click_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"clickEvent\":{\"action\":\"run_command\",\"value\":\"";
        self.json += command;
        self.json += "\"}";
        self
    }

    #[inline]
    pub fn click_suggest_command(mut self, command: &str) -> Self {
        if self.click_event_defined {
            return self;
        }

        self.click_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"clickEvent\":{\"action\":\"suggest_command\",\"value\":\"";
        self.json += command;
        self.json += "\"}";
        self
    }

    #[inline]
    pub fn click_change_book_page(mut self, page_to: usize) -> Self {
        if self.click_event_defined {
            return self;
        }

        self.click_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"clickEvent\":{\"action\":\"suggest_command\",\"value\":\"";
        self.json += &page_to.to_string();
        self.json += "\"}";
        self
    }

    #[inline]
    pub fn hover_show_text(mut self, v: &str) -> Self {
        if self.hover_event_defined {
            return self;
        }

        self.hover_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"hoverEvent\":{\"action\":\"show_text\",\"value\":\"";
        self.json += v;
        self.json += "\"}";
        self
    }

    // This is pretty much useless?
    #[inline]
    pub fn hover_show_text_chat(mut self, v: Chat) -> Self {
        if self.hover_event_defined {
            return self;
        }

        self.hover_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"hoverEvent\":{\"action\":\"show_text\",\"value\":\"";
        self.json += &v.finish();
        self.json += "\"}";
        self
    }

    // TODO: perhaps whenever NBT is really supported by us, require to input it instead?
    #[inline]
    pub fn hover_show_item(mut self, v: &str) -> Self {
        if self.hover_event_defined {
            return self;
        }

        self.hover_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"hoverEvent\":{\"action\":\"show_item\",\"value\":\"";
        self.json += v;
        self.json += "\"}";
        self
    }

    // TODO: same as above ^, support NBT
    #[inline]
    pub fn hover_show_entity(mut self, v: &str) -> Self {
        if self.hover_event_defined {
            return self;
        }

        self.hover_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"hoverEvent\":{\"action\":\"show_entity\",\"value\":\"";
        self.json += v;
        self.json += "\"}";
        self
    }

    // TODO: same as above ^, but support achievements.
    #[inline]
    pub fn hover_show_achievement(mut self, v: &str) -> Self {
        if self.hover_event_defined {
            return self;
        }

        self.hover_event_defined = true;

        if !self.first {
            self.json += ",";
        }

        self.first = false;

        self.json += "\"hoverEvent\":{\"action\":\"show_achievement\",\"value\":\"";
        self.json += v;
        self.json += "\"}";
        self
    }

    #[inline]
    pub fn extra(mut self) -> Self {
        if !self.first || !self.extra_exists {
            self.json += ",";
        }

        self.hover_event_defined = false;
        self.click_event_defined = false;
        self.first = true;
        self.is_first_text = true;
        self.extra_exists = true;

        if self.first_extra {
            self.first_extra = false;
            self.json += "\"extra\":[{";
        } else {
            self.json += "},{";
        }

        self
    }

    // The purpose of this function is to add spacing between text components, without formatting or anything.
    #[inline]
    pub fn space(self) -> Self {
        self.text(" ")
    }

    #[inline]
    pub fn finish(mut self) -> String {
        if !self.first_extra {
            self.json += "}]";
        }

        self.json += "}";
        self.json
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    Purple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    BrightGreen,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Black => f.write_str("black"),
            Color::DarkBlue => f.write_str("dark_blue"),
            Color::DarkGreen => f.write_str("dark_green"),
            Color::DarkCyan => f.write_str("dark_aqua"),
            Color::DarkRed => f.write_str("dark_red"),
            Color::Purple => f.write_str("dark_purple"),
            Color::Gold => f.write_str("gold"),
            Color::Gray => f.write_str("gray"),
            Color::DarkGray => f.write_str("dark_gray"),
            Color::Blue => f.write_str("blue"),
            Color::BrightGreen => f.write_str("green"),
            Color::Cyan => f.write_str("aqua"),
            Color::Red => f.write_str("red"),
            Color::Pink => f.write_str("light_purple"),
            Color::Yellow => f.write_str("yellow"),
            Color::White => f.write_str("white"),
        }
    }
}
