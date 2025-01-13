use core::cell::RefCell;
use core::mem::Discriminant;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use noli::error::Result as OsResult;
use noli::prelude::MouseEvent;
use noli::prelude::SystemApi;
use noli::print;
use noli::println;
use noli::rect::Rect;
use noli::sys::wasabi::Api;
use noli::window::{StringSize, Window};
use alloc::string::ToString;
use saba_core::browser::Browser;
use saba_core::constants::CONTENT_AREA_HEIGHT;
use saba_core::constants::CONTENT_AREA_WIDTH;
use saba_core::constants::TITLE_BAR_HEIGHT;
use saba_core::constants::WINDOW_PADDING;
use saba_core::constants::{ADDRESSBAR_HEIGHT, BLACK, DARKGREY, GREY, LIGHTGREY, TOOLBAR_HEIGHT, WHITE, WINDOW_HEIGHT, WINDOW_INIT_X_POS, WINDOW_INIT_Y_POS, WINDOW_WIDTH};
use saba_core::display_item::DisplayItem;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use saba_core::renderer::layout::computed_style::FontSize;
use saba_core::renderer::layout::computed_style::TextDecoration;
use crate::cursor::Cursor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    input_url: String,
    input_mode: InputMode,
    window: Window,
    cursor: Cursor
}

impl WasabiUI {
    pub fn new(browser: Rc<RefCell<Browser>>) -> Self {
        Self {
            browser,
            input_url: String::new(),
            input_mode: InputMode::Normal,
            window: Window::new(
                "saba".to_string(),
                WHITE,
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            ).
            unwrap(),
            cursor: Cursor::new()
        }
    }

    fn setup(&mut self) -> Result<(), Error> {
       if let Err(error) = self.setup_toolbar() {
           return Err(Error::InvalidUI(
            format!("failed to initialize a toolbar with error: {:?}", 
            error
            )));
        }
        // 画面を描画する
        self.window.flush();
        Ok(())
    }

    fn setup_toolbar(&mut self) -> OsResult<()>{

        //ツールバーの背景の四角を描画
        self.window.fill_rect(
            LIGHTGREY,
            0,
            0,
            WINDOW_WIDTH,
            TOOLBAR_HEIGHT,
        )?;

        // ツールバーとコンテンツエリアの境界線を描画
        self.window.draw_line(
            GREY, 
            0,
            TOOLBAR_HEIGHT,
            WINDOW_WIDTH - 1, 
            TOOLBAR_HEIGHT)?;
        self.window.draw_line(
            DARKGREY,
            0,
            TOOLBAR_HEIGHT + 1,
            WINDOW_WIDTH - 1,
            TOOLBAR_HEIGHT + 1,
        )?;

        // アドレスバーの横に"Address:"という文字列を描画
        self.window.draw_string(
            BLACK,
            5,
            5,
            "Address:",
            StringSize::Medium,
            /*underline=*/ false,
        )?;

        // アドレスバーの背景の四角を描画
        self.window.fill_rect(
            WHITE,
            70,
            2,
            WINDOW_WIDTH - 74,
            ADDRESSBAR_HEIGHT + 2,
        )?;

        // アドレスバーの影の線を描画
        self.window.draw_line(
            GREY,
            70,
            2,
            WINDOW_WIDTH - 4,
            2,
        )?;
        self.window.draw_line(
            GREY,
            70,
            2,
            70,
            ADDRESSBAR_HEIGHT + 2,
        )?;
        self.window.draw_line(
            BLACK,
            71,
            3,
            WINDOW_WIDTH - 5,
            2,
        )?;

        self.window.draw_line(
            GREY,
            71,
            3,
            71,
            ADDRESSBAR_HEIGHT + 1,
        )?;


        Ok(())


    }


