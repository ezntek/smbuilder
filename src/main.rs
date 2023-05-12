// Copyright 2023 Eason Qin <eason@ezntek.com>.
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use gtk4::prelude::*;
use gtk4::glib;

fn on_click(btn: &gtk4::Button) {
    btn.set_label("you clicked me!");
}

fn on_app_activate(app: &adw::Application) {
    let builder = gtk4::Builder::from_file("./ui_xml/main.ui");

    let window: gtk4::Window = builder.object("window").unwrap();    
    window.set_application(Some(app));
    let button: gtk4::Button = builder.object("btn").unwrap();
    button.connect_clicked(on_click);

    window.show();
}

fn main() -> glib::ExitCode {
    let application = adw::Application::builder()
        .application_id("com.ezntek.smbuilder")
        .build();

    application.connect_activate(on_app_activate);
    application.run()
}
