pub struct Info {
    title: &'static str,
    application_id: &'static str,
    default_width: i32,
    default_height: i32,
}

const DEFAULT_TITLE: &'static str = "LogicRs";
const DEFAULT_APPLICAITON_ID: &'static str = "com.spydr06.LogicRs";
const DEFAULT_WIDTH: i32 = 1280;
const DEFAULT_HEIGHT: i32 = 720;

impl Info {
    pub fn new() -> Self {
        Self {
            application_id: DEFAULT_APPLICAITON_ID,
            title: DEFAULT_TITLE,
            default_width: DEFAULT_WIDTH,
            default_height: DEFAULT_HEIGHT,
        }
    }

    pub fn title(self, title: &'static str) -> Self {
        let mut new = self;
        new.title = title;
        new
    }

    pub fn get_title(&self) -> &'static str {
        self.title
    }

    pub fn default_size(self, width: i32, height: i32) -> Self {
        let mut new = self;
        new.default_width = width;
        new.default_height = height;
        new
    }

    pub fn get_app_id(&self) -> &'static str {
        self.application_id
    }
}
