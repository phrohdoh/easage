#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::rc::Rc;
use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::io::Write;

extern crate easage as lib;
use lib::Archive;

extern crate number_prefix;
use number_prefix::{binary_prefix, Standalone, Prefixed};

extern crate gdk;
use gdk::enums::key;

extern crate gtk;
use gtk::prelude::Inhibit;

use gtk::{
    Builder,
    Window,
    WindowType,
    Button,
    Entry as EntryBox,
    FileChooserDialog,
    ListStore,
    Type,
    TreeSelection,
    TreeView,
    TreeViewColumn
};

use gtk::{
    BuilderExt,
    ButtonExt,
    CellLayoutExt,
    DialogExt,
    EntryExt,
    FileChooserExt,
    GtkWindowExt,
    ListStoreExt,
    ListStoreExtManual,
    TreeModelExt,
    TreeSelectionExt,
    TreeViewExt,
    TreeViewColumnExt,
    WidgetExt,
};

fn main() {
    gtk::init().unwrap();

    let builder = Builder::new();
    builder.add_from_string(include_str!("../ui.glade")).unwrap();
    let window: Window = builder.get_object("main_window").unwrap();
    let archive_entrybox: EntryBox = builder.get_object("archive_file_entry").unwrap();
    let archive_button: Button = builder.get_object("archive_file_button").unwrap();
    let extract_button: Button = builder.get_object("extract_button").unwrap();
    let entryinfo_tree = {
        let t: TreeView = builder.get_object("entryinfo_tree").unwrap();
        let sel = t.get_selection();
        sel.set_mode(gtk::SelectionMode::Multiple);
        t
    };

    window.set_title("BIG Editor");
    window.set_position(gtk::WindowPosition::Center);
    window.get_preferred_width();
    window.set_default_size(1440, 900);

    let ei_store = ListStore::new(&[Type::String, Type::String, Type::String]);

    macro_rules! add_column {
        ($tree:ident, $title:expr, $id:expr) => {{
            let column = TreeViewColumn::new();
            let renderer = gtk::CellRendererText::new();
            column.set_title($title);
            column.set_resizable(true);
            column.pack_start(&renderer, true);
            column.add_attribute(&renderer, "text", $id);
            $tree.append_column(&column);
        }}
    }

    entryinfo_tree.set_model(Some(&ei_store));
    entryinfo_tree.set_headers_visible(true);

    add_column!(entryinfo_tree, "Name", 0);
    add_column!(entryinfo_tree, "Size", 1);
    add_column!(entryinfo_tree, "Offset", 2);

    fn setup_tree(tree: TreeView, extract_button: Button) {
        let sel = tree.get_selection();
        let model = match tree.get_model() {
            Some(m) => m,
            _ => return,
        };

        sel.connect_changed(move |this| {
            // TODO: Do all of this when an archive is opened, too.
            let selected_count = this.count_selected_rows();
            let store_len = model.iter_n_children(None);

            let count_str = if selected_count == 0 || selected_count == store_len {
                "all".into()
            } else {
                format!("({})", selected_count)
            };

            extract_button.set_label(&format!("Extract {}", count_str))
        });
    }

    setup_tree(entryinfo_tree.clone(), extract_button.clone());

    entryinfo_tree.connect_row_activated(move |this, path, column| {
        println!("connect_row_activated(_, {}, {:?})", path, column);
    });

    let archive: Rc<RefCell<Option<Archive>>> = Rc::new(RefCell::new(None));
    // TODO
    // let archive_table: Rc<RefCell<Option<HashMap<String, EntryInfo>>>> = Rc::new(RefCell::new(None));

    let archive_entrybox_clone = archive_entrybox.clone();
    let archive1 = archive.clone();
    archive_button.connect_clicked(move |_this| {
        let dialog = FileChooserDialog::new(
            Some("Select a BIG archive"),
            Some(&Window::new(WindowType::Popup)),
            gtk::FileChooserAction::Open
        );

        dialog.add_button("_Cancel", gtk::ResponseType::Cancel.into());
        dialog.add_button("_Open", gtk::ResponseType::Ok.into());

        if dialog.run() == gtk::ResponseType::Ok.into() {
            dialog.get_filename().map(|path| path.to_str().map(|s| archive_entrybox_clone.set_text(s)));
        } else {
            archive_entrybox_clone.set_text("");
        }

        dialog.destroy();

        if let Some(archive_path) = archive_entrybox_clone.get_text() {
            if !archive_path.is_empty() {
                let mut a = Archive::from_path(archive_path).unwrap();
                let table = a.read_entry_metadata_table().unwrap();

                *archive1.borrow_mut() = Some(a);

                ei_store.clear();

                for (name_path, info) in table.iter() {
                    let float_len = info.len as f32;

                    let formatted_size = match binary_prefix(float_len) {
                        Standalone(bytes) => format!("{} B", bytes),
                        Prefixed(prefix, n) => format!("{:.2} {}B", n, prefix),
                    };

                    ei_store.insert_with_values(None,
                        &[0, 1, 2],
                        &[
                            &name_path.to_string(),
                            &formatted_size,
                            &format!("{:#X}", info.offset),
                        ]
                    );
                }
            }
        }
    });

    let archive2 = archive.clone();
    extract_button.connect_clicked(move |_this| {
        let sel = entryinfo_tree.get_selection();
        let (mut sel_paths, model) = sel.get_selected_rows();

        let dialog = FileChooserDialog::new(
            Some("Select a directory to extract to"),
            Some(&Window::new(WindowType::Toplevel)),
            gtk::FileChooserAction::SelectFolder
        );

        dialog.add_button("_Cancel", gtk::ResponseType::Cancel.into());
        let resp_ok = gtk::ResponseType::Ok.into();
        dialog.add_button("_Select", resp_ok);

        let dest_dir_path: PathBuf;
        let dialog_res = dialog.run();
        let filename = dialog.get_filename();
        dialog.destroy();

        /* KNOWN BUG:
         * Hitting esc *or* clicking the cancel button
         * will still extract files.
         */

        match (dialog_res, filename) {
            (_resp_ok, Some(path)) => dest_dir_path = path,
            _ => return,
        };

        let mut a = archive2.borrow_mut();
        let a = a.as_mut().unwrap();
        let table = a.read_entry_metadata_table().unwrap();

        if sel_paths.len() == 0 {
            sel.select_all();
            let (s, _) = sel.get_selected_rows();
            sel_paths = s;
            sel.unselect_all();
        }

        for sel_path in sel_paths {
            if let Some(iter) = model.get_iter(&sel_path) {
                let val = model.get_value(&iter, 0);
                let name = val
                    .get::<String>()
                    .expect(&format!("Unable to convert gtk::Type::String {:?} to a Rust String", val));

                if let Some(data) = a.get_bytes_via_table(&table, &name) {
                    let mut output_filepath = dest_dir_path.clone();
                    output_filepath.push(name.replace("\\", "/"));

                    let parent = output_filepath.parent()
                        .expect(&format!("Unable to determine parent path of {:?}", &output_filepath));

                    fs::create_dir_all(&parent)
                        .expect("Failed to create necessary parent directories");
                    let mut f = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .truncate(true)
                        .open(&output_filepath)
                        .expect(&format!("Failed to open file {:?} for writing", output_filepath));

                    f.write(data).expect("Failed to write data");
                }
            }
        }
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.connect_key_press_event(move |_, key| {
        if let key::Escape = key.get_keyval() {
            gtk::main_quit();
        }

        Inhibit(false)
    });

    window.show_all();
    gtk::main();
}
