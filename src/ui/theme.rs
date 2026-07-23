use macroquad::prelude::*;

pub struct Theme {
    pub inter_bold: Font,
    pub inter_semibold: Font,
    pub inter_medium: Font,
    pub inter_regular: Font,
}

impl Theme {
    pub async fn load() -> Self {
        Self {
            inter_bold: load_ttf_font("assets/fonts/Inter-Bold.ttf")
                .await
                .unwrap(),

            inter_semibold: load_ttf_font("assets/fonts/Inter-SemiBold.ttf")
                .await
                .unwrap(),

            inter_medium: load_ttf_font("assets/fonts/Inter-Medium.ttf")
                .await
                .unwrap(),

            inter_regular: load_ttf_font("assets/fonts/Inter-Regular.ttf")
                .await
                .unwrap(),
        }
    }
}