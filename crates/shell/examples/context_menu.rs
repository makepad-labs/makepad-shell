use makepad_shell::{
    CommandId, CommandItem, MenuAnchor, MenuItem, MenuModel, MenuTrigger, Submenu,
};

fn main() {
    let copy = CommandId::new(1).unwrap();
    let paste = CommandId::new(2).unwrap();

    let menu = MenuModel::new(vec![
        MenuItem::Command(CommandItem::new(copy, "Copy")),
        MenuItem::Command(CommandItem::new(paste, "Paste")),
        MenuItem::Separator,
        MenuItem::Submenu(Submenu::new(
            "View",
            vec![
                MenuItem::Command(CommandItem {
                    id: CommandId::new(3).unwrap(),
                    label: "Show Grid".into(),
                    enabled: true,
                    checked: true,
                    shortcut: None,
                    role: None,
                }),
            ],
        )),
    ]);

    let result = makepad_shell::popup_context_menu(
        menu,
        MenuAnchor::Screen { x: 200.0, y: 200.0 },
        MenuTrigger::MouseRight,
        |cmd| {
            println!("Command invoked: {:?}", cmd);
        },
    );
    
    println!("popup_context_menu result = {:?}", result);
}