    pub fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        self.setup()?;
        self.run_up(handle_url)?;
        Ok(())
    }

    fn handle_mouse_input(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>
    ) -> Result<(), Error> {
       if let Some(MouseEvent{
        button,
        position
       }) = Api::get_mouse_cursor_info() {
            //println!("mouse position: {:?}", position);
            
            self.window.flush_area(self.cursor.rect());
            self.cursor.set_position(position.x, position.y);
            self.window.flush_area(self.cursor.rect());
            self.cursor.flush();

            if button.l() || button.c() || button.r() {
                // 相対位置を計算する
                let relative_pos = (
                    position.x - WINDOW_INIT_X_POS, 
                    position.y - WINDOW_INIT_Y_POS
                );

                // ウィンドウの外をクリックした場合は何もしない
                if relative_pos.0 < 0 
                    || relative_pos.0 > WINDOW_WIDTH 
                    || relative_pos.1 < 0 
                    || relative_pos.1 > WINDOW_HEIGHT {
                    println!("click outside the window: {button:?} {position:?}");
                    return Ok(());
                }

                // ツールバーの範囲をクリックされた場合は、InputModeをEditingに変更
                if relative_pos.1 < TOOLBAR_HEIGHT + TITLE_BAR_HEIGHT 
                && relative_pos.1 >= TITLE_BAR_HEIGHT {
                    self.clear_address_bar()?;
                    self.input_url = String::new();
                    self.input_mode = InputMode::Editing;
                    println!("button clicked in toolbar: {button:?} {position:?}");
                    return Ok(());
                }

                self.input_mode = InputMode::Normal;

                let position_in_content_area = (
                    relative_pos.0,
                    relative_pos.1 - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT,
                );

                let page = self.browser.borrow().current_page();
                let next_destination = page.borrow_mut().clicked(position_in_content_area);
                if let Some(url) = next_destination { 
                    self.input_url = url.clone();
                    self.update_address_bar()?;
                    self.start_navigation(handle_url, url)?;
                }
            }
            
       }

       Ok(())
    }

    fn handle_key_input(
        &mut self, 
        handle_url: fn(String) -> Result<HttpResponse, Error>
    ) -> Result<(), Error> {
        match self.input_mode {
            InputMode::Normal => {
                let _ = Api::read_key();
            }
            InputMode::Editing => {
                if let Some(c) = Api::read_key() {
                     // エンターキーが押されたのでナビゲーションを開始する
                    if c == 0x0A as char{
                        let _ = self.start_navigation(handle_url, self.input_url.clone());
                        self.input_url = String::new();
                        self.input_mode = InputMode::Normal;
                    // デリートキーまたはバックスペースキーが押された場合
                    } else if c == 0x7F as char || c == 0x08 as char {
                        // 最後の文字を削除
                        self.input_url.pop();
                        self.update_address_bar()?;
                    } else {
                        self.input_url.push(c);
                        self.update_address_bar()?;
                    }
                    println!("input text: {:?}", c);
                }
            }
        }

        Ok(())
    }

    fn start_navigation(
        &mut self, 
        handle_url: fn(String) -> Result<HttpResponse, Error>, 
        destination: String) -> Result<(), Error> 
        {

            // コンテンツエリアをリセット
            self.clear_content_area()?;

            // URLのリクエスト処理を実行する
            match handle_url(destination) {
                Ok(response) => {
                    let page = self.browser.borrow().current_page();
                    page.borrow_mut().receive_response(response);
                }
                Err(e) => {
                    return Err(e);
                }
            }

            self.update_ui()?;

            Ok(())
    }

    fn clear_content_area(&mut self) -> Result<(), Error> {
        if self.window.fill_rect(
            WHITE, 
            0, 
            TOOLBAR_HEIGHT + 2, 
            CONTENT_AREA_WIDTH, 
            CONTENT_AREA_HEIGHT -2
        ).is_err() {
                return Err(Error::InvalidUI(
                    "failed to clear content area".to_string(),
                ));
        }

        self.window.flush();

        Ok(())
    }
   
    fn update_address_bar(&mut self) -> Result<(), Error> {
        // アドレスバーを白く塗りつぶす
        if self.window.fill_rect(
            WHITE,
            72,
            4, 
            WINDOW_WIDTH - 76, 
            ADDRESSBAR_HEIGHT -2
        ).is_err() {
            return Err(Error::InvalidUI("failed to fill address bar".to_string()));
        }

        // input_urlをアドレスバーに描画
        if self.window.draw_string(
            BLACK, 
            74, 
            6, 
            &self.input_url, 
            StringSize::Medium, /*underline=*/
            false
        ).is_err() {
            return Err(Error::InvalidUI("failed to draw address bar".to_string()));
        }

        // アドレスバーの部分の画面を更新する
        self.window.flush_area(
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT
            ).expect("failed to create a rect for the address bar")
        );

        Ok(())

    }

    fn clear_address_bar(&mut self) -> Result<(), Error> {
        // アドレスバーを白く塗りつぶす
        if self.window.fill_rect(
            WHITE, 
            72, 
            4, 
            WINDOW_WIDTH - 76,
            ADDRESSBAR_HEIGHT - 2
        ).is_err() {
            return Err(Error::InvalidUI("failed to fill address bar".to_string()));
        }
        
        // アドレスバーの部分の画面を更新する
        self.window.flush_area(
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT
            ).expect("failed to create a rect for the address bar")
        );

        Ok(())
    }

    fn update_ui(&mut self) -> Result<(), Error> {
        let display_items = self
        .browser
        .borrow()
        .current_page()
        .borrow()
        .display_items();

        for item in display_items {
            println!("{:?}", item);
            match item {
                DisplayItem::Text {
                    text,
                    style,
                    layout_point
                } => {
                    if self.window.draw_string(
                        style.color().code_u32(),
                        layout_point.x() + WINDOW_PADDING,
                        layout_point.y() + WINDOW_PADDING + TOOLBAR_HEIGHT,
                        &text,
                        convert_font_size(style.font_size()),
                        style.text_decoration() == TextDecoration::Underline,
                    ).is_err(){
                        return Err(Error::InvalidUI("failed to draw a string".to_string()))
                    }
                }
                DisplayItem::Rect {
                    style,
                    layout_point,
                    layout_size
                } => {
                    if self.window.fill_rect(
                        style.background_color().code_u32(),
                        layout_point.x() + WINDOW_PADDING,
                        layout_point.y() + WINDOW_PADDING + TOOLBAR_HEIGHT,
                        layout_size.width(),
                        layout_size.height()
                    ).is_err() {
                        return Err(Error::InvalidUI("failed to draw a string".to_string()))
                    }
                }
            }
        }

        self.window.flush();

        Ok(())
    }
    

    pub fn run_up(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        loop {
            // キーボードの入力を処理
            self.handle_key_input(handle_url)?;
            // マウスの入力を処理
            self.handle_mouse_input(handle_url)?;
        }
    }


}




fn convert_font_size(size: FontSize) -> StringSize {
    match size {
        FontSize::Medium => StringSize::Medium,
        FontSize::XLarge => StringSize::Large,
        FontSize::XXLarge => StringSize::XLarge,
    }
}