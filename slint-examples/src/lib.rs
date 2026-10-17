#![cfg(target_os = "android")]

slint::slint! {
    import { AboutSlint, Button, LineEdit, ScrollView, VerticalBox } from "std-widgets.slint";

    export component MainWindow inherits Window {
        in-out property <int> counter;

        ScrollView {
            x: root.safe-area-insets.left;
            y: root.safe-area-insets.top;
            width: root.width - root.safe-area-insets.left - root.safe-area-insets.right;
            height: root.height - root.safe-area-insets.top - root.safe-area-insets.bottom;

            VerticalBox {
                alignment: center;

                AboutSlint { }

                name := LineEdit {
                    placeholder-text: "Your name";
                }

                Text {
                    text: name.text.is-empty ? "Type your name above" : "Hello, " + name.text;
                    horizontal-alignment: center;
                }

                Button {
                    text: "Clicked " + root.counter + " times";
                    clicked => { root.counter += 1; }
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    MainWindow::new().unwrap().run().unwrap();
}
